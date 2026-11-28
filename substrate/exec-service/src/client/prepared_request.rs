//! Exact request preparation and submission through the existing Exec REST
//! service. Preparation never submits a run or uploads/pins artifacts.
//! KBD must separately admit the selected reservation before invoking submit.
use std::{collections::{BTreeMap, BTreeSet}, fs, io::Write, path::{Path, PathBuf}, process::ExitCode, time::Duration};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use hyper::Method;
use prometheus_exec_contracts::{
    self as contracts, CapabilityManifest, CodeIdentity, CodeKind, Digest, EvidenceIdentity,
    ExecutionLimits, ExecutionProvenance, ExecutionReceipt, NamedInput, RequestedTier, RunState,
    RuntimeKind, SignatureAlgorithm, SignedExecRequest,
};
use prometheus_exec_core::ArtifactStore;
use crate::{ReadinessSnapshot, ReadinessStatus, RunRecord, RunResponse};
use uuid::Uuid;
use super::{identity, uds_client, BoxError, fetch_run, mebibytes, release_pins};

#[derive(Debug)]
pub struct PrepareRequest {
    pub identity: PathBuf,
    pub code: PathBuf,
    pub inputs: Vec<String>,
    pub phase: String,
    pub start: String,
    pub toolchain_hash: String,
    pub validity_window_secs: u64,
    pub timeout_ms: u64,
    pub output_mb: u64,
    pub request_out: PathBuf,
}

#[derive(Debug)]
pub struct SubmitRequest {
    pub socket: PathBuf,
    pub state_dir: PathBuf,
    pub identity: PathBuf,
    pub request: PathBuf,
    pub expected_request_hash: String,
    pub code: PathBuf,
    pub inputs: Vec<String>,
    pub receipt_out: PathBuf,
    pub artifact_root: PathBuf,
    pub reservation_started_at: DateTime<Utc>,
    pub complete_before: DateTime<Utc>,
    pub expected_runner_key: String,
    pub artifact_budget_mb: u64,
    /// Reconcile a previously accepted exact request without any POST/upload.
    pub reconcile_only: bool,
}


