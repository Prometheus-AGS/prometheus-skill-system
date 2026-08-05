use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

#[cfg(unix)]
use std::time::{Duration, Instant as StdInstant};

use chrono::Utc;
use ed25519_dalek::SigningKey;
#[cfg(unix)]
use prometheus_exec_contracts::verify_request_signature;
use prometheus_exec_contracts::{
    hash_bytes, hash_serializable, sign_receipt_ed25519, EvidenceClass, ExecutingDevice,
    ExecutionBackend, ExecutionExit, ExecutionFailure, ExecutionFailureKind, ExecutionOutputs,
    ExecutionReceipt, ExecutionTier, RequestedTier, ResourceUsage, RunState, RuntimeKind,
    SignatureAlgorithm, VerificationKey, SCHEMA_VERSION,
};
use prometheus_exec_core::ArtifactStore;
#[cfg(unix)]
use prometheus_exec_core::{
    BackendExecution, BaselinePolicy, CedarTighteningPolicy, Ed25519ReceiptSigner, ExecutionJob,
    ExecutionPort, PolicyEvaluator, PolicyOutcome, ReceiptAssembler, ValidatedExecutionJob,
};
#[cfg(unix)]
use prometheus_exec_service::RunEventData;
#[cfg(unix)]
use prometheus_exec_service::UdsSidecar;
use prometheus_exec_service::{ExecutionService, ReadinessStatus, RunRecord, SidecarState};
#[cfg(target_os = "macos")]
use prometheus_exec_tier_p::{SeatbeltConfig, SeatbeltExecutor};
#[cfg(unix)]
use prometheus_exec_tier_w::{
    compiled_backend, BackendProfile, ComponentAuthorizer, EngineProfile, TierWEngine, TierWError,
    TierWExecutionPort, TierWPortError,
};

use crate::identity;

#[cfg(unix)]
const RUNNER_POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Debug)]
pub struct DaemonConfig {
    pub socket: PathBuf,
    pub state_dir: PathBuf,
    pub identity: PathBuf,
    pub plugin_root: PathBuf,
    pub artifact_budget_bytes: u64,
}

