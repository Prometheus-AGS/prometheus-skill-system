use std::{path::Path, sync::Arc};

use prometheus_exec_contracts::{
    verify_receipt, Digest, ExecutionReceipt, SignedExecRequest, VerificationKey,
    VerificationResult,
};
use prometheus_exec_core::{ArtifactStore, CasError};
use thiserror::Error;
use uuid::Uuid;

use crate::{ExecutionService, ExecutionServiceError, RunEvent, RunRecord, SubmitRunResult};

/// Maximum artifact size returned inline unless a caller deliberately chooses
/// a smaller ceiling.
pub const DEFAULT_INLINE_ARTIFACT_BYTES: usize = 1024 * 1024;

/// Transport-independent local execution operations shared by REST, MCP, and
/// embedded adapters.
///
/// The facade owns no runtime and performs no execution itself. It preserves
/// the durable service/CAS ordering while keeping transport-specific parsing,
/// streaming, and response envelopes outside the execution boundary.
#[derive(Clone, Debug)]
pub struct LocalExecutionFacade {
    service: Arc<ExecutionService>,
    artifacts: Arc<ArtifactStore>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactPayload {
    pub digest: Digest,
    pub size_bytes: u64,
    pub bytes: Option<Vec<u8>>,
}

impl ArtifactPayload {
    pub fn is_inline(&self) -> bool {
        self.bytes.is_some()
    }
}

#[derive(Debug, Error)]
pub enum LocalExecutionFacadeError {
    #[error("execution service failed: {0}")]
    Service(#[from] ExecutionServiceError),
    #[error("artifact store failed: {0}")]
    Artifact(#[from] CasError),
    #[error("artifact inline ceiling must be non-zero")]
    InvalidInlineCeiling,
}

impl LocalExecutionFacade {
    pub fn new(service: Arc<ExecutionService>, artifacts: Arc<ArtifactStore>) -> Self {
        Self { service, artifacts }
    }

    pub fn service(&self) -> &Arc<ExecutionService> {
        &self.service
    }

    pub fn artifacts(&self) -> &Arc<ArtifactStore> {
        &self.artifacts
    }

    /// Transfer upload ownership before accepting the request, and release it
    /// on every failure or terminal replay path.
    pub fn submit(
        &self,
        request: SignedExecRequest,
    ) -> Result<SubmitRunResult, LocalExecutionFacadeError> {
        self.artifacts.transfer_upload_to_request(&request)?;
        match self.service.submit(request.clone()) {
            Ok(result) => {
                if result.record.state.is_terminal() {
                    self.artifacts.release_request(&request)?;
                }
                Ok(result)
            }
            Err(error) => match self.artifacts.release_request(&request) {
                Ok(()) => Err(error.into()),
                Err(rollback) => Err(LocalExecutionFacadeError::Artifact(rollback)),
            },
        }
    }

    pub fn run(&self, run_id: Uuid) -> Result<Option<RunRecord>, LocalExecutionFacadeError> {
        Ok(self.service.run(run_id)?)
    }

    pub fn events_after(
        &self,
        run_id: Uuid,
        after: u64,
    ) -> Result<Vec<RunEvent>, LocalExecutionFacadeError> {
        Ok(self.service.events_after(run_id, after)?)
    }

    pub fn receipt(
        &self,
        run_id: Uuid,
    ) -> Result<Option<ExecutionReceipt>, LocalExecutionFacadeError> {
        Ok(self.service.receipt(run_id)?)
    }

    pub fn artifact(
        &self,
        digest: &Digest,
        inline_ceiling: usize,
    ) -> Result<ArtifactPayload, LocalExecutionFacadeError> {
        if inline_ceiling == 0 {
            return Err(LocalExecutionFacadeError::InvalidInlineCeiling);
        }
        let bytes = self.artifacts.get(digest)?;
        let size_bytes = bytes.len() as u64;
        Ok(ArtifactPayload {
            digest: digest.clone(),
            size_bytes,
            bytes: (bytes.len() <= inline_ceiling).then_some(bytes),
        })
    }

    pub fn verify(
        &self,
        receipt: &ExecutionReceipt,
        key: &VerificationKey,
        request: Option<&SignedExecRequest>,
        artifact_root: Option<&Path>,
    ) -> VerificationResult {
        verify_receipt(receipt, key, request, artifact_root)
    }
}