const RUNNER_SHA: &str = "sha256:6ca67fd7b411ec30945ee903228a2b0501bca78c7b442bcef4196b8e97310376";
const PYTHON_SHA: &str = "sha256:1025ceb64342aa35b18fa36ab7e0d4d3732566a9e172fd9eff9c4d82d9c617ee";
const INTENT: &str = ".selected-exec-dispatch-intent.json";
pub(super) fn installed_home() -> Result<PathBuf, BoxError> {
    #[cfg(unix)]
    { Ok(nix::unistd::User::from_uid(nix::unistd::Uid::effective())?.ok_or("effective user has no home")?.dir) }
    #[cfg(not(unix))]
    { Err("selected Exec host is unsupported".into()) }
}
fn verify_installed_paths(args: &SubmitRequest) -> Result<(), BoxError> {
    let home=installed_home()?;
    let state=home.join(".prometheus/exec");
    let socket=home.join(".prometheus/run/prometheus-exec.sock");
    let identity=state.join("identity.json");
    // The only enabled profile is the already managed local install. No
    // caller-selected socket, alternate state or credential is accepted.
    if args.state_dir!=state || args.socket!=socket || args.identity!=identity {
        return Err("selected request requires the existing managed Exec socket, state and identity together".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{FileTypeExt,MetadataExt,PermissionsExt};
        let uid=nix::unistd::Uid::effective().as_raw();
        for (path,mode) in [(&state,0o700),(&identity,0o600),(&socket,0o600)] {
            let info=fs::symlink_metadata(path)?;
            if info.uid()!=uid || info.file_type().is_symlink() || info.permissions().mode()&0o777!=mode {
                return Err("managed Exec path ownership/permissions changed".into());
            }
            if path==&state && !info.is_dir() || path==&identity && !info.is_file() || path==&socket && !info.file_type().is_socket() {
                return Err("managed Exec path type changed".into());
            }
        }
    }
    if contracts::hash_bytes(&fs::read(home.join(".local/bin/prometheus-exec"))?).as_str()!=RUNNER_SHA
        || contracts::hash_bytes(&fs::read("/usr/bin/python3")?).as_str()!=PYTHON_SHA {
        return Err("qualified installed runner or Python toolchain changed".into());
    }
    Ok(())
}
fn ensure_window(args:&SubmitRequest, request:&SignedExecRequest) -> Result<(),BoxError>{
    let now=Utc::now();
    let maximum=ChronoDuration::try_milliseconds(i64::try_from(request.limits.wall_clock_ms)?).ok_or("wall budget overflow")?;
    if args.reservation_started_at<request.issued_at || now<args.reservation_started_at
        || args.complete_before>request_expiry(request)?
        || now.checked_add_signed(maximum).is_none_or(|end|end>args.complete_before) {
        return Err("insufficient remaining exact selected reservation (no dispatch)".into());
    }
    Ok(())
}
fn record_intent(args:&SubmitRequest, request:&SignedExecRequest,expected:&Digest)->Result<(),BoxError>{
    let info=fs::symlink_metadata(&args.artifact_root)?;
    if !info.is_dir() || info.file_type().is_symlink(){return Err("invalid selected artifact root".into());}
    let path=args.artifact_root.join(INTENT);
    let content=serde_json::to_vec_pretty(&serde_json::json!({"requestId":request.request_id,"requestHash":expected,
        "reservationStartedAt":args.reservation_started_at,"completeBefore":args.complete_before,"dispatchIntentAt":Utc::now()}))?;
    let mut temp=tempfile::NamedTempFile::new_in(&args.artifact_root)?;
    temp.write_all(&content)?;
    temp.as_file().sync_all()?;
    temp.persist_noclobber(&path).map_err(|_| "selected dispatch intent already recorded or unavailable; reconcile only")?;
    fs::File::open(&args.artifact_root)?.sync_all()?;
    Ok(())
}

fn inputs(values: &[String]) -> Result<BTreeMap<String, Vec<u8>>, BoxError> {
    let mut out = BTreeMap::new();
    for value in values {
        let (name, path) = value.split_once('=').ok_or("input must be NAME=PATH")?;
        if name.is_empty() || out.contains_key(name) { return Err("empty or duplicate input name".into()); }
        out.insert(name.to_owned(), fs::read(path)?);
    }
    if !out.contains_key("candidate-manifest.json") { return Err("selected request must include the candidate manifest input".into()); }
    Ok(out)
}

fn request_expiry(request: &SignedExecRequest) -> Result<chrono::DateTime<Utc>, BoxError> {
    let seconds = i64::try_from(request.validity_window_secs)?;
    request.issued_at.checked_add_signed(ChronoDuration::try_seconds(seconds).ok_or("validity overflow")?)
        .ok_or_else(|| "validity overflow".into())
}

fn ensure_profile(request: &SignedExecRequest) -> Result<(), BoxError> {
    request.validate()?;
    if request.tier != RequestedTier::P || request.code.kind != CodeKind::File
        || request.code.runtime != RuntimeKind::Python3 || request.code.toolchain_pin.as_ref().map(Digest::as_str)!=Some(PYTHON_SHA)
        || request.capabilities != CapabilityManifest::default() || !request.targets.is_empty()
        || request.provenance.phase.as_deref().is_none_or(str::is_empty)
        || request.provenance.change.as_deref().is_none_or(str::is_empty)
        || request.provenance.component_authorization.is_some()
        || request.sig_alg != SignatureAlgorithm::Ed25519 {
        return Err("unsupported prepared request: only exact phase/start-bound local Python Tier P is eligible".into());
    }
    Ok(())
}

fn write_new_or_equal(path: &Path, bytes: &[u8]) -> Result<(), BoxError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {
            if fs::read(path)? == bytes { return Ok(()); }
            return Err(format!("refusing to overwrite different evidence at {}", path.display()).into());
        }
        Ok(_) => return Err(format!("unsafe evidence file {}", path.display()).into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {},
        Err(error) => return Err(error.into()),
    }
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(bytes)?;
    file.as_file().sync_all()?;
    file.persist_noclobber(path)?;
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}