#[cfg(unix)]
pub async fn run(config: DaemonConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let sidecar = UdsSidecar::start(&config.socket).await?;
    let state = sidecar.state().clone();
    state
        .set_readiness(
            "receipt-identity",
            ReadinessStatus::Initializing,
            "receipt signing identity is loading",
        )
        .await;
    state
        .set_readiness(
            "tier-p-backend",
            ReadinessStatus::Initializing,
            "Tier P sandbox is being detected",
        )
        .await;
    state
        .set_readiness(
            "tier-w-backend",
            ReadinessStatus::Initializing,
            "Tier W Wasmtime backend is being probed",
        )
        .await;
    state
        .set_readiness(
            "tier-w-trust",
            ReadinessStatus::Initializing,
            "Tier W component trust is being verified",
        )
        .await;
    state
        .set_readiness(
            "execution-runner",
            ReadinessStatus::Initializing,
            "durable execution runner is starting",
        )
        .await;

    let initialized = initialize(&config, &state).await;
    let (service, artifacts, signing_key, verification_key) = match initialized {
        Ok(initialized) => initialized,
        Err(error) => {
            state
                .set_readiness(
                    "daemon-initialization",
                    ReadinessStatus::Failed,
                    error.to_string(),
                )
                .await;
            sidecar.shutdown().await?;
            return Err(error);
        }
    };

    let recovery_key = signing_key.clone();
    let recovery_artifacts = artifacts.clone();
    service.reconcile(&verification_key, move |record| {
        synthetic_receipt(
            record,
            RunState::Interrupted,
            "daemon restarted after the durable spawn boundary",
            ExecutionFailureKind::Interrupted,
            &recovery_key,
            &recovery_artifacts,
        )
        .map_err(|error| error.to_string())
    })?;
    retain_reconciled_receipts(&service, &artifacts)?;
    if let Err(error) = artifacts.garbage_collect() {
        eprintln!("prometheus-exec: startup artifact GC failed: {error}");
    }
    state.install(service.clone(), artifacts.clone()).await;

    state
        .set_readiness(
            "receipt-identity",
            ReadinessStatus::Ready,
            format!("receipt signer {} is ready", verification_key.key_id()),
        )
        .await;

    let authorizer = ComponentAuthorizer::estate(&config.plugin_root);
    match authorizer.inspect() {
        Ok(inspection) => {
            state
                .set_readiness(
                    "tier-w-trust",
                    ReadinessStatus::Ready,
                    format!(
                        "active signed generation {} authorizes {} component(s)",
                        inspection.generation_id.as_deref().unwrap_or("exact-pins"),
                        inspection.component_count
                    ),
                )
                .await;
        }
        Err(error) => {
            state
                .set_readiness("tier-w-trust", ReadinessStatus::Failed, error.to_string())
                .await;
        }
    }
    let tier_w = match TierWEngine::new(EngineProfile::for_current_target()) {
        Ok(engine) => {
            state
                .set_readiness(
                    "tier-w-backend",
                    ReadinessStatus::Ready,
                    format!(
                        "Wasmtime 46 {} backend is available for {}",
                        engine.profile().backend.name(),
                        engine.profile().target.name()
                    ),
                )
                .await;
            Some(TierWExecutionPort::new(engine, authorizer.clone()))
        }
        Err(error) => {
            state
                .set_readiness("tier-w-backend", ReadinessStatus::Failed, error.to_string())
                .await;
            None
        }
    };

    #[cfg(target_os = "macos")]
    let tier_p = match SeatbeltConfig::detect() {
        Ok(detected) => {
            let executor =
                SeatbeltExecutor::new(detected.with_work_root(config.state_dir.join("work")));
            state
                .set_readiness(
                    "tier-p-backend",
                    ReadinessStatus::Ready,
                    "macOS Seatbelt Tier P backend is available",
                )
                .await;
            Some(executor)
        }
        Err(error) => {
            state
                .set_readiness("tier-p-backend", ReadinessStatus::Failed, error.to_string())
                .await;
            None
        }
    };

    #[cfg(not(target_os = "macos"))]
    {
        state
            .set_readiness(
                "tier-p-backend",
                ReadinessStatus::Failed,
                "Tier P execution is not runtime-certified on this platform",
            )
            .await;
    }

    #[cfg(target_os = "macos")]
    let backends = RunnerBackends {
        tier_w,
        tier_w_trust: authorizer,
        tier_p,
    };
    #[cfg(not(target_os = "macos"))]
    let backends = RunnerBackends {
        tier_w,
        tier_w_trust: authorizer,
    };
    let runner = tokio::spawn(runner_loop(
        service.clone(),
        artifacts.clone(),
        signing_key,
        verification_key,
        backends,
        state.clone(),
    ));

    tokio::signal::ctrl_c().await?;
    runner.abort();
    let _ = runner.await;
    sidecar.shutdown().await?;
    Ok(())
}

#[cfg(not(unix))]
pub async fn run(_config: DaemonConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err("daemon mode requires Unix-domain socket support".into())
}

type Initialized = (
    Arc<ExecutionService>,
    Arc<ArtifactStore>,
    SigningKey,
    VerificationKey,
);

async fn initialize(
    config: &DaemonConfig,
    _state: &SidecarState,
) -> Result<Initialized, Box<dyn std::error::Error + Send + Sync>> {
    let identity = identity::load(&config.identity)?;
    let service = Arc::new(ExecutionService::open(config.state_dir.join("service"))?);
    let artifacts = Arc::new(ArtifactStore::open(
        config.state_dir.join("artifacts"),
        config.artifact_budget_bytes,
    )?);
    Ok((
        service,
        artifacts,
        identity.signing_key,
        identity.verification_key,
    ))
}

#[cfg(unix)]
struct RunnerBackends {
    tier_w: Option<TierWExecutionPort>,
    tier_w_trust: ComponentAuthorizer,
    #[cfg(target_os = "macos")]
    tier_p: Option<SeatbeltExecutor>,
}

#[cfg(unix)]
#[derive(Debug)]
struct BackendFailure {
    kind: ExecutionFailureKind,
    message: String,
}

