//! Selected integration uses the existing Exec contract verifier. This is
//! containment/settlement evidence, not a product scenario verdict.
use super::{fail, require, read_pin, confined, sha, PinnedFile};
use crate::{GateOutcome, GateReceipt, GateRun, Result};
use chrono::{DateTime, Duration, Utc};
use prometheus_exec_contracts::{
    self as exec, CapabilityManifest, CodeKind, Digest, EvidenceClass, EvidenceIdentity,
    ExecutionBackend, ExecutionReceipt, ExecutionTier, RequestedTier, RunState, RuntimeKind,
    SignedExecRequest, VerificationKey,
};
use serde::Deserialize;
use std::{collections::{BTreeMap, BTreeSet}, fs, path::{Path, PathBuf}};

const PROFILE: &str = "exec-1.7.0/macos-aarch64/python3/seatbelt";
const RUNNER_SHA: &str = "6ca67fd7b411ec30945ee903228a2b0501bca78c7b442bcef4196b8e97310376";
const TOOLCHAIN_SHA: &str = "sha256:1025ceb64342aa35b18fa36ab7e0d4d3732566a9e172fd9eff9c4d82d9c617ee";
const START_PREFIX: &str = "exec-request:";
const FINISH_PREFIX: &str = "verified-exec/v1:";
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ExecBinding {
    profile: String,
    runner_binary: PathBuf,
    pub(super) request: PinnedFile,
    pub(super) request_hash: Digest,
    requester: EvidenceIdentity,
    runner: EvidenceIdentity,
    code: PinnedFile,
    inputs: BTreeMap<String, PinnedFile>,
    toolchain_hash: Digest,
    pub(super) receipt_path: String,
    pub(super) artifact_root: String,
    required_artifacts: BTreeSet<String>,
}
fn checked<T>(value: exec::Result<T>) -> Result<T> {
    value.map_err(|error| fail(format!("Exec verification: {error}")))
}
fn key(identity: &EvidenceIdentity) -> Result<VerificationKey> {
    let key = checked(VerificationKey::from_base64url(identity.sig_alg, &identity.public_key))?;
    require(key.key_id() == identity.key_id, "Exec identity does not match its signed public key")?;
    Ok(key)
}
fn expiry(request: &SignedExecRequest) -> Result<DateTime<Utc>> {
    let seconds = i64::try_from(request.validity_window_secs).map_err(|_| fail("Exec validity overflow"))?;
    request.issued_at.checked_add_signed(Duration::try_seconds(seconds).ok_or_else(|| fail("Exec validity overflow"))?)
        .ok_or_else(|| fail("Exec validity overflow"))
}
fn hex(d: &Digest) -> &str { d.as_str().strip_prefix("sha256:").expect("validated Digest") }
fn valid_hex(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub(super) fn request_token(hash: &str) -> String {
    sha(format!("kbd.selected-exec-request/v1\0{hash}").as_bytes())
}
fn receipt_token(hash: &str) -> String {
    sha(format!("kbd.selected-exec-receipt/v1\0{hash}").as_bytes())
}
pub(super) fn executable(binding: &ExecBinding) -> String {
    format!("{START_PREFIX}{}", hex(&binding.request_hash))
}
pub(crate) fn start_token(gate: &GateRun) -> Result<String> {
    let hash = gate.executable.strip_prefix(START_PREFIX).filter(|s| valid_hex(s))
        .ok_or_else(|| fail("selected start lacks its exact Exec request identity"))?;
    Ok(request_token(hash))
}
impl ExecBinding {
    pub(super) fn validate(&self, root: &Path, phase: &str, start: &str, candidate: &PinnedFile,
        guard: &PinnedFile, sources: &BTreeMap<String,String>, argv: &[String], cwd: &str,
        max_seconds: u64) -> Result<SignedExecRequest> {
        require(self.profile == PROFILE && cfg!(all(target_os = "macos", target_arch = "aarch64")),
            "unsupported selected Exec profile; only the qualified local macOS arm64 Python transport is enabled")?;
        require(self.runner_binary.is_absolute() && self.runner_binary == managed_home()?.join(".local/bin/prometheus-exec")
            && self.toolchain_hash.as_str() == TOOLCHAIN_SHA, "unsupported managed installed runner or qualified toolchain")?;
        let request: SignedExecRequest = serde_json::from_slice(&read_pin(root, &self.request.path, &self.request.sha256)?)
            .map_err(|error| fail(format!("invalid pinned Exec request: {error}")))?;
        checked(request.validate())?;
        checked(exec::verify_request_signature(&request, &key(&self.requester)?))?;
        key(&self.runner)?;
        // Exec 1.7.0 verifies requests using its own receipt identity. Independent
        // verification means a separate verifier, not an invented second key.
        require(self.requester == self.runner, "Exec 1.7.0 requires the enrolled request/receipt identity")?;
        // The installed persistent identity is checked independently by the
        // admitting host at handoff and reservation, not an identity chosen
        // solely by the candidate. The dispatch client rechecks its live peer.
        require(checked(request.request_hash())? == self.request_hash
            && request.provenance.phase.as_deref() == Some(phase)
            && request.provenance.change.as_deref() == Some(start),
            "Exec request is not bound to the exact selected phase/start")?;
        require(request.tier == RequestedTier::P && request.code.kind == CodeKind::File
            && request.code.runtime == RuntimeKind::Python3 && request.targets.is_empty()
            && request.capabilities == CapabilityManifest::default()
            && request.limits.wall_clock_ms <= max_seconds.saturating_mul(1000)
            && request.code.toolchain_pin.as_ref() == Some(&self.toolchain_hash),
            "unsupported or excessive selected Exec request capabilities/runtime/budget")?;
        // Exec Tier P file requests do not carry arbitrary argv. Do not silently
        // translate a Node/native/Cargo command or omit arguments from it.
        require(self.code.path == guard.path && self.code.sha256 == guard.sha256
            && sources.get(&self.code.path) == Some(&self.code.sha256),
            "Exec code must be the exact selected source guard")?;
        let working_dir = confined(root,cwd)?;
        let code_path = confined(root,&self.code.path)?;
        let exact_code = |s: &str| working_dir.join(s).canonicalize().is_ok_and(|p| p == code_path);
        require(matches!(argv, [file] if exact_code(file))
            || matches!(argv, [python,file] if Path::new(python).file_name().and_then(|p| p.to_str()) == Some("python3") && exact_code(file)),
            "selected command is not exactly the supported Exec Python file invocation")?;
        require(hex(&request.code.hash) == self.code.sha256
            && exec::hash_bytes(&read_pin(root,&self.code.path,&self.code.sha256)?) == request.code.hash,
            "Exec code hash differs from selected source")?;
        let candidate_input = self.inputs.get("candidate-manifest.json").ok_or_else(|| fail("Exec candidate manifest input absent"))?;
        require(candidate_input.path == candidate.path && candidate_input.sha256 == candidate.sha256,
            "Exec request does not carry the selected candidate manifest")?;
        require(self.inputs.len() == request.inputs.len(), "Exec input roster differs from signed selection")?;
        let mut names = BTreeSet::new();
        for input in &request.inputs {
            let pin = self.inputs.get(&input.name).ok_or_else(|| fail("unbound Exec input"))?;
            require(names.insert(&input.name)
                && ((pin.path == candidate.path && pin.sha256 == candidate.sha256)
                    || sources.get(&pin.path) == Some(&pin.sha256))
                && exec::hash_bytes(&read_pin(root,&pin.path,&pin.sha256)?) == input.hash,
                "Exec input is unknown, duplicate, or stale")?;
        }
        require(!self.required_artifacts.is_empty(), "selected Exec start requires independently verifiable artifacts")?;
        for path in &self.required_artifacts { checked(exec::validate_artifact_path(path))?; }
        // Do not accept a receipt destination outside the project. The
        // reservation requires a fresh file, while reconciliation requires it.
        let path = Path::new(&self.receipt_path);
        let parent = path.parent().ok_or_else(|| fail("invalid Exec receipt path"))?;
        confined(root,&parent.to_string_lossy())?;
        require(path.file_name().is_some() && !path.is_absolute()
            && path.components().all(|c| matches!(c,std::path::Component::Normal(_) | std::path::Component::CurDir)),
            "invalid Exec receipt destination")?;
        confined(root,&self.artifact_root)?;
        Ok(request)
    }
    pub(super) fn admit_start(&self, root: &Path, request: &SignedExecRequest, now: DateTime<Utc>,
        scope_expiry: DateTime<Utc>, max_seconds: u64) -> Result<()> {
        let maximum = i64::try_from(max_seconds).map_err(|_| fail("Exec reservation duration overflow"))?;
        let end = now.checked_add_signed(Duration::try_seconds(maximum).ok_or_else(|| fail("Exec reservation duration overflow"))?)
            .ok_or_else(|| fail("Exec reservation duration overflow"))?;
        require(request.issued_at <= now && end <= expiry(request)? && end <= scope_expiry,
            "Exec request or signed scope cannot cover the entire reservation")?;
        require(!root.join(&self.receipt_path).try_exists()? && !root.join(&self.artifact_root).join(".selected-exec-dispatch-intent.json").try_exists()?, "selected Exec receipt or dispatch-intent destination already exists")?;
        let runs = managed_home()?.join(".prometheus/exec/service/ledger/runs");
        require(runs.is_dir() && !runs.join(format!("{}.json",request.request_id)).try_exists()?, "selected Exec durable request already accepted or ledger unavailable")?;
        Ok(())
    }
    pub(super) fn completion(&self, root: &Path, gate: &GateRun, request: &SignedExecRequest,
        candidate: &PinnedFile, scope_expiry: DateTime<Utc>, max_seconds: u64) -> Result<GateReceipt> {
        require(gate.executable == executable(self), "Exec request differs from the consumed gate")?;
        let receipt: ExecutionReceipt = serde_json::from_slice(&fs::read(confined(root,&self.receipt_path)?)?)
            .map_err(|error| fail(format!("invalid terminal Exec receipt: {error}")))?;
        let verified = exec::verify_receipt(&receipt,&key(&self.runner)?,Some(request),
            Some(&confined(root,&self.artifact_root)?));
        require(verified.valid, &format!("Exec receipt/artifact verification failed: {:?}",verified.failures))?;
        let inputs: BTreeMap<_,_> = request.inputs.iter().map(|i| (i.name.clone(),i.hash.clone())).collect();
        let env: BTreeMap<String,String> = BTreeMap::new();
        require(receipt.request_hash == self.request_hash && receipt.code_hash == request.code.hash
            && receipt.input_set_hash == checked(exec::hash_serializable(&inputs))?
            && receipt.env_hash == checked(exec::hash_serializable(&env))?
            && receipt.toolchain_hash.as_ref() == Some(&self.toolchain_hash)
            && receipt.executing_device.platform == "macos-aarch64"
            && receipt.tier == ExecutionTier::P && receipt.evidence_class == EvidenceClass::Attested
            && receipt.backend == ExecutionBackend::Seatbelt && receipt.grants.is_empty(),
            "Exec receipt has wrong candidate/request/input/environment/runner/profile")?;
        let elapsed = receipt.finished_at.signed_duration_since(gate.started_at).num_milliseconds();
        require(receipt.started_at >= gate.started_at && receipt.finished_at >= receipt.started_at
            && receipt.finished_at <= Utc::now() && receipt.finished_at <= scope_expiry
            && receipt.finished_at <= expiry(request)? && elapsed >= 0
            && (elapsed as u64) <= max_seconds.saturating_mul(1000)
            && receipt.usage.wall_clock_ms <= request.limits.wall_clock_ms,
            "Exec receipt is stale or outside the consumed reservation")?;
        let succeeded = receipt.state == RunState::Succeeded && receipt.exit.status == 0
            && receipt.exit.signal_or_trap.is_none();
        if succeeded {
            require(self.required_artifacts.iter().all(|path| receipt.outputs.artifacts.iter().any(|a| &a.path == path)),
                "successful Exec receipt lacks a required artifact")?;
        }
        let receipt_hash = verified.receipt_hash.ok_or_else(|| fail("verified receipt identity missing"))?;
        Ok(GateReceipt { gate_id: gate.id.clone(), kind: gate.kind, scope: gate.scope.clone(),
            phase_id: gate.phase_id.clone(), source_revision: gate.source_revision,
            outcome: if succeeded { GateOutcome::Passed } else { GateOutcome::Failed },
            exit_code: Some(receipt.exit.status), duration_ms: receipt.usage.wall_clock_ms,
            finished_at: receipt.finished_at,
            summary: format!("{FINISH_PREFIX}{}:{}:{}",hex(&self.request_hash),hex(&receipt_hash),candidate.sha256) })
    }
}
fn completion_parts(receipt: &GateReceipt) -> Result<(&str,&str,&str)> {
    let mut parts = receipt.summary.strip_prefix(FINISH_PREFIX).ok_or_else(|| fail("selected finish lacks verified Exec attribution"))?.split(':');
    let request = parts.next().filter(|s| valid_hex(s)).ok_or_else(|| fail("invalid Exec request attribution"))?;
    let observed = parts.next().filter(|s| valid_hex(s)).ok_or_else(|| fail("invalid Exec receipt attribution"))?;
    let candidate = parts.next().filter(|s| valid_hex(s)).ok_or_else(|| fail("invalid candidate attribution"))?;
    require(parts.next().is_none(), "invalid selected receipt attribution")?;
    Ok((request,observed,candidate))
}
pub(crate) fn replay_finish(state: &crate::KbdStateV2, gate: &GateRun, receipt: &GateReceipt) -> Result<String> {
    let (request,observed,_) = completion_parts(receipt)?;
    require(gate.executable == format!("{START_PREFIX}{request}")
        && state.phase_closure.consumed_manifests.contains(&request_token(request))
        && matches!(receipt.outcome,GateOutcome::Passed | GateOutcome::Failed),
        "selected finish does not bind the consumed Exec request")?;
    Ok(receipt_token(observed))
}
pub(super) fn settled(state: &crate::KbdStateV2, receipt: &GateReceipt, binding: &ExecBinding, candidate: &PinnedFile) -> bool {
    completion_parts(receipt).is_ok_and(|(request,observed,candidate_hash)|
        request == hex(&binding.request_hash) && candidate_hash == candidate.sha256
        && state.phase_closure.consumed_manifests.contains(&request_token(request))
        && state.phase_closure.consumed_manifests.contains(&receipt_token(observed)))
}

fn managed_home() -> Result<PathBuf> {
    dirs_next::home_dir().ok_or_else(|| fail("managed local Exec home unavailable"))
}

impl ExecBinding {
    pub(super) fn same_runner(&self, other:&Self)->bool {
        self.runner_binary==other.runner_binary && self.runner==other.runner && self.profile==other.profile
    }
    /// Existing read-only installed doctor, owned child with a finite deadline.
    /// The project does not choose an enrollment key or a different daemon.
    pub(super) fn verify_installed_identity(&self) -> Result<()> {
        use std::{io::Read,process::{Command,Stdio},thread,time::{Duration as StdDuration,Instant}};
        let home=managed_home()?;
        let binary=home.join(".local/bin/prometheus-exec");
        require(self.runner_binary==binary && sha(&fs::read(&binary)?)==RUNNER_SHA
            && exec::hash_bytes(&fs::read("/usr/bin/python3")?).as_str()==TOOLCHAIN_SHA,
            "installed qualified Exec binary or Python changed")?;
        let state=home.join(".prometheus/exec");
        let socket=home.join(".prometheus/run/prometheus-exec.sock");
        let mut child=Command::new(&binary).arg("doctor").arg("--socket").arg(&socket)
            .arg("--state-dir").arg(&state).arg("--identity").arg(state.join("identity.json"))
            .arg("--plugin-root").arg(home.join(".prometheus/plugins/prometheus-skill-pack"))
            .args(["--format","json"]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::null()).spawn()?;
        let stdout=child.stdout.take().ok_or_else(||fail("installed doctor output missing"))?;
        let reader=thread::spawn(move || { let mut bytes=Vec::new(); stdout.take(1024*1024+1).read_to_end(&mut bytes).map(|_|bytes) });
        let deadline=Instant::now()+StdDuration::from_secs(15);
        let status=loop {
            if let Some(status)=child.try_wait()?{break status;}
            if Instant::now()>=deadline {
                let _=child.kill(); let _=child.wait(); let _=reader.join();
                return Err(fail("installed Exec doctor exceeded finite readiness deadline"));
            }
            thread::sleep(StdDuration::from_millis(25));
        };
        let bytes=reader.join().map_err(|_|fail("installed doctor read failed"))??;
        require(status.success()&&bytes.len()<=1024*1024,"installed Exec doctor refused readiness or identity")?;
        let report:serde_json::Value=serde_json::from_slice(&bytes).map_err(|_|fail("invalid installed doctor report"))?;
        let checks=report["checks"].as_array().ok_or_else(||fail("installed doctor checks absent"))?;
        let passed=|name:&str,detail:Option<&str>| checks.iter().filter(|check|check["name"]==name).count()==1
            && checks.iter().any(|check|check["name"]==name && check["status"]=="pass" && check["required"]==true
                && detail.is_none_or(|expected|check["detail"]==expected));
        require(report["version"]=="1.7.0" && report["healthy"]==true
            && report["excluded"].as_array().is_some_and(Vec::is_empty)
            && checks.iter().all(|c|c["required"]!=true||c["status"]=="pass")
            && passed("binary-identity",Some(&format!("prometheus-exec 1.7.0 sha256:{RUNNER_SHA}")))
            && passed("receipt-identity",Some(&format!("{} is internally consistent",self.runner.key_id)))
            && passed("socket-permissions",None) && passed("socket-peer-health",None)
            && passed("readiness",None) && passed("tier-p-backend",None)
            && passed("state-reconciliation",None) && passed("receipt-reconciliation",None)
            && passed("artifact-cas",None),
            "managed installed signer/profile/ledger/readiness does not match selected authority")
    }

    pub(super) fn dispatch(&self,root:&Path,gate:&GateRun,request:&SignedExecRequest,scope_expiry:DateTime<Utc>,max_seconds:u64,reconcile_only:bool)
        -> Result<super::SelectedDispatch> {
        require(gate.executable==executable(self),"dispatch request differs from consumed gate")?;
        let max=i64::try_from(max_seconds).map_err(|_|fail("reservation duration overflow"))?;
        let end=gate.started_at.checked_add_signed(Duration::try_seconds(max).ok_or_else(||fail("reservation duration overflow"))?)
            .ok_or_else(||fail("reservation duration overflow"))?.min(scope_expiry).min(expiry(request)?);
        if !reconcile_only {
            let wall=i64::try_from(request.limits.wall_clock_ms).map_err(|_|fail("request wall budget overflow"))?;
            require(Utc::now().checked_add_signed(Duration::try_milliseconds(wall).ok_or_else(||fail("request wall budget overflow"))?)
                .is_some_and(|finish|finish<=end),"selected reservation lacks remaining runtime budget")?;
        }
        let home=managed_home()?;
        let mut inputs=Vec::new();
        for (name,pin) in &self.inputs {
            let file=confined(root,&pin.path)?;
            inputs.push(format!("{name}={}",file.to_str().ok_or_else(||fail("non-UTF8 selected input path"))?));
        }
        let receipt=root.join(&self.receipt_path);
        Ok(super::SelectedDispatch{socket:home.join(".prometheus/run/prometheus-exec.sock"),
            state_dir:home.join(".prometheus/exec"),identity:home.join(".prometheus/exec/identity.json"),
            request:confined(root,&self.request.path)?,expected_request_hash:self.request_hash.to_string(),
            code:confined(root,&self.code.path)?,inputs,receipt_out:receipt,artifact_root:confined(root,&self.artifact_root)?,
            reservation_started_at:gate.started_at,complete_before:end,expected_runner_key:self.runner.key_id.clone(),reconcile_only})
    }
}
