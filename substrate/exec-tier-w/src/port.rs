use std::{collections::BTreeMap, time::Instant};

use async_trait::async_trait;
use chrono::Utc;
use prometheus_exec_contracts::{
    canonical_bytes, hash_serializable, ComponentProvenance, EvidenceClass, ExecutionBackend,
    ExecutionExit, ExecutionTier, ResourceUsage, RuntimeKind,
};
use prometheus_exec_core::{
    BackendExecution, ExecutionPort, ProducedArtifact, ValidatedExecutionJob,
};
use serde::Serialize;
use sha2::Digest as _;

use crate::{
    engine::WASMTIME_VERSION, BackendProfile, CapabilityGrant, ComponentAuthorizer, TierWEngine,
    TierWError, TierWExecutionOutcome, TierWLimits,
};

const DETERMINISTIC_RANDOM_BYTES: usize = 4096;

#[derive(Debug, thiserror::Error)]
pub enum TierWPortError {
    #[error("Tier W adapter failed: {0}")]
    Adapter(#[from] TierWError),
    #[error("Tier W request contract failed: {0}")]
    Contract(#[from] prometheus_exec_contracts::ContractError),
    #[error("Tier W requires a wasm-component request")]
    InvalidRuntime,
    #[error("request component authorization differs from the active authorization")]
    AuthorizationMismatch,
    #[error("Tier W capability is unsupported: {0}")]
    UnsupportedCapability(String),
    #[error("request issuedAt precedes the Unix epoch")]
    InvalidClock,
    #[error("canonical request bytes are not UTF-8")]
    InvalidInvocation,
}

pub struct TierWExecutionPort {
    engine: TierWEngine,
    authorizer: ComponentAuthorizer,
}

impl TierWExecutionPort {
    pub fn new(engine: TierWEngine, authorizer: ComponentAuthorizer) -> Self {
        Self { engine, authorizer }
    }

    pub fn engine(&self) -> &TierWEngine {
        &self.engine
    }

    pub fn authorizer(&self) -> &ComponentAuthorizer {
        &self.authorizer
    }

    /// Performs every fallible trust, component, capability, limit, and
    /// invocation check before a durable caller crosses its spawn boundary.
    pub fn preflight_job(&self, job: &ValidatedExecutionJob) -> Result<(), TierWPortError> {
        self.prepare_job(job).map(|_| ())
    }

    pub(crate) fn execute_job(
        &self,
        job: &ValidatedExecutionJob,
    ) -> Result<BackendExecution, TierWPortError> {
        let (component, grant, limits, invocation) = self.prepare_job(job)?;
        let capability_hash = hash_serializable(&grant)?;
        let sandbox_profile_hash = hash_serializable(&SandboxIdentity {
            schema_version: 1,
            backend_profile_hash: self.engine.profile_hash(),
            capability_hash: &capability_hash,
            limits: &limits,
        })?;
        let started_at = Utc::now();
        let started = Instant::now();
        let outcome = self.engine.execute_component(
            &component,
            &self.authorizer,
            grant.clone(),
            &limits,
            &invocation,
        );
        let finished_at = Utc::now();
        let wall_clock_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        let deterministic_projection_hash = outcome
            .deterministic_receipt_projection(&component, &grant, &limits, &invocation)?
            .canonical_hash()?;
        let component_provenance = Some(ComponentProvenance {
            authorization: component.authorization().clone(),
            engine_version: WASMTIME_VERSION.into(),
            backend_profile_hash: self.engine.profile_hash().clone(),
            deterministic_projection_hash,
        });
        let backend = match self.engine.profile().backend {
            BackendProfile::Cranelift => ExecutionBackend::Cranelift,
            BackendProfile::Pulley => ExecutionBackend::Pulley,
        };

        let (state, exit, stdout, stderr, artifacts, fuel_consumed, failure) = match outcome {
            TierWExecutionOutcome::Succeeded(success) => (
                success.state,
                ExecutionExit {
                    status: 0,
                    signal_or_trap: None,
                },
                success.output.into_bytes(),
                canonical_bytes(&success.logs)?,
                success
                    .artifacts
                    .into_iter()
                    .map(|(path, bytes)| ProducedArtifact { path, bytes })
                    .collect(),
                success.fuel_consumed,
                None,
            ),
            TierWExecutionOutcome::Failed(failed) => {
                let code = failed.failure.code.clone();
                (
                    failed.state,
                    ExecutionExit {
                        status: 1,
                        signal_or_trap: Some(code),
                    },
                    Vec::new(),
                    canonical_bytes(&Vec::<crate::HostLogEntry>::new())?,
                    Vec::new(),
                    failed.fuel_consumed,
                    Some(failed.failure),
                )
            }
        };

        Ok(BackendExecution {
            state,
            evidence_class: EvidenceClass::Verified,
            tier: ExecutionTier::W,
            sandbox_profile_hash,
            backend,
            exit,
            stdout,
            stderr,
            artifacts,
            usage: ResourceUsage {
                wall_clock_ms,
                cpu_ms: 0,
                peak_mem_mb: 0,
                fuel_consumed,
            },
            started_at,
            finished_at,
            toolchain_hash: None,
            environment: BTreeMap::new(),
            platform: format!(
                "{}-{}",
                self.engine.profile().target.name(),
                std::env::consts::ARCH
            ),
            component: component_provenance,
            failure,
        })
    }

    fn prepare_job(
        &self,
        job: &ValidatedExecutionJob,
    ) -> Result<
        (
            crate::ValidatedComponent,
            CapabilityGrant,
            TierWLimits,
            String,
        ),
        TierWPortError,
    > {
        if job.request().code.runtime != RuntimeKind::WasmComponent {
            return Err(TierWPortError::InvalidRuntime);
        }
        let component = self.engine.load_component(job.code(), &self.authorizer)?;
        let requested_authorization = job
            .request()
            .provenance
            .component_authorization
            .as_ref()
            .ok_or(TierWPortError::AuthorizationMismatch)?;
        if requested_authorization != component.authorization() {
            return Err(TierWPortError::AuthorizationMismatch);
        }
        let grant = capability_grant(job)?;
        let limits = TierWLimits::from_contract(&job.request().limits)?;
        let invocation = String::from_utf8(job.request().canonical_unsigned()?)
            .map_err(|_| TierWPortError::InvalidInvocation)?;
        Ok((component, grant, limits, invocation))
    }
}

#[async_trait]
impl ExecutionPort for TierWExecutionPort {
    type Error = TierWPortError;

    fn tier(&self) -> ExecutionTier {
        ExecutionTier::W
    }

    async fn execute(&self, job: &ValidatedExecutionJob) -> Result<BackendExecution, Self::Error> {
        self.execute_job(job)
    }
}

pub(crate) fn capability_grant(
    job: &ValidatedExecutionJob,
) -> Result<CapabilityGrant, TierWPortError> {
    let capabilities = &job.request().capabilities;
    if let Some(egress) = capabilities.net.egress.first() {
        return Err(TierWPortError::UnsupportedCapability(format!(
            "network egress {egress}"
        )));
    }
    if let Some(environment) = capabilities.env.read.first() {
        return Err(TierWPortError::UnsupportedCapability(format!(
            "environment read {environment}"
        )));
    }
    let mut grant = CapabilityGrant::empty().allow_log();
    for path in &capabilities.fs.read_only {
        grant = grant.allow_kv_scope(path)?;
    }
    for path in &capabilities.fs.read_write {
        // The public contract represents directory scopes with a trailing
        // separator (for example `outputs/`), while the capability host stores
        // canonical path prefixes without empty segments.
        grant = grant.allow_output_prefix(path.trim_end_matches(['/', '\\']))?;
    }
    for (name, bytes) in job.inputs() {
        grant = grant.allow_input(name, Some(bytes.clone()));
    }
    if capabilities.clock {
        let milliseconds = u64::try_from(job.request().issued_at.timestamp_millis())
            .map_err(|_| TierWPortError::InvalidClock)?;
        grant = grant.allow_clock(milliseconds);
    }
    if capabilities.random {
        grant = grant.allow_random(deterministic_random(&job.request().request_hash()?));
    }
    Ok(grant)
}

fn deterministic_random(request_hash: &prometheus_exec_contracts::Digest) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(DETERMINISTIC_RANDOM_BYTES);
    for counter in 0u32..(DETERMINISTIC_RANDOM_BYTES / 32) as u32 {
        let mut hasher = sha2::Sha256::new();
        hasher.update(request_hash.as_str().as_bytes());
        hasher.update(counter.to_be_bytes());
        bytes.extend_from_slice(&hasher.finalize());
    }
    bytes
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SandboxIdentity<'a> {
    schema_version: u32,
    backend_profile_hash: &'a prometheus_exec_contracts::Digest,
    capability_hash: &'a prometheus_exec_contracts::Digest,
    limits: &'a TierWLimits,
}

#[cfg(all(test, feature = "cranelift"))]
mod tests {
    use super::*;
    use chrono::TimeZone as _;
    use ed25519_dalek::SigningKey;
    use prometheus_exec_contracts::{
        hash_bytes, verify_receipt, CapabilityManifest, CodeIdentity, CodeKind,
        ComponentAuthorization, ComponentAuthorizationMode, EnvironmentCapabilities,
        ExecutionLimits, ExecutionProvenance, FilesystemCapabilities, NetworkCapabilities,
        RequestedTier, RunState, SignatureAlgorithm, SignedExecRequest, VerificationKey,
        SCHEMA_VERSION,
    };
    use prometheus_exec_core::{
        ArtifactBudgetProfile, ArtifactStore, Ed25519ReceiptSigner, ExecutionJob, ReceiptAssembler,
    };
    use tempfile::tempdir;
    use uuid::Uuid;