#[cfg(unix)]
impl RunnerBackends {
    async fn refresh_tier_w_trust(&self, state: &SidecarState) {
        match self.tier_w_trust.inspect() {
            Ok(inspection) => {
                state
                    .set_readiness(
                        "tier-w-trust",
                        ReadinessStatus::Ready,
                        format!(
                            "active signed generation {} authorizes {} component(s)",
                            inspection.generation_id.as_deref().unwrap_or("exact-pins"),
                            inspection.component_count
                        ),
                    )
                    .await;
            }
            Err(error) => {
                state
                    .set_readiness("tier-w-trust", ReadinessStatus::Failed, error.to_string())
                    .await;
            }
        }
    }

    async fn execute(
        &self,
        job: &ValidatedExecutionJob,
    ) -> Result<BackendExecution, BackendFailure> {
        match selected_tier(job.request().tier, job.request().code.runtime) {
            ExecutionTier::W => {
                let Some(executor) = &self.tier_w else {
                    return Err(BackendFailure {
                        kind: ExecutionFailureKind::BackendUnavailable,
                        message: "Tier W backend or component trust is unavailable".into(),
                    });
                };
                executor.execute(job).await.map_err(map_tier_w_failure)
            }
            ExecutionTier::P => {
                #[cfg(target_os = "macos")]
                {
                    let Some(executor) = &self.tier_p else {
                        return Err(BackendFailure {
                            kind: ExecutionFailureKind::BackendUnavailable,
                            message: "Tier P Seatbelt backend is unavailable".into(),
                        });
                    };
                    executor.execute(job).await.map_err(|error| BackendFailure {
                        kind: ExecutionFailureKind::Trap,
                        message: format!("Seatbelt execution failed: {error}"),
                    })
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Err(BackendFailure {
                        kind: ExecutionFailureKind::BackendUnavailable,
                        message: "Tier P execution is not runtime-certified on this platform"
                            .into(),
                    })
                }
            }
        }
    }
}

#[cfg(unix)]
fn selected_tier(requested: RequestedTier, runtime: RuntimeKind) -> ExecutionTier {
    if requested == RequestedTier::W || runtime == RuntimeKind::WasmComponent {
        ExecutionTier::W
    } else {
        ExecutionTier::P
    }
}

#[cfg(unix)]
fn map_tier_w_failure(error: TierWPortError) -> BackendFailure {
    let kind = match &error {
        TierWPortError::Adapter(TierWError::ComponentUnauthorized(_))
        | TierWPortError::AuthorizationMismatch
        | TierWPortError::InvalidRuntime => ExecutionFailureKind::ComponentUnauthorized,
        TierWPortError::Adapter(TierWError::CapabilityDenied(_))
        | TierWPortError::Adapter(TierWError::InvalidCapabilityGrant(_))
        | TierWPortError::UnsupportedCapability(_) => ExecutionFailureKind::CapabilityDenied,
        TierWPortError::Adapter(TierWError::BackendUnavailable { .. }) => {
            ExecutionFailureKind::BackendUnavailable
        }
        _ => ExecutionFailureKind::Trap,
    };
    BackendFailure {
        kind,
        message: error.to_string(),
    }
}

#[cfg(unix)]
async fn runner_loop(
    service: Arc<ExecutionService>,
    artifacts: Arc<ArtifactStore>,
    signing_key: SigningKey,
    verification_key: VerificationKey,
    backends: RunnerBackends,
    state: SidecarState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let policy = CedarTighteningPolicy::default();
    let mut last_trust_check = StdInstant::now();
    state
        .set_readiness(
            "execution-runner",
            ReadinessStatus::Ready,
            "durable Tier P/Tier W execution dispatcher is running",
        )
        .await;
    loop {
        if last_trust_check.elapsed() >= Duration::from_secs(1) {
            backends.refresh_tier_w_trust(&state).await;
            last_trust_check = StdInstant::now();
        }
        // New requests are retained by the sidecar HTTP handler before the
        // service accepts them. This loop therefore never needs a racy second
        // pin pass and can process each durable record independently.
        for record in service.ledger().records()? {
            if record.state != RunState::Queued {
                continue;
            }
            if let Err(error) = process_queued_run(
                &service,
                &artifacts,
                &signing_key,
                &verification_key,
                &backends,
                &policy,
                record,
            )
            .await
            {
                state
                    .set_readiness(
                        "execution-runner",
                        ReadinessStatus::Failed,
                        format!("execution runner stopped: {error}"),
                    )
                    .await;
                return Err(error);
            }
        }
        tokio::time::sleep(RUNNER_POLL_INTERVAL).await;
    }
}

