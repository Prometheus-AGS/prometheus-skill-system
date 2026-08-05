use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use prometheus_exec_contracts::{
    hash_serializable, Digest, ExecutionReceipt, SignatureAlgorithm, SignedExecRequest,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{RemoteError, Result};

pub const REMOTE_SCHEMA_VERSION: &str = "1";
const MAX_ENDPOINT_ID_BYTES: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnrollmentBinding {
    pub endpoint_id: String,
    pub sig_alg: SignatureAlgorithm,
    pub key_id: String,
    pub public_key: String,
}

impl EnrollmentBinding {
    pub fn validate(&self) -> Result<()> {
        validate_endpoint(&self.endpoint_id)?;
        if self.key_id.is_empty() || self.public_key.is_empty() {
            return Err(RemoteError::Contract(
                "enrollment key ID and public key must be present".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EnrollmentSnapshot {
    pub schema_version: String,
    pub captured_at: DateTime<Utc>,
    pub bindings: BTreeMap<String, EnrollmentBinding>,
}

impl EnrollmentSnapshot {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != REMOTE_SCHEMA_VERSION {
            return Err(RemoteError::Contract(
                "unsupported enrollment snapshot schema".into(),
            ));
        }
        for (endpoint, binding) in &self.bindings {
            binding.validate()?;
            if endpoint != &binding.endpoint_id {
                return Err(RemoteError::Contract(format!(
                    "enrollment map key {endpoint} does not match binding {}",
                    binding.endpoint_id
                )));
            }
        }
        Ok(())
    }

    pub fn snapshot_hash(&self) -> Result<Digest> {
        self.validate()?;
        Ok(hash_serializable(self)?)
    }

    pub fn binding(&self, endpoint_id: &str) -> Result<&EnrollmentBinding> {
        self.bindings
            .get(endpoint_id)
            .ok_or_else(|| RemoteError::UnknownEndpoint(endpoint_id.into()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignedRemoteDispatch {
    pub schema_version: String,
    pub dispatch_id: Uuid,
    pub request: SignedExecRequest,
    pub request_hash: Digest,
    pub origin_endpoint_id: String,
    pub target_endpoint_id: String,
    pub enrollment_snapshot_hash: Digest,
    pub issued_at: DateTime<Utc>,
    pub validity_window_secs: u64,
    pub signer_key_id: String,
    pub sig_alg: SignatureAlgorithm,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl SignedRemoteDispatch {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != REMOTE_SCHEMA_VERSION {
            return Err(RemoteError::Contract(
                "unsupported remote dispatch schema".into(),
            ));
        }
        validate_endpoint(&self.origin_endpoint_id)?;
        validate_endpoint(&self.target_endpoint_id)?;
        if self.origin_endpoint_id == self.target_endpoint_id {
            return Err(RemoteError::Contract(
                "origin and target endpoints must differ".into(),
            ));
        }
        self.request.validate()?;
        if self.request.request_hash()? != self.request_hash {
            return Err(RemoteError::Contract(
                "requestHash does not match the canonical request".into(),
            ));
        }
        if !self.request.targets.contains(&self.target_endpoint_id) {
            return Err(RemoteError::Contract(
                "target endpoint is absent from the signed execution request".into(),
            ));
        }
        if self.validity_window_secs == 0 {
            return Err(RemoteError::Contract(
                "remote validity window must be non-zero".into(),
            ));
        }
        if self.signer_key_id.is_empty() || self.signature.as_deref().is_none_or(str::is_empty) {
            return Err(RemoteError::Contract(
                "remote signer key ID and signature must be present".into(),
            ));
        }
        Ok(())
    }

    pub fn dispatch_hash(&self) -> Result<Digest> {
        self.validate()?;
        Ok(prometheus_exec_contracts::hash_bytes(
            &prometheus_exec_contracts::canonical_bytes_without(self, "signature")?,
        ))
    }

    pub fn expires_at(&self) -> Result<DateTime<Utc>> {
        let seconds = i64::try_from(self.validity_window_secs)
            .map_err(|_| RemoteError::Contract("validity window exceeds i64".into()))?;
        self.issued_at
            .checked_add_signed(chrono::Duration::seconds(seconds))
            .ok_or_else(|| RemoteError::Contract("validity window overflows timestamp".into()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PeerDispatchState {
    Queued,
    Received,
    Running,
    Applied,
    Rejected,
    Expired,
    Unavailable,
    PendingEvidence,
}

impl PeerDispatchState {
    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Applied
                | Self::Rejected
                | Self::Expired
                | Self::Unavailable
                | Self::PendingEvidence
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PeerDispatchRecord {
    pub endpoint_id: String,
    pub state: PeerDispatchState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt: Option<ExecutionReceipt>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<String>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RemoteDispatchAggregate {
    pub schema_version: String,
    pub dispatch_id: Uuid,
    pub request_hash: Digest,
    pub terminal: bool,
    pub universally_applied: bool,
    pub peers: BTreeMap<String, PeerDispatchRecord>,
}

impl RemoteDispatchAggregate {
    pub fn derive(
        dispatch_id: Uuid,
        request_hash: Digest,
        peers: BTreeMap<String, PeerDispatchRecord>,
    ) -> Result<Self> {
        if peers.is_empty() {
            return Err(RemoteError::Contract(
                "remote aggregate requires at least one peer".into(),
            ));
        }
        for (endpoint, peer) in &peers {
            validate_endpoint(endpoint)?;
            if endpoint != &peer.endpoint_id {
                return Err(RemoteError::Contract(format!(
                    "peer map key {endpoint} does not match record {}",
                    peer.endpoint_id
                )));
            }
            if peer.state == PeerDispatchState::Applied && peer.receipt.is_none() {
                return Err(RemoteError::Contract(format!(
                    "applied peer {endpoint} has no receipt"
                )));
            }
        }
        let terminal = peers.values().all(|peer| peer.state.is_terminal());
        let universally_applied = terminal
            && peers
                .values()
                .all(|peer| peer.state == PeerDispatchState::Applied);
        Ok(Self {
            schema_version: REMOTE_SCHEMA_VERSION.into(),
            dispatch_id,
            request_hash,
            terminal,
            universally_applied,
            peers,
        })
    }
}

fn validate_endpoint(endpoint: &str) -> Result<()> {
    if endpoint.is_empty()
        || endpoint.len() > MAX_ENDPOINT_ID_BYTES
        || endpoint.chars().any(char::is_whitespace)
    {
        return Err(RemoteError::Contract(format!(
            "invalid endpoint ID: {endpoint:?}"
        )));
    }
    Ok(())
}