fn artifact_destination(root: &Path, logical: &str) -> Result<PathBuf, BoxError> {
    contracts::validate_artifact_path(logical)?;
    let mut current = root.to_owned();
    let parts = Path::new(logical).components().collect::<Vec<_>>();
    for part in &parts[..parts.len()-1] {
        current.push(part.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(info) if info.is_dir() && !info.file_type().is_symlink() => {},
            Ok(_) => return Err("unsafe artifact parent in archive".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => fs::create_dir(&current)?,
            Err(error) => return Err(error.into()),
        }
    }
    current.push(parts[parts.len()-1].as_os_str());
    Ok(current)
}

pub fn prepare(args: PrepareRequest) -> Result<ExitCode, BoxError> {
    if !cfg!(all(target_os="macos",target_arch="aarch64")) {
        return Err("prepared request profile is unsupported on this host".into());
    }
    if args.request_out.try_exists()? { return Err("request output already exists; no new signature".into()); }
    if args.phase.trim().is_empty() || args.start.trim().is_empty() || args.timeout_ms == 0 || args.output_mb == 0
        || args.validity_window_secs.saturating_mul(1000) < args.timeout_ms {
        return Err("invalid explicit prepared request scope or budget".into());
    }
    let home=installed_home()?;
    if args.identity!=home.join(".prometheus/exec/identity.json"){return Err("prepared request must use the managed identity".into());}
    let loaded = identity::load(&args.identity)?;
    let code = fs::read(&args.code)?;
    let named = inputs(&args.inputs)?.into_iter().map(|(name, bytes)| NamedInput {
        name, hash: contracts::hash_bytes(&bytes)
    }).collect();
    let now = Utc::now();
    let mut request = SignedExecRequest {
        schema_version: contracts::SCHEMA_VERSION.into(),
        request_id: Uuid::new_v4(),
        issued_at: now,
        queued_at: Some(now),
        validity_window_secs: args.validity_window_secs,
        tier: RequestedTier::P,
        code: CodeIdentity { kind: CodeKind::File, hash: contracts::hash_bytes(&code),
            runtime: RuntimeKind::Python3, toolchain_pin: Some(Digest::parse(args.toolchain_hash)?) },
        inputs: named,
        capabilities: CapabilityManifest::default(),
        limits: ExecutionLimits { wall_clock_ms: args.timeout_ms, output_mb: args.output_mb, ..ExecutionLimits::default() },
        targets: vec![],
        provenance: ExecutionProvenance { harness: Some("prometheus-exec-cli".into()),
            phase: Some(args.phase), change: Some(args.start), ..ExecutionProvenance::default() },
        signer_key_id: None, sig_alg: SignatureAlgorithm::Ed25519, signature: None,
    };
    ensure_profile(&request)?;
    request_expiry(&request)?;
    contracts::sign_request_ed25519(&mut request, &loaded.signing_key)?;
    contracts::verify_request_signature(&request, &loaded.verification_key)?;
    let bytes = serde_json::to_vec_pretty(&request)?;
    write_new_or_equal(&args.request_out, &bytes)?;
    let public = EvidenceIdentity { sig_alg: loaded.file.sig_alg, key_id: loaded.file.key_id.clone(), public_key: loaded.file.public_key.clone() };
    println!("{}",serde_json::to_string_pretty(&serde_json::json!({
        "requestId":request.request_id,"requestHash":request.request_hash()?,
        "requestFileHash":contracts::hash_bytes(&bytes),"requestPath":args.request_out,
        "identity":public,"issuedAt":request.issued_at,"expiresAt":request_expiry(&request)?,
        "dispatched":false
    }))?);
    Ok(ExitCode::SUCCESS)
}

fn existing_record(state_dir: &Path, request: &SignedExecRequest) -> Result<Option<RunRecord>, BoxError> {
    let root = state_dir.join("service/ledger/runs");
    if !root.is_dir() { return Err("installed durable request ledger is unavailable".into()); }
    let path = root.join(format!("{}.json",request.request_id));
    let metadata = match fs::symlink_metadata(&path) {
        Ok(info) => info,
        Err(error) if error.kind()==std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len()>8*1024*1024 {
        return Err("unsafe durable request record".into());
    }
    let record: RunRecord = serde_json::from_slice(&fs::read(path)?)?;
    record.validate()?;
    if record.request != *request || record.request_hash != request.request_hash()? {
        return Err("request identity conflicts with the existing durable record; no dispatch".into());
    }
    Ok(Some(record))
}

async fn check_ready(socket: &Path, key_id: &str) -> Result<(), BoxError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::{FileTypeExt, PermissionsExt};
        let meta=fs::symlink_metadata(socket)?;
        if !meta.file_type().is_socket() || meta.permissions().mode() & 0o777 != 0o600 {
            return Err("expected existing mode-0600 Unix socket".into());
        }
    }
    let response = tokio::time::timeout(Duration::from_secs(5),uds_client::request_same_uid(socket,Method::GET,"/ready",vec![])).await??;
    if response.status != 200 { return Err("existing runner readiness refused".into()); }
    let ready: ReadinessSnapshot=serde_json::from_slice(&response.body)?;
    let signer=ready.subsystems.get("receipt-identity").ok_or("daemon receipt identity missing")?;
    if !ready.ready || signer.status != ReadinessStatus::Ready || signer.detail != format!("receipt signer {key_id} is ready") {
        return Err("daemon readiness or live signer differs from exact prepared request".into());
    }
    Ok(())
}