#[cfg(unix)]
async fn process_queued_run(
    service: &ExecutionService,
    artifacts: &ArtifactStore,
    signing_key: &SigningKey,
    verification_key: &VerificationKey,
    backends: &RunnerBackends,
    policy: &CedarTighteningPolicy,
    record: RunRecord,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let expires_at = record.request.issued_at
        + chrono::Duration::seconds(i64::try_from(record.request.validity_window_secs)?);
    if Utc::now() > expires_at {
        return reject_run(
            service,
            artifacts,
            signing_key,
            &record,
            "signed request validity window expired",
        );
    }
    if let Err(error) = verify_request_signature(&record.request, verification_key) {
        return reject_run(
            service,
            artifacts,
            signing_key,
            &record,
            &format!("request signature verification failed: {error}"),
        );
    }
    match BaselinePolicy.evaluate(&record.request) {
        PolicyOutcome::AutoApproved => {}
        PolicyOutcome::GrantRequired { reasons } => {
            service.mark_grant_pending(
                record.request_id,
                &record.request_hash,
                "grant.policy",
                serde_json::to_string(&reasons)?,
            )?;
            return Ok(());
        }
        PolicyOutcome::Denied { reasons } => {
            return reject_run(
                service,
                artifacts,
                signing_key,
                &record,
                &format!("baseline policy denied request: {reasons:?}"),
            )
        }
    }
    match policy.evaluate(&record.request) {
        PolicyOutcome::AutoApproved => {}
        PolicyOutcome::GrantRequired { reasons } => {
            service.mark_grant_pending(
                record.request_id,
                &record.request_hash,
                "grant.cedar",
                serde_json::to_string(&reasons)?,
            )?;
            return Ok(());
        }
        PolicyOutcome::Denied { reasons } => {
            return reject_run(
                service,
                artifacts,
                signing_key,
                &record,
                &format!("Cedar policy denied request: {reasons:?}"),
            )
        }
    }

    let code = match artifacts.get(&record.request.code.hash) {
        Ok(code) => code,
        Err(error) => {
            return reject_run(
                service,
                artifacts,
                signing_key,
                &record,
                &format!("code artifact is unavailable: {error}"),
            )
        }
    };
    let mut inputs = BTreeMap::new();
    for input in &record.request.inputs {
        match artifacts.get(&input.hash) {
            Ok(bytes) => {
                inputs.insert(input.name.clone(), bytes);
            }
            Err(error) => {
                return reject_run(
                    service,
                    artifacts,
                    signing_key,
                    &record,
                    &format!("input {} is unavailable: {error}", input.name),
                )
            }
        }
    }
    let job = match (ExecutionJob {
        request: record.request.clone(),
        code,
        inputs,
        grants: vec![],
    })
    .validate()
    {
        Ok(job) => job,
        Err(error) => {
            return reject_run(
                service,
                artifacts,
                signing_key,
                &record,
                &format!("execution job is invalid: {error}"),
            )
        }
    };
    let claim = service.claim_for_execution(record.request_id, &record.request_hash)?;
    if !claim.claimed {
        return Ok(());
    }

    let execution = match backends.execute(&job).await {
        Ok(execution) => execution,
        Err(failure) => {
            let receipt = synthetic_receipt(
                &claim.record,
                RunState::Failed,
                &failure.message,
                failure.kind,
                signing_key,
                artifacts,
            )?;
            commit_terminal_with_retention(
                service,
                artifacts,
                &claim.record,
                receipt,
                verification_key,
            )?;
            return Ok(());
        }
    };

    emit_stream_events(
        service,
        claim.record.run_id,
        "stdout",
        &execution.stdout,
        execution.finished_at,
    )?;
    emit_stream_events(
        service,
        claim.record.run_id,
        "stderr",
        &execution.stderr,
        execution.finished_at,
    )?;
    let receipt = ReceiptAssembler::new(Ed25519ReceiptSigner::new(signing_key.clone()))
        .assemble_for_run(claim.record.run_id, &job, execution.clone())?;
    artifacts.persist_and_retain_for_receipt(&claim.record.request, &receipt, &execution)?;
    commit_terminal_with_retention(service, artifacts, &claim.record, receipt, verification_key)?;
    Ok(())
}

