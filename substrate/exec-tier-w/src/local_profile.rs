use std::path::PathBuf;

use prometheus_exec_contracts::{Digest, ExecutionReceipt, SignedExecRequest, VerificationKey};
use prometheus_exec_core::{
    AppendReceiptResult, BackendExecution, CedarTighteningPolicy, PolicyEvaluator, PolicyOutcome,
    ReceiptLog, ReceiptLogError, ValidatedExecutionJob,
};

use crate::{
    ComponentAuthorizer, EngineProfile, TierWEngine, TierWError, TierWExecutionPort, TierWPortError,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalExecutionMode {
    Standalone,
    BundledMobile,
}

#[derive(Debug, thiserror::Error)]
pub enum LocalExecutionProfileError {
    #[error("local component trust failed: {0}")]
    Trust(#[from] TierWError),
    #[error("local receipt log failed: {0}")]
    ReceiptLog(#[from] ReceiptLogError),
    #[error("local Tier W execution failed: {0}")]
    Execution(#[from] TierWPortError),
}

/// Estate-free Tier W runtime material: an embedded one-way-tightening Cedar
/// policy, exact component pins, and a private hash-linked local receipt log.
pub struct LocalExecutionProfile {
    mode: LocalExecutionMode,
    port: TierWExecutionPort,
    policy: CedarTighteningPolicy,
    receipts: ReceiptLog,
}

impl LocalExecutionProfile {
    #[cfg(feature = "standalone")]
    pub fn standalone(
        receipt_log_root: impl Into<PathBuf>,
        component_pins: impl IntoIterator<Item = Digest>,
    ) -> Result<Self, LocalExecutionProfileError> {
        Self::new(
            LocalExecutionMode::Standalone,
            EngineProfile::desktop(),
            ComponentAuthorizer::hash_pins(component_pins),
            receipt_log_root.into(),
        )
    }

    #[cfg(feature = "bundled-mobile")]
    pub fn bundled_mobile(
        receipt_log_root: impl Into<PathBuf>,
        bundled_component_pins: impl IntoIterator<Item = Digest>,
    ) -> Result<Self, LocalExecutionProfileError> {
        Self::new(
            LocalExecutionMode::BundledMobile,
            EngineProfile::for_current_target(),
            ComponentAuthorizer::bundled(bundled_component_pins),
            receipt_log_root.into(),
        )
    }

    fn new(
        mode: LocalExecutionMode,
        engine_profile: EngineProfile,
        authorizer: ComponentAuthorizer,
        receipt_log_root: PathBuf,
    ) -> Result<Self, LocalExecutionProfileError> {
        // Reject an empty or invalid pin set before constructing the engine or
        // creating any durable local state.
        authorizer.inspect()?;
        let engine = TierWEngine::new(engine_profile)?;
        let receipts = ReceiptLog::open(receipt_log_root)?;
        Ok(Self {
            mode,
            port: TierWExecutionPort::new(engine, authorizer),
            policy: CedarTighteningPolicy::embedded_default(),
            receipts,
        })
    }

    pub fn mode(&self) -> LocalExecutionMode {
        self.mode
    }

    pub fn execution_port(&self) -> &TierWExecutionPort {
        &self.port
    }

    pub fn receipt_log(&self) -> &ReceiptLog {
        &self.receipts
    }

    pub fn policy_hash(&self) -> &Digest {
        self.policy.policy_hash()
    }

    pub fn evaluate(&self, request: &SignedExecRequest) -> PolicyOutcome {
        self.policy.evaluate(request)
    }

    pub fn execute(
        &self,
        job: &ValidatedExecutionJob,
    ) -> Result<BackendExecution, LocalExecutionProfileError> {
        Ok(self.port.execute_job(job)?)
    }

    pub fn append_receipt(
        &self,
        receipt: ExecutionReceipt,
        verification_key: &VerificationKey,
    ) -> Result<AppendReceiptResult, LocalExecutionProfileError> {
        Ok(self.receipts.append(receipt, verification_key)?)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::{TimeZone as _, Utc};
    use ed25519_dalek::SigningKey;
    use prometheus_exec_contracts::{
        hash_bytes, CapabilityManifest, CodeIdentity, CodeKind, ComponentAuthorizationMode,
        ExecutionLimits, ExecutionProvenance, RequestedTier, RuntimeKind, SignatureAlgorithm,
        SCHEMA_VERSION,
    };
    use prometheus_exec_core::{Ed25519ReceiptSigner, ExecutionJob, ReceiptAssembler};
    use tempfile::tempdir;
    use uuid::Uuid;

    use super::*;

    const REFERENCE_COMPONENT: &[u8] = include_bytes!(
        "../../../skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm"
    );

    fn prove_local_profile(
        profile: &LocalExecutionProfile,
        expected_mode: ComponentAuthorizationMode,
    ) {
        let authorization = profile
            .execution_port()
            .authorizer()
            .authorization_for_bytes(REFERENCE_COMPONENT)
            .unwrap();
        assert_eq!(authorization.mode, expected_mode);
        let request = SignedExecRequest {
            schema_version: SCHEMA_VERSION.into(),
            request_id: Uuid::from_u128(0x77),
            issued_at: Utc.with_ymd_and_hms(2026, 8, 4, 13, 0, 0).unwrap(),
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
            capabilities: CapabilityManifest::default(),
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
        assert_eq!(profile.evaluate(&request), PolicyOutcome::AutoApproved);
        let job = ExecutionJob {
            request,
            code: REFERENCE_COMPONENT.to_vec(),
            inputs: BTreeMap::new(),
            grants: Vec::new(),
        }
        .validate()
        .unwrap();
        let execution = profile.execute(&job).unwrap();
        let signer = Ed25519ReceiptSigner::new(SigningKey::from_bytes(&[82; 32]));
        let verification_key = VerificationKey::ed25519(signer.public_key());
        let receipt = ReceiptAssembler::new(signer)
            .assemble_for_run(Uuid::from_u128(0x88), &job, execution)
            .unwrap();
        let appended = profile.append_receipt(receipt, &verification_key).unwrap();
        assert!(appended.created);
        assert_eq!(profile.receipt_log().segments().unwrap().len(), 1);
    }

    #[cfg(feature = "standalone")]
    #[test]
    fn standalone_profile_executes_with_hash_pins_policy_and_local_log() {
        let directory = tempdir().unwrap();
        let profile =
            LocalExecutionProfile::standalone(directory.path(), [hash_bytes(REFERENCE_COMPONENT)])
                .unwrap();
        assert_eq!(profile.mode(), LocalExecutionMode::Standalone);
        prove_local_profile(&profile, ComponentAuthorizationMode::HashPin);
    }

    #[cfg(feature = "bundled-mobile")]
    #[test]
    fn bundled_mobile_profile_is_pulley_only_and_uses_bundled_pins() {
        let directory = tempdir().unwrap();
        let profile = LocalExecutionProfile::bundled_mobile(
            directory.path(),
            [hash_bytes(REFERENCE_COMPONENT)],
        )
        .unwrap();
        assert_eq!(profile.mode(), LocalExecutionMode::BundledMobile);
        assert_eq!(
            profile.execution_port().engine().profile().backend,
            crate::BackendProfile::Pulley
        );
        prove_local_profile(&profile, ComponentAuthorizationMode::Bundled);
    }

    #[cfg(feature = "standalone")]
    #[test]
    fn empty_standalone_pins_create_no_receipt_log() {
        let directory = tempdir().unwrap();
        let log = directory.path().join("receipts");
        let result = LocalExecutionProfile::standalone(&log, []);
        assert!(matches!(result, Err(LocalExecutionProfileError::Trust(_))));
        assert!(!log.exists());
    }
}