fn check_response(run: &RunResponse, request: &SignedExecRequest, expected: &Digest) -> Result<(), BoxError> {
    if run.request_id != request.request_id || &run.request_hash != expected {
        return Err("daemon response differs from exact prepared request".into());
    }
    Ok(())
}

async fn archive(args: &SubmitRequest, receipt: &ExecutionReceipt, request: &SignedExecRequest,
    loaded: &identity::LoadedIdentity) -> Result<(), BoxError> {
    let checked=contracts::verify_receipt(receipt,&loaded.verification_key,Some(request),None);
    if !checked.valid {return Err(format!("receipt verification failed: {:?}",checked.failures).into());}
    let root=args.artifact_root.canonicalize()?;
    if !root.is_dir() || fs::symlink_metadata(&args.artifact_root)?.file_type().is_symlink() {
        return Err("artifact archive must be an existing non-symlink directory".into());
    }
    let mut seen=BTreeSet::new();
    for artifact in &receipt.outputs.artifacts {
        if !seen.insert(&artifact.path) { return Err("duplicate artifact in signed receipt".into()); }
        contracts::validate_artifact_path(&artifact.path)?;
        let response=tokio::time::timeout(Duration::from_secs(10),uds_client::request_same_uid(&args.socket,Method::GET,
            &format!("/api/v2/exec/artifacts/{}",artifact.hash),vec![])).await??;
        if response.status != 200 || contracts::hash_bytes(&response.body)!=artifact.hash
            || artifact.size_bytes.is_some_and(|n|n!=response.body.len() as u64) {
            return Err("signed artifact bytes unavailable or mismatched".into());
        }
        write_new_or_equal(&artifact_destination(&root,&artifact.path)?,&response.body)?;
    }
    let verified=contracts::verify_receipt(receipt,&loaded.verification_key,Some(request),Some(&root));
    if !verified.valid {return Err(format!("archived receipt/artifact verification failed: {:?}",verified.failures).into());}
    write_new_or_equal(&args.receipt_out,&serde_json::to_vec_pretty(receipt)?)?;
    Ok(())
}