#[cfg(unix)]
fn emit_stream_events(
    service: &ExecutionService,
    run_id: uuid::Uuid,
    stream: &str,
    bytes: &[u8],
    occurred_at: chrono::DateTime<Utc>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (index, chunk) in bytes.chunks(16 * 1024).enumerate() {
        let data = match stream {
            "stdout" => RunEventData::Stdout {
                chunk: String::from_utf8_lossy(chunk).into_owned(),
            },
            _ => RunEventData::Stderr {
                chunk: String::from_utf8_lossy(chunk).into_owned(),
            },
        };
        service.append_runtime_event(
            run_id,
            format!("{stream}.{}", index + 1),
            occurred_at,
            data,
        )?;
    }
    Ok(())
}

#[cfg(unix)]
fn reject_run(
    service: &ExecutionService,
    artifacts: &ArtifactStore,
    signing_key: &SigningKey,
    record: &RunRecord,
    reason: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let verification_key = VerificationKey::ed25519(signing_key.verifying_key().to_bytes());
    let receipt = synthetic_receipt(
        record,
        RunState::Rejected,
        reason,
        ExecutionFailureKind::CapabilityDenied,
        signing_key,
        artifacts,
    )?;
    commit_terminal_with_retention(service, artifacts, record, receipt, &verification_key)?;
    Ok(())
}

#[cfg(unix)]
fn commit_terminal_with_retention(
    service: &ExecutionService,
    artifacts: &ArtifactStore,
    record: &RunRecord,
    receipt: ExecutionReceipt,
    verification_key: &VerificationKey,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    artifacts.retain_for_receipt(&record.request, &receipt)?;
    if let Err(commit_error) = service.commit_terminal(
        record.request_id,
        &record.request_hash,
        receipt.clone(),
        verification_key,
    ) {
        if let Err(error) = artifacts.release_receipt(&record.request, &receipt) {
            eprintln!(
                "prometheus-exec: receipt-pin rollback failed for {}: {error}",
                record.run_id
            );
        }
        return Err(commit_error.into());
    }
    if let Err(error) = artifacts.release_request(&record.request) {
        eprintln!(
            "prometheus-exec: request-pin cleanup failed for {}: {error}",
            record.run_id
        );
    }
    if let Err(error) = artifacts.garbage_collect() {
        eprintln!(
            "prometheus-exec: post-commit artifact GC failed for {}: {error}",
            record.run_id
        );
    }
    Ok(())
}

#[cfg(unix)]
fn retain_reconciled_receipts(
    service: &ExecutionService,
    artifacts: &ArtifactStore,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for record in service.ledger().records()? {
        let result = if record.state.is_terminal() {
            match &record.terminal {
                Some(terminal) => artifacts
                    .retain_for_receipt(&record.request, &terminal.receipt)
                    .and_then(|()| artifacts.release_request(&record.request)),
                None => {
                    eprintln!(
                        "prometheus-exec: terminal record {} has no terminal receipt; preserving request ownership",
                        record.run_id
                    );
                    artifacts.retain_for_request(&record.request)
                }
            }
        } else {
            artifacts.retain_for_request(&record.request)
        };
        if let Err(error) = result {
            eprintln!(
                "prometheus-exec: artifact retention reconciliation failed for {}: {error}",
                record.run_id
            );
        }
    }
    Ok(())
}