    const REFERENCE_COMPONENT: &[u8] = include_bytes!(
        "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
    );

    #[test]
    fn execution_port_assembles_and_receipt_first_retains_verified_evidence() {
        let authorization = ComponentAuthorization {
            mode: ComponentAuthorizationMode::HashPin,
            world: crate::COMPONENT_WORLD.into(),
            manifest_hash: None,
            generation_id: None,
        };
        let request = SignedExecRequest {
            schema_version: SCHEMA_VERSION.into(),
            request_id: Uuid::from_u128(0x1234),
            issued_at: Utc.with_ymd_and_hms(2026, 8, 4, 12, 0, 0).unwrap(),
            queued_at: None,
            validity_window_secs: 300,
            tier: RequestedTier::W,
            code: CodeIdentity {
                kind: CodeKind::Component,
                hash: hash_bytes(REFERENCE_COMPONENT),
                runtime: RuntimeKind::WasmComponent,
                toolchain_pin: None,
            },
            inputs: Vec::new(),
            capabilities: CapabilityManifest {
                fs: FilesystemCapabilities {
                    read_only: vec![
                        ".kbd-orchestrator/project.json".into(),
                        ".evolver/".into(),
                        ".refiner/".into(),
                        "openspec/".into(),
                    ],
                    read_write: vec!["outputs/".into()],
                },
                net: NetworkCapabilities::default(),
                env: EnvironmentCapabilities::default(),
                clock: false,
                random: false,
            },
            limits: ExecutionLimits::default(),
            targets: Vec::new(),
            provenance: ExecutionProvenance {
                component_authorization: Some(authorization.clone()),
                ..ExecutionProvenance::default()
            },
            signer_key_id: None,
            sig_alg: SignatureAlgorithm::Ed25519,
            signature: None,
        };
        let job = ExecutionJob {
            request: request.clone(),
            code: REFERENCE_COMPONENT.to_vec(),
            inputs: BTreeMap::new(),
            grants: Vec::new(),
        }
        .validate()
        .unwrap();
        let port = TierWExecutionPort::new(
            TierWEngine::new(crate::EngineProfile::desktop()).unwrap(),
            ComponentAuthorizer::hash_pins([request.code.hash.clone()]),
        );
        assert_eq!(port.tier(), ExecutionTier::W);
        let execution = port.execute_job(&job).unwrap();
        assert_eq!(execution.state, RunState::Succeeded);
        assert_eq!(execution.evidence_class, EvidenceClass::Verified);
        assert_eq!(
            execution.component.as_ref().unwrap().authorization,
            authorization
        );

        let signer = Ed25519ReceiptSigner::new(SigningKey::from_bytes(&[91; 32]));
        let public = VerificationKey::ed25519(signer.public_key());
        let receipt = ReceiptAssembler::new(signer)
            .assemble_for_run(Uuid::from_u128(0x5678), &job, execution.clone())
            .unwrap();
        assert!(verify_receipt(&receipt, &public, Some(&request), None).valid);

        let directory = tempdir().unwrap();
        let store = ArtifactStore::open_for_profile(
            directory.path(),
            ArtifactBudgetProfile::Mobile,
            Some(1),
        )
        .unwrap();
        store.put(REFERENCE_COMPONENT).unwrap();
        assert!(store.get(&receipt.outputs.stdout).is_err());
        store
            .persist_and_retain_for_receipt(&request, &receipt, &execution)
            .unwrap();
        let removable = store.put(b"unreferenced").unwrap();
        let report = store.garbage_collect().unwrap();
        assert!(report.removed.contains(&removable.hash));
        for retained in [
            &request.code.hash,
            &receipt.outputs.stdout,
            &receipt.outputs.stderr,
        ] {
            assert!(store.is_pinned(retained).unwrap());
            assert!(store.get(retained).is_ok());
        }
    }
}