pub async fn submit<F,Fut>(args: SubmitRequest, before_post:F) -> Result<ExitCode, BoxError>
where F:FnOnce()->Fut,Fut:std::future::Future<Output=Result<(),BoxError>> {
    if !cfg!(all(target_os="macos",target_arch="aarch64")) { return Err("prepared request profile unsupported".into()); }
    verify_installed_paths(&args)?;
    let loaded=identity::load(&args.identity)?;
    if loaded.file.key_id!=args.expected_runner_key{return Err("managed signer differs from canonical selected authority".into());}
    let bytes=fs::read(&args.request)?;
    let request:SignedExecRequest=serde_json::from_slice(&bytes)?;
    ensure_profile(&request)?;
    contracts::verify_request_signature(&request,&loaded.verification_key)?;
    let expected=Digest::parse(args.expected_request_hash.clone())?;
    if request.request_hash()?!=expected {return Err("exact prepared request hash mismatch".into());}
    let code=fs::read(&args.code)?;
    let input_bytes=inputs(&args.inputs)?;
    let exact_inputs:BTreeMap<_,_>=request.inputs.iter().map(|i|(i.name.clone(),i.hash.clone())).collect();
    if contracts::hash_bytes(&code)!=request.code.hash || exact_inputs.len()!=request.inputs.len()
        || exact_inputs!=input_bytes.iter().map(|(n,b)|(n.clone(),contracts::hash_bytes(b))).collect() {
        return Err("prepared request code or inputs changed".into());
    }
    // Existing durable identity is read before CAS mutations/POST. A
    // previously accepted attempt is never dispatched again.
    let existing=existing_record(&args.state_dir,&request)?;
    let mut run=if let Some(record)=existing {
        if !args.reconcile_only {return Err(format!("request already accepted as {}; use --reconcile-only, no new POST",record.run_id).into());}
        if record.accepted_at<args.reservation_started_at || record.accepted_at>args.complete_before {return Err("durable run acceptance lies outside the consumed reservation".into());}
        fetch_run(&args.socket,record.run_id).await?
    } else {
        if args.reconcile_only {return Err("no existing durable request to reconcile; no dispatch".into());}
        ensure_window(&args,&request)?;
        check_ready(&args.socket,&loaded.file.key_id).await?;
        record_intent(&args,&request,&expected)?;
        let artifacts=ArtifactStore::open(args.state_dir.join("artifacts"),mebibytes(args.artifact_budget_mb)?)?;
        let upload=format!("upload:{}",request.request_id);
        let mut pinned=Vec::new();
        for data in std::iter::once(code.as_slice()).chain(input_bytes.values().map(Vec::as_slice)) {
            match artifacts.put_pinned(data,&upload) {
                Ok(stored)=>{if !pinned.contains(&stored.hash){pinned.push(stored.hash);}},
                Err(error)=>{
                    release_pins(&artifacts,&pinned,&upload)?;
                    return Err(error.into());
                }
            }
        }
        if let Err(error)=async { before_post().await?; check_ready(&args.socket,&loaded.file.key_id).await?; ensure_window(&args,&request) }.await {
            release_pins(&artifacts,&pinned,&upload)?;
            return Err(error);
        }
        // No replacement request, automatic retry, or rollback on uncertain
        // response. The service owns transfer to the durable request pin.
        let response=tokio::time::timeout(Duration::from_secs(10),uds_client::request_same_uid(&args.socket,Method::POST,
            "/api/v2/exec/runs",bytes)).await
            .map_err(|_|format!("dispatch outcome uncertain for {}; reconcile durable request; do not POST again",request.request_id))?
            .map_err(|e|format!("dispatch outcome uncertain for {}: {e}; reconcile durable request; do not POST again",request.request_id))?;
        if !matches!(response.status,200|202) {
            return Err(format!("existing REST refused HTTP {}; reconcile durable request {} before any further action",response.status,request.request_id).into());
        }
        serde_json::from_slice::<RunResponse>(&response.body)?
    };
    check_response(&run,&request,&expected)?;
    let run_id=run.run_id;
    eprintln!("Exact request {} accepted as run {}; only reconcile this durable identity",request.request_id,run_id);
    let deadline=tokio::time::Instant::now()+Duration::from_millis(request.limits.wall_clock_ms.saturating_add(10_000));
    while !run.state.is_terminal() {
        if run.state==RunState::GrantPending {return Err("unexpected grant pending; selected authority does not authorize escalation".into());}
        if tokio::time::Instant::now()>=deadline {return Err(format!("run {run_id} unsettled; reconcile only").into());}
        tokio::time::sleep(Duration::from_millis(50)).await;
        run=tokio::time::timeout(Duration::from_secs(5),fetch_run(&args.socket,run_id)).await??;
        check_response(&run,&request,&expected)?;
        if run.run_id!=run_id {return Err("durable run identity changed".into());}
    }
    let receipt=run.receipt.as_ref().ok_or("terminal state has no signed receipt")?;
    if receipt.run_id!=run_id || receipt.state!=run.state {return Err("terminal receipt differs from durable run".into());}
    archive(&args,receipt,&request,&loaded).await?;
    println!("{}",serde_json::to_string_pretty(&run)?);
    Ok(if run.state==RunState::Succeeded {ExitCode::SUCCESS} else {ExitCode::from(1)})
}