#[cfg(unix)]
fn synthetic_receipt(
    record: &RunRecord,
    state: RunState,
    reason: &str,
    failure_kind: ExecutionFailureKind,
    signing_key: &SigningKey,
    artifacts: &ArtifactStore,
) -> Result<ExecutionReceipt, Box<dyn std::error::Error + Send + Sync>> {
    let stdout = artifacts.put(b"")?;
    let stderr = artifacts.put(reason.as_bytes())?;
    let input_hashes: BTreeMap<_, _> = record
        .request
        .inputs
        .iter()
        .map(|input| (input.name.clone(), input.hash.clone()))
        .collect();
    let now = Utc::now();
    let tier = selected_tier(record.request.tier, record.request.code.runtime);
    let tier_w = tier == ExecutionTier::W;
    let mut receipt = ExecutionReceipt {
        schema_version: SCHEMA_VERSION.into(),
        run_id: record.run_id,
        request_hash: record.request_hash.clone(),
        state,
        evidence_class: if tier_w {
            EvidenceClass::Verified
        } else {
            EvidenceClass::Attested
        },
        tier,
        code_hash: record.request.code.hash.clone(),
        input_set_hash: hash_serializable(&input_hashes)?,
        env_hash: hash_serializable(&BTreeMap::<String, String>::new())?,
        toolchain_hash: None,
        sandbox_profile_hash: hash_bytes(if tier_w {
            b"prometheus-exec-tier-w-no-instantiation-v1"
        } else {
            b"prometheus-exec-tier-p-no-spawn-v1"
        }),
        backend: if tier_w {
            tier_w_backend()
        } else {
            host_backend()
        },
        exit: ExecutionExit {
            status: 125,
            signal_or_trap: Some(reason.into()),
        },
        outputs: ExecutionOutputs {
            stdout: stdout.hash,
            stderr: stderr.hash,
            artifacts: vec![],
        },
        usage: ResourceUsage::default(),
        started_at: now,
        finished_at: now,
        executing_device: ExecutingDevice {
            key_id: String::new(),
            sig_alg: SignatureAlgorithm::Ed25519,
            platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        },
        grants: vec![],
        component: None,
        failure: Some(ExecutionFailure {
            kind: failure_kind,
            code: failure_code(failure_kind).into(),
            message: reason.into(),
        }),
        signature: None,
    };
    sign_receipt_ed25519(&mut receipt, signing_key)?;
    Ok(receipt)
}

#[cfg(unix)]
fn failure_code(kind: ExecutionFailureKind) -> &'static str {
    match kind {
        ExecutionFailureKind::Trap => "backend_error",
        ExecutionFailureKind::FuelExhausted => "fuel_exhausted",
        ExecutionFailureKind::EpochDeadline => "epoch_deadline",
        ExecutionFailureKind::MemoryLimit => "memory_limit",
        ExecutionFailureKind::TableLimit => "table_limit",
        ExecutionFailureKind::InstanceLimit => "instance_limit",
        ExecutionFailureKind::StreamLimit => "stream_limit",
        ExecutionFailureKind::ArtifactLimit => "artifact_limit",
        ExecutionFailureKind::CapabilityDenied => "capability_denied",
        ExecutionFailureKind::ComponentUnauthorized => "component_unauthorized",
        ExecutionFailureKind::BackendUnavailable => "backend_unavailable",
        ExecutionFailureKind::Interrupted => "interrupted",
    }
}

#[cfg(unix)]
fn tier_w_backend() -> ExecutionBackend {
    match compiled_backend() {
        BackendProfile::Cranelift => ExecutionBackend::Cranelift,
        BackendProfile::Pulley => ExecutionBackend::Pulley,
    }
}

