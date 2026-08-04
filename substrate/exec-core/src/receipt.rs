use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use ed25519_dalek::{Signer as _, SigningKey};
use prometheus_exec_contracts::{
    canonical_bytes_without, hash_bytes, hash_serializable, key_id, ArtifactReference,
    ExecutingDevice, ExecutionOutputs, ExecutionReceipt, ExecutionTier, RequestedTier,
    SignatureAlgorithm,
};
use thiserror::Error;
use uuid::Uuid;

use crate::{BackendExecution, ValidatedExecutionJob};

pub trait ReceiptSigner: Send + Sync {
    fn algorithm(&self) -> SignatureAlgorithm;
    fn key_id(&self) -> String;
    fn sign(&self, canonical_payload: &[u8]) -> Result<Vec<u8>, ReceiptAssemblyError>;
}

pub struct Ed25519ReceiptSigner {
    key: SigningKey,
}

impl Ed25519ReceiptSigner {
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }

    pub fn public_key(&self) -> [u8; 32] {
        self.key.verifying_key().to_bytes()
    }
}

impl ReceiptSigner for Ed25519ReceiptSigner {
    fn algorithm(&self) -> SignatureAlgorithm {
        SignatureAlgorithm::Ed25519
    }

    fn key_id(&self) -> String {
        key_id(self.algorithm(), &self.public_key())
    }

    fn sign(&self, canonical_payload: &[u8]) -> Result<Vec<u8>, ReceiptAssemblyError> {
        let signature: ed25519_dalek::Signature = self.key.sign(canonical_payload);
        Ok(signature.to_bytes().to_vec())
    }
}

pub struct ReceiptAssembler<S> {
    signer: S,
}

#[derive(Debug, Error)]
pub enum ReceiptAssemblyError {
    #[error("requested tier {requested:?} does not permit backend tier {actual:?}")]
    TierMismatch {
        requested: RequestedTier,
        actual: ExecutionTier,
    },
    #[error("receipt contract failed: {0}")]
    Contract(#[from] prometheus_exec_contracts::ContractError),
    #[error("receipt signer failed: {0}")]
    Signer(String),
}

impl<S: ReceiptSigner> ReceiptAssembler<S> {
    pub fn new(signer: S) -> Self {
        Self { signer }
    }

    pub fn assemble(
        &self,
        job: &ValidatedExecutionJob,
        execution: BackendExecution,
    ) -> Result<ExecutionReceipt, ReceiptAssemblyError> {
        self.assemble_for_run(Uuid::new_v4(), job, execution)
    }

    /// Assemble a receipt for a run identifier durably assigned before spawn.
    pub fn assemble_for_run(
        &self,
        run_id: Uuid,
        job: &ValidatedExecutionJob,
        execution: BackendExecution,
    ) -> Result<ExecutionReceipt, ReceiptAssemblyError> {
        let requested = job.request().tier;
        let allowed = matches!(requested, RequestedTier::Auto)
            || matches!(
                (requested, execution.tier),
                (RequestedTier::W, ExecutionTier::W)
            )
            || matches!(
                (requested, execution.tier),
                (RequestedTier::P, ExecutionTier::P)
            );
        if !allowed {
            return Err(ReceiptAssemblyError::TierMismatch {
                requested,
                actual: execution.tier,
            });
        }

        let artifacts = execution
            .artifacts
            .into_iter()
            .map(|artifact| ArtifactReference {
                path: artifact.path,
                hash: hash_bytes(&artifact.bytes),
                size_bytes: Some(artifact.bytes.len() as u64),
            })
            .collect();
        let outputs = ExecutionOutputs {
            stdout: hash_bytes(&execution.stdout),
            stderr: hash_bytes(&execution.stderr),
            artifacts,
        };
        let env_hash = hash_serializable(&execution.environment)?;

        let mut receipt = ExecutionReceipt {
            schema_version: prometheus_exec_contracts::SCHEMA_VERSION.into(),
            run_id,
            request_hash: job.request().request_hash()?,
            state: execution.state,
            evidence_class: execution.evidence_class,
            tier: execution.tier,
            code_hash: job.request().code.hash.clone(),
            input_set_hash: job.input_set_hash().clone(),
            env_hash,
            toolchain_hash: execution.toolchain_hash,
            sandbox_profile_hash: execution.sandbox_profile_hash,
            backend: execution.backend,
            exit: execution.exit,
            outputs,
            usage: execution.usage,
            started_at: execution.started_at,
            finished_at: execution.finished_at,
            executing_device: ExecutingDevice {
                key_id: self.signer.key_id(),
                sig_alg: self.signer.algorithm(),
                platform: execution.platform,
            },
            grants: job.grants().to_vec(),
            signature: None,
        };
        receipt.validate()?;
        let canonical = canonical_bytes_without(&receipt, "signature")?;
        receipt.signature = Some(
            URL_SAFE_NO_PAD.encode(
                self.signer
                    .sign(&canonical)
                    .map_err(|error| ReceiptAssemblyError::Signer(error.to_string()))?,
            ),
        );
        Ok(receipt)
    }
}
