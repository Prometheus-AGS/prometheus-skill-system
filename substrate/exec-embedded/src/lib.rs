//! Embedded, estate-free Prometheus Exec API.
//!
//! The API never constructs or owns an async runtime. Every blocking engine,
//! ledger, and CAS operation is dispatched with [`tokio::task::spawn_blocking`]
//! onto the process-global runtime supplied by the embedding application.

#![forbid(unsafe_code)]

use std::{collections::BTreeMap, path::PathBuf, sync::Arc};

use chrono::Utc;
use ed25519_dalek::SigningKey;
use prometheus_exec_contracts::{
    ContractError, Digest, EvidenceClass, ExecutionBackend, ExecutionFailure, ExecutionFailureKind,
    ExecutionReceipt, ExecutionTier, ResourceUsage, RunState, SignedExecRequest, TierWReplayResult,
    VerificationKey, VerificationResult,
};
use prometheus_exec_core::{
    ArtifactBudgetProfile, ArtifactStore, BackendExecution, CasError, Ed25519ReceiptSigner,
    ExecutionJob, JobValidationError, PolicyOutcome, PolicyReason, ReceiptAssembler,
    ReceiptAssemblyError,
};
use prometheus_exec_service::{
    ExecutionService, ExecutionServiceError, RunEvent, RunLedgerError, RunRecord, SpawnStatus,
};
use prometheus_exec_tier_w::{
    replay_verified_receipt, BackendProfile, LocalExecutionProfile, LocalExecutionProfileError,
    TierWPortError, TierWReplayError,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

mod adapter;
pub use adapter::{EmbeddedExecutionAdapter, EmbeddedExecutionAdapterError, EmbeddedPublicKey};

#[cfg(all(feature = "standalone", feature = "mobile"))]
compile_error!("embedded builds must select exactly one of standalone or mobile");

#[cfg(not(any(feature = "standalone", feature = "mobile")))]
compile_error!("embedded builds must select exactly one of standalone or mobile");

/// Durable configuration for exactly one embedded API instance per state root.
pub struct EmbeddedExecutionConfig {
    pub state_root: PathBuf,
    pub signing_key: SigningKey,
    pub component_pins: Vec<Digest>,
    pub artifact_budget_bytes: Option<u64>,
}

/// Complete in-memory material for one Tier W invocation.
#[derive(Clone, Debug)]
pub struct EmbeddedRunRequest {
    pub request: SignedExecRequest,
    pub component: Vec<u8>,
    pub inputs: BTreeMap<String, Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedRunResult {
    pub run_id: Uuid,
    pub state: RunState,
    pub replayed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt: Option<ExecutionReceipt>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedRunStatus {
    pub run_id: Uuid,
    pub state: RunState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt: Option<ExecutionReceipt>,
}

#[derive(Clone, Debug)]
pub struct EmbeddedReplayMaterial {
    pub request: SignedExecRequest,
    pub component: Vec<u8>,
    pub inputs: BTreeMap<String, Vec<u8>>,
}

#[derive(Clone, Debug)]
pub struct EmbeddedVerifyRequest {
    pub receipt: ExecutionReceipt,
    pub verification_key: VerificationKey,
    pub request: Option<SignedExecRequest>,
    pub artifact_root: Option<PathBuf>,
    pub replay: Option<EmbeddedReplayMaterial>,
}

#[derive(Clone)]
pub struct EmbeddedExecutionApi {
    inner: Arc<Inner>,
}

struct Inner {
    service: ExecutionService,
    artifacts: ArtifactStore,
    profile: LocalExecutionProfile,
    signing_key: SigningKey,
    verification_key: VerificationKey,
}

#[derive(Debug, Error)]
pub enum EmbeddedExecutionError {
    #[error("embedded execution contract failed: {0}")]
    Contract(#[from] ContractError),
    #[error("embedded execution job is invalid: {0}")]
    Job(#[from] JobValidationError),
    #[error("embedded policy denied execution: {0:?}")]
    PolicyDenied(Vec<PolicyReason>),
    #[error("embedded execution service failed: {0}")]
    Service(#[from] ExecutionServiceError),
    #[error("embedded run ledger failed: {0}")]
    Ledger(#[from] RunLedgerError),
    #[error("embedded artifact store failed: {0}")]
    Artifact(#[from] CasError),
    #[error("embedded Tier W profile failed: {0}")]
    Profile(#[from] LocalExecutionProfileError),
    #[error("embedded Tier W preflight failed: {0}")]
    Preflight(#[from] TierWPortError),
    #[error("embedded receipt assembly failed: {0}")]
    Receipt(#[from] ReceiptAssemblyError),
    #[error("embedded portable replay failed: {0}")]
    Replay(#[from] TierWReplayError),
    #[error("embedded policy decision serialization failed: {0}")]
    PolicySerialization(#[from] serde_json::Error),
    #[error("embedded blocking task failed: {0}")]
    Task(#[from] tokio::task::JoinError),
    #[error("portable verification replay requires the same request supplied to verification")]
    ReplayRequestMismatch,
}

impl EmbeddedExecutionApi {
    /// Opens durable embedded state without creating an async runtime.
    pub fn open(config: EmbeddedExecutionConfig) -> Result<Self, EmbeddedExecutionError> {
        let receipt_root = config.state_root.join("service/ledger/receipts");
        #[cfg(feature = "standalone")]
        let profile =
            LocalExecutionProfile::standalone(receipt_root, config.component_pins.clone())?;
        #[cfg(feature = "mobile")]
        let profile =
            LocalExecutionProfile::bundled_mobile(receipt_root, config.component_pins.clone())?;

        #[cfg(feature = "standalone")]
        let artifact_profile = ArtifactBudgetProfile::Desktop;
        #[cfg(feature = "mobile")]
        let artifact_profile = ArtifactBudgetProfile::Mobile;

        let service = ExecutionService::open(config.state_root.join("service"))?;
        let artifacts = ArtifactStore::open_for_profile(
            config.state_root.join("artifacts"),
            artifact_profile,
            config.artifact_budget_bytes,
        )?;
        let verification_key =
            VerificationKey::ed25519(config.signing_key.verifying_key().to_bytes());
        let inner = Arc::new(Inner {
            service,
            artifacts,
            profile,
            signing_key: config.signing_key,
            verification_key,
        });
        inner.reconcile_restart()?;
        Ok(Self { inner })
    }

    pub fn verification_key(&self) -> VerificationKey {
        self.inner.verification_key.clone()
    }

    /// Runs or exactly replays a request on the embedding process's runtime.
    pub async fn run(
        &self,
        request: EmbeddedRunRequest,
    ) -> Result<EmbeddedRunResult, EmbeddedExecutionError> {
        let inner = Arc::clone(&self.inner);
        tokio::task::spawn_blocking(move || inner.run(request)).await?
    }

    /// Returns the hash-linked lifecycle stream in strict sequence order.
    pub async fn events(
        &self,
        run_id: Uuid,
        after: u64,
    ) -> Result<Vec<RunEvent>, EmbeddedExecutionError> {
        let inner = Arc::clone(&self.inner);
        Ok(
            tokio::task::spawn_blocking(move || inner.service.events_after(run_id, after))
                .await??,
        )
    }

    pub async fn receipt(
        &self,
        run_id: Uuid,
    ) -> Result<Option<ExecutionReceipt>, EmbeddedExecutionError> {
        let inner = Arc::clone(&self.inner);
        Ok(tokio::task::spawn_blocking(move || inner.service.receipt(run_id)).await??)
    }

    pub async fn status(
        &self,
        run_id: Uuid,
    ) -> Result<Option<EmbeddedRunStatus>, EmbeddedExecutionError> {
        let inner = Arc::clone(&self.inner);
        let record = tokio::task::spawn_blocking(move || inner.service.run(run_id)).await??;
        Ok(record.map(|record| EmbeddedRunStatus {
            run_id: record.run_id,
            state: record.state,
            receipt: record.terminal.map(|terminal| terminal.receipt),
        }))
    }

    /// Retrieves and re-hashes any content-addressed input or output artifact.
    pub async fn artifact(&self, hash: Digest) -> Result<Vec<u8>, EmbeddedExecutionError> {
        let inner = Arc::clone(&self.inner);
        Ok(tokio::task::spawn_blocking(move || inner.artifacts.get(&hash)).await??)
    }

    /// Performs transport-free verification and optional portable replay.
    pub async fn verify(
        &self,
        request: EmbeddedVerifyRequest,
    ) -> Result<VerificationResult, EmbeddedExecutionError> {
        tokio::task::spawn_blocking(move || verify_inner(request)).await?
    }
}

impl Inner {
    fn run(
        &self,
        embedded: EmbeddedRunRequest,
    ) -> Result<EmbeddedRunResult, EmbeddedExecutionError> {
        let job = ExecutionJob {
            request: embedded.request,
            code: embedded.component,
            inputs: embedded.inputs,
            grants: Vec::new(),
        }
        .validate()?;
        let request = job.request().clone();
        let request_hash = request.request_hash()?;

        let outcome = self.profile.evaluate(&request);
        if let PolicyOutcome::Denied { reasons } = outcome {
            return Err(EmbeddedExecutionError::PolicyDenied(reasons));
        }
        if matches!(outcome, PolicyOutcome::AutoApproved) {
            self.profile.execution_port().preflight_job(&job)?;
        }

        self.stage_request(&job)?;
        let submitted = match self.service.submit(request.clone()) {
            Ok(submitted) => submitted,
            Err(error) => {
                self.artifacts.release_upload(&request)?;
                return Err(error.into());
            }
        };
        self.artifacts.transfer_upload_to_request(&request)?;

        if let Some(terminal) = submitted.record.terminal {
            self.artifacts.release_request(&request)?;
            return Ok(EmbeddedRunResult {
                run_id: submitted.record.run_id,
                state: submitted.record.state,
                replayed: true,
                receipt: Some(terminal.receipt),
            });
        }

        if let PolicyOutcome::GrantRequired { reasons } = outcome {
            let reason = serde_json::to_string(&reasons)?;
            let record = self.service.mark_grant_pending(
                request.request_id,
                &request_hash,
                "embedded.grant-pending",
                reason,
            )?;
            return Ok(result_from_record(record, submitted.replayed));
        }

        let claimed = self
            .service
            .claim_for_execution(request.request_id, &request_hash)?;
        if !claimed.claimed {
            return Ok(result_from_record(claimed.record, true));
        }

        let execution = self.profile.execute(&job)?;
        let receipt = ReceiptAssembler::new(Ed25519ReceiptSigner::new(self.signing_key.clone()))
            .assemble_for_run(claimed.record.run_id, &job, execution.clone())?;
        self.artifacts
            .persist_and_retain_for_receipt(&request, &receipt, &execution)?;
        self.profile
            .append_receipt(receipt.clone(), &self.verification_key)?;
        let committed = self.service.commit_terminal(
            request.request_id,
            &request_hash,
            receipt,
            &self.verification_key,
        )?;
        self.artifacts.release_request(&request)?;
        Ok(result_from_record(committed.record, submitted.replayed))
    }

    fn stage_request(
        &self,
        job: &prometheus_exec_core::ValidatedExecutionJob,
    ) -> Result<(), CasError> {
        let reason = format!("upload:{}", job.request().request_id);
        self.artifacts.put_pinned(job.code(), &reason)?;
        for bytes in job.inputs().values() {
            self.artifacts.put_pinned(bytes, &reason)?;
        }
        Ok(())
    }

    fn reconcile_restart(&self) -> Result<(), EmbeddedExecutionError> {
        let report = self.service.reconcile(&self.verification_key, |record| {
            self.interrupted_receipt(record)
                .map_err(|error| error.to_string())
        })?;
        for request_id in report
            .recovered_from_log
            .iter()
            .chain(report.interrupted.iter())
        {
            if let Some(record) = self.service.ledger().get(*request_id)? {
                self.artifacts.release_request(&record.request)?;
            }
        }
        Ok(())
    }

    fn interrupted_receipt(
        &self,
        record: &RunRecord,
    ) -> Result<ExecutionReceipt, EmbeddedExecutionError> {
        let code = self.artifacts.get(&record.request.code.hash)?;
        let mut inputs = BTreeMap::new();
        for input in &record.request.inputs {
            inputs.insert(input.name.clone(), self.artifacts.get(&input.hash)?);
        }
        let job = ExecutionJob {
            request: record.request.clone(),
            code,
            inputs,
            grants: Vec::new(),
        }
        .validate()?;
        let started_at = match record.spawn {
            SpawnStatus::Spawned { spawned_at } => spawned_at,
            SpawnStatus::NotSpawned => Utc::now(),
        };
        let backend = match self.profile.execution_port().engine().profile().backend {
            BackendProfile::Cranelift => ExecutionBackend::Cranelift,
            BackendProfile::Pulley => ExecutionBackend::Pulley,
        };
        let execution = BackendExecution {
            state: RunState::Interrupted,
            evidence_class: EvidenceClass::Verified,
            tier: ExecutionTier::W,
            sandbox_profile_hash: self
                .profile
                .execution_port()
                .engine()
                .profile_hash()
                .clone(),
            backend,
            exit: prometheus_exec_contracts::ExecutionExit {
                status: 125,
                signal_or_trap: Some("embedded-restart".into()),
            },
            stdout: Vec::new(),
            stderr: Vec::new(),
            artifacts: Vec::new(),
            usage: ResourceUsage::default(),
            started_at,
            finished_at: Utc::now(),
            toolchain_hash: None,
            environment: BTreeMap::new(),
            platform: format!("embedded-{}", std::env::consts::ARCH),
            component: None,
            failure: Some(ExecutionFailure {
                kind: ExecutionFailureKind::Interrupted,
                code: "embedded.restart-interrupted".into(),
                message: "the embedding process restarted after the durable spawn boundary".into(),
            }),
        };
        let receipt = ReceiptAssembler::new(Ed25519ReceiptSigner::new(self.signing_key.clone()))
            .assemble_for_run(record.run_id, &job, execution.clone())?;
        self.artifacts
            .persist_and_retain_for_receipt(&record.request, &receipt, &execution)?;
        Ok(receipt)
    }
}

fn result_from_record(record: RunRecord, replayed: bool) -> EmbeddedRunResult {
    EmbeddedRunResult {
        run_id: record.run_id,
        state: record.state,
        replayed,
        receipt: record.terminal.map(|terminal| terminal.receipt),
    }
}

fn verify_inner(
    request: EmbeddedVerifyRequest,
) -> Result<VerificationResult, EmbeddedExecutionError> {
    let mut result = prometheus_exec_contracts::verify_receipt(
        &request.receipt,
        &request.verification_key,
        request.request.as_ref(),
        request.artifact_root.as_deref(),
    );
    if let Some(replay) = request.replay {
        if request.request.as_ref() != Some(&replay.request) {
            return Err(EmbeddedExecutionError::ReplayRequestMismatch);
        }
        let replay_result: TierWReplayResult = replay_verified_receipt(
            &request.receipt,
            &replay.request,
            &request.verification_key,
            replay.component,
            replay.inputs,
        )?;
        result.valid &= replay_result.valid;
        result.replay = Some(replay_result);
    }
    Ok(result)
}

/// Release family shared by every execution crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