fn host_backend() -> ExecutionBackend {
    #[cfg(target_os = "macos")]
    {
        ExecutionBackend::Seatbelt
    }
    #[cfg(target_os = "linux")]
    {
        ExecutionBackend::Bwrap
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        ExecutionBackend::Bwrap
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use prometheus_exec_contracts::{
        sign_request_ed25519, CapabilityManifest, CodeIdentity, CodeKind, ComponentAuthorization,
        ComponentAuthorizationMode, ExecutionLimits, ExecutionProvenance, SignatureAlgorithm,
        SignedExecRequest,
    };
    use tempfile::tempdir;
    use uuid::Uuid;

    const REFERENCE_COMPONENT: &[u8] = include_bytes!(
        "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
    );

    #[tokio::test]
    async fn tier_w_uses_the_shared_durable_ledger_events_and_receipt_path() {
        let directory = tempdir().unwrap();
        let service = ExecutionService::open(directory.path().join("service")).unwrap();
        let artifacts =
            ArtifactStore::open(directory.path().join("artifacts"), 64 * 1024 * 1024).unwrap();
        let signing_key = SigningKey::from_bytes(&[89; 32]);
        let verification_key = VerificationKey::ed25519(signing_key.verifying_key().to_bytes());
        let code = artifacts.put(REFERENCE_COMPONENT).unwrap();
        let authorization = ComponentAuthorization {
            mode: ComponentAuthorizationMode::HashPin,
            world: prometheus_exec_tier_w::COMPONENT_WORLD.into(),
            manifest_hash: None,
            generation_id: None,
        };
        let mut request = SignedExecRequest {
            schema_version: SCHEMA_VERSION.into(),
            request_id: Uuid::new_v4(),
            issued_at: Utc::now(),
            queued_at: None,
            validity_window_secs: 300,
            tier: RequestedTier::W,
            code: CodeIdentity {
                kind: CodeKind::Component,
                hash: code.hash.clone(),
                runtime: RuntimeKind::WasmComponent,
                toolchain_pin: None,
            },
            inputs: Vec::new(),
            capabilities: CapabilityManifest {
                fs: prometheus_exec_contracts::FilesystemCapabilities {
                    read_only: vec![
                        ".kbd-orchestrator/project.json".into(),
                        ".evolver/".into(),
                        ".refiner/".into(),
                        "openspec/".into(),
                    ],
                    read_write: Vec::new(),
                },
                net: prometheus_exec_contracts::NetworkCapabilities::default(),
                env: prometheus_exec_contracts::EnvironmentCapabilities::default(),
                clock: false,
                random: false,
            },
            limits: ExecutionLimits::default(),
            targets: Vec::new(),
            provenance: ExecutionProvenance {
                component_authorization: Some(authorization),
                ..ExecutionProvenance::default()
            },
            signer_key_id: None,
            sig_alg: SignatureAlgorithm::Ed25519,
            signature: None,
        };
        sign_request_ed25519(&mut request, &signing_key).unwrap();
        artifacts.retain_for_request(&request).unwrap();
        let accepted = service.submit(request.clone()).unwrap().record;
        let backends = test_backends(code.hash);

        process_queued_run(
            &service,
            &artifacts,
            &signing_key,
            &verification_key,
            &backends,
            &CedarTighteningPolicy::default(),
            accepted,
        )
        .await
        .unwrap();

        let terminal = service.ledger().get(request.request_id).unwrap().unwrap();
        assert_eq!(terminal.state, RunState::Succeeded, "{terminal:#?}");
        let receipt = terminal.terminal.unwrap().receipt;
        assert_eq!(receipt.tier, ExecutionTier::W);
        assert_eq!(receipt.evidence_class, EvidenceClass::Verified);
        assert!(matches!(
            receipt.backend,
            ExecutionBackend::Cranelift | ExecutionBackend::Pulley
        ));
        assert!(receipt.component.is_some());
        assert!(artifacts.get(&receipt.outputs.stdout).is_ok());
        assert!(artifacts.is_pinned(&receipt.outputs.stdout).unwrap());

        let events = service.events_after(receipt.run_id, 0).unwrap();
        assert!(events
            .iter()
            .any(|event| matches!(&event.data, RunEventData::Accepted { .. })));
        assert!(events
            .iter()
            .any(|event| matches!(&event.data, RunEventData::Completed { .. })));
    }

    fn test_backends(component_hash: prometheus_exec_contracts::Digest) -> RunnerBackends {
        let tier_w = Some(TierWExecutionPort::new(
            TierWEngine::new(EngineProfile::for_current_target()).unwrap(),
            ComponentAuthorizer::hash_pins([component_hash.clone()]),
        ));
        #[cfg(target_os = "macos")]
        {
            RunnerBackends {
                tier_w,
                tier_w_trust: ComponentAuthorizer::hash_pins([component_hash]),
                tier_p: None,
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            RunnerBackends {
                tier_w,
                tier_w_trust: ComponentAuthorizer::hash_pins([component_hash]),
            }
        }
    }
}
