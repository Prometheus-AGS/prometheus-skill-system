use std::collections::BTreeMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use prometheus_exec_contracts::{
    Digest, EvidenceClass, ExecutionBackend, ExecutionExit, ExecutionTier, ResourceUsage, RunState,
};

use crate::ValidatedExecutionJob;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProducedArtifact {
    pub path: String,
    pub bytes: Vec<u8>,
}

/// Backend observations before content-addressed persistence and signing.
#[derive(Clone, Debug)]
pub struct BackendExecution {
    pub state: RunState,
    pub evidence_class: EvidenceClass,
    pub tier: ExecutionTier,
    pub sandbox_profile_hash: Digest,
    pub backend: ExecutionBackend,
    pub exit: ExecutionExit,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub artifacts: Vec<ProducedArtifact>,
    pub usage: ResourceUsage,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub toolchain_hash: Option<Digest>,
    pub environment: BTreeMap<String, String>,
    pub platform: String,
}

#[async_trait]
pub trait ExecutionPort: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    fn tier(&self) -> ExecutionTier;

    async fn execute(&self, job: &ValidatedExecutionJob) -> Result<BackendExecution, Self::Error>;
}
