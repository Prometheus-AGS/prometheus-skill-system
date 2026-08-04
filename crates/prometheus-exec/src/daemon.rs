use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

#[cfg(target_os = "macos")]
use std::time::Duration;

use chrono::Utc;
use ed25519_dalek::SigningKey;
#[cfg(target_os = "macos")]
use prometheus_exec_contracts::verify_request_signature;
use prometheus_exec_contracts::{
    hash_bytes, hash_serializable, sign_receipt_ed25519, EvidenceClass, ExecutingDevice,
    ExecutionBackend, ExecutionExit, ExecutionOutputs, ExecutionReceipt, ExecutionTier,
    ResourceUsage, RunState, SignatureAlgorithm, VerificationKey, SCHEMA_VERSION,
};
use prometheus_exec_core::ArtifactStore;
#[cfg(target_os = "macos")]
use prometheus_exec_core::{
    BaselinePolicy, CedarTighteningPolicy, Ed25519ReceiptSigner, ExecutionJob, ExecutionPort,
    PolicyEvaluator, PolicyOutcome, ReceiptAssembler,
};
#[cfg(target_os = "macos")]
use prometheus_exec_service::RunEventData;
#[cfg(unix)]
use prometheus_exec_service::UdsSidecar;
use prometheus_exec_service::{ExecutionService, ReadinessStatus, RunRecord, SidecarState};
#[cfg(target_os = "macos")]
use prometheus_exec_tier_p::{SeatbeltConfig, SeatbeltExecutor};

use crate::identity;

#[cfg(target_os = "macos")]
const RUNNER_POLL_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Debug)]
pub struct DaemonConfig {
    pub socket: PathBuf,
    pub state_dir: PathBuf,
    pub identity: PathBuf,
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
            "sandbox",
            ReadinessStatus::Initializing,
            "Tier P sandbox is being detected",
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
            &recovery_key,
            &recovery_artifacts,
        )
        .map_err(|error| error.to_string())
    })?;
    state.install(service.clone(), artifacts.clone()).await;

    #[cfg(target_os = "macos")]
    let runner = match SeatbeltConfig::detect() {
        Ok(detected) => {
            let executor =
                SeatbeltExecutor::new(detected.with_work_root(config.state_dir.join("work")));
            state
                .set_readiness(
                    "sandbox",
                    ReadinessStatus::Ready,
                    "macOS Seatbelt Tier P backend is available",
                )
                .await;
            state
                .set_readiness(
                    "receipt-identity",
                    ReadinessStatus::Ready,
                    format!("receipt signer {} is ready", verification_key.key_id()),
                )
                .await;
            Some(tokio::spawn(runner_loop(
                service.clone(),
                artifacts.clone(),
                signing_key,
                verification_key,
                executor,
            )))
        }
        Err(error) => {
            state
                .set_readiness("sandbox", ReadinessStatus::Failed, error.to_string())
                .await;
            state
                .set_readiness(
                    "receipt-identity",
                    ReadinessStatus::Ready,
                    format!("receipt signer {} is ready", verification_key.key_id()),
                )
                .await;
            None
        }
    };

    #[cfg(not(target_os = "macos"))]
    let runner: Option<
        tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
    > = {
        state
            .set_readiness(
                "sandbox",
                ReadinessStatus::Failed,
                "Tier P execution is not runtime-certified on this platform",
            )
            .await;
        state
            .set_readiness(
                "receipt-identity",
                ReadinessStatus::Ready,
                format!("receipt signer {} is ready", verification_key.key_id()),
            )
            .await;
        None
    };

    tokio::signal::ctrl_c().await?;
    if let Some(runner) = runner {
        runner.abort();
        let _ = runner.await;
    }
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

#[cfg(target_os = "macos")]
async fn runner_loop(
    service: Arc<ExecutionService>,
    artifacts: Arc<ArtifactStore>,
    signing_key: SigningKey,
    verification_key: VerificationKey,
    executor: SeatbeltExecutor,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let policy = CedarTighteningPolicy::default();
    loop {
        let records = service.ledger().records()?;
        for record in &records {
            if !record.state.is_terminal() {
                artifacts.retain_for_request(&record.request)?;
            }
        }
        for record in records {
            if record.state != RunState::Queued {
                continue;
            }
            process_queued_run(
                &service,
                &artifacts,
                &signing_key,
                &verification_key,
                &executor,
                &policy,
                record,
            )
            .await?;
        }
        tokio::time::sleep(RUNNER_POLL_INTERVAL).await;
    }
}

#[cfg(target_os = "macos")]
async fn process_queued_run(
    service: &ExecutionService,
    artifacts: &ArtifactStore,
    signing_key: &SigningKey,
    verification_key: &VerificationKey,
    executor: &SeatbeltExecutor,
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

    let execution = match executor.execute(&job).await {
        Ok(execution) => execution,
        Err(error) => {
            let receipt = synthetic_receipt(
                &claim.record,
                RunState::Failed,
                &format!("Seatbelt execution failed: {error}"),
                signing_key,
                artifacts,
            )?;
            service.commit_terminal(
                claim.record.request_id,
                &claim.record.request_hash,
                receipt,
                verification_key,
            )?;
            return Ok(());
        }
    };

    artifacts.put(&execution.stdout)?;
    artifacts.put(&execution.stderr)?;
    for artifact in &execution.artifacts {
        artifacts.put(&artifact.bytes)?;
    }
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
        .assemble_for_run(claim.record.run_id, &job, execution)?;
    artifacts.retain_for_receipt(&claim.record.request, &receipt)?;
    service.commit_terminal(
        claim.record.request_id,
        &claim.record.request_hash,
        receipt,
        verification_key,
    )?;
    artifacts.garbage_collect()?;
    Ok(())
}

#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
fn reject_run(
    service: &ExecutionService,
    artifacts: &ArtifactStore,
    signing_key: &SigningKey,
    record: &RunRecord,
    reason: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let verification_key = VerificationKey::ed25519(signing_key.verifying_key().to_bytes());
    let receipt = synthetic_receipt(record, RunState::Rejected, reason, signing_key, artifacts)?;
    service.commit_terminal(
        record.request_id,
        &record.request_hash,
        receipt,
        &verification_key,
    )?;
    Ok(())
}

fn synthetic_receipt(
    record: &RunRecord,
    state: RunState,
    reason: &str,
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
    let mut receipt = ExecutionReceipt {
        schema_version: SCHEMA_VERSION.into(),
        run_id: record.run_id,
        request_hash: record.request_hash.clone(),
        state,
        evidence_class: EvidenceClass::Attested,
        tier: ExecutionTier::P,
        code_hash: record.request.code.hash.clone(),
        input_set_hash: hash_serializable(&input_hashes)?,
        env_hash: hash_serializable(&BTreeMap::<String, String>::new())?,
        toolchain_hash: None,
        sandbox_profile_hash: hash_bytes(b"prometheus-exec-no-spawn-v1"),
        backend: host_backend(),
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
        signature: None,
    };
    sign_receipt_ed25519(&mut receipt, signing_key)?;
    artifacts.retain_for_receipt(&record.request, &receipt)?;
    artifacts.garbage_collect()?;
    Ok(receipt)
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
