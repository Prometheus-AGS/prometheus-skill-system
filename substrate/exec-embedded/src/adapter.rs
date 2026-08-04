use std::collections::BTreeMap;

use prometheus_exec_contracts::{
    SignatureAlgorithm, SignedExecRequest, VerificationKey, VerificationResult,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    EmbeddedExecutionApi, EmbeddedExecutionError, EmbeddedRunRequest, EmbeddedVerifyRequest,
};

/// Public-only receipt verification material safe to cross an FFI or IPC boundary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedPublicKey {
    pub algorithm: SignatureAlgorithm,
    pub key_id: String,
    pub bytes_base64url: String,
}

impl EmbeddedPublicKey {
    fn from_verification_key(key: VerificationKey) -> Self {
        Self {
            algorithm: key.algorithm,
            key_id: key.key_id(),
            bytes_base64url: key.to_base64url(),
        }
    }

    fn verification_key(&self) -> Result<VerificationKey, EmbeddedExecutionAdapterError> {
        let key = VerificationKey::from_base64url(self.algorithm, &self.bytes_base64url)?;
        if key.key_id() != self.key_id {
            return Err(EmbeddedExecutionAdapterError::PublicKeyIdentity);
        }
        Ok(key)
    }
}

/// String/byte adapter directly usable from Tauri commands and FRB wrappers.
///
/// The trusted Rust host constructs this value with an already-configured API;
/// no private signing key is accepted by any adapter method.
#[derive(Clone)]
pub struct EmbeddedExecutionAdapter {
    api: EmbeddedExecutionApi,
}

#[derive(Debug, Error)]
pub enum EmbeddedExecutionAdapterError {
    #[error("embedded adapter input JSON is invalid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("embedded adapter UUID is invalid: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("embedded adapter execution failed: {0}")]
    Execution(#[from] EmbeddedExecutionError),
    #[error("embedded adapter contract failed: {0}")]
    Contract(#[from] prometheus_exec_contracts::ContractError),
    #[error("embedded public verification key identity does not match its bytes")]
    PublicKeyIdentity,
}

impl EmbeddedExecutionAdapter {
    pub fn new(api: EmbeddedExecutionApi) -> Self {
        Self { api }
    }

    pub fn public_key(&self) -> EmbeddedPublicKey {
        EmbeddedPublicKey::from_verification_key(self.api.verification_key())
    }

    pub async fn run_json(
        &self,
        request_json: String,
        component: Vec<u8>,
        inputs: BTreeMap<String, Vec<u8>>,
    ) -> Result<String, EmbeddedExecutionAdapterError> {
        let request = serde_json::from_str::<SignedExecRequest>(&request_json)?;
        let result = self
            .api
            .run(EmbeddedRunRequest {
                request,
                component,
                inputs,
            })
            .await?;
        Ok(serde_json::to_string(&result)?)
    }

    pub async fn status_json(
        &self,
        run_id: String,
    ) -> Result<String, EmbeddedExecutionAdapterError> {
        let result = self.api.status(Uuid::parse_str(&run_id)?).await?;
        Ok(serde_json::to_string(&result)?)
    }

    pub async fn events_json(
        &self,
        run_id: String,
        after: u64,
    ) -> Result<String, EmbeddedExecutionAdapterError> {
        let events = self.api.events(Uuid::parse_str(&run_id)?, after).await?;
        Ok(serde_json::to_string(&events)?)
    }

    pub async fn receipt_json(
        &self,
        run_id: String,
    ) -> Result<String, EmbeddedExecutionAdapterError> {
        let receipt = self.api.receipt(Uuid::parse_str(&run_id)?).await?;
        Ok(serde_json::to_string(&receipt)?)
    }

    pub async fn artifact(&self, hash: String) -> Result<Vec<u8>, EmbeddedExecutionAdapterError> {
        Ok(self
            .api
            .artifact(prometheus_exec_contracts::Digest::parse(hash)?)
            .await?)
    }

    pub async fn verify_json(
        &self,
        receipt_json: String,
        request_json: Option<String>,
        public_key_json: String,
    ) -> Result<String, EmbeddedExecutionAdapterError> {
        let receipt = serde_json::from_str(&receipt_json)?;
        let request = request_json
            .map(|json| serde_json::from_str(&json))
            .transpose()?;
        let public_key = serde_json::from_str::<EmbeddedPublicKey>(&public_key_json)?;
        let result: VerificationResult = self
            .api
            .verify(EmbeddedVerifyRequest {
                receipt,
                verification_key: public_key.verification_key()?,
                request,
                artifact_root: None,
                replay: None,
            })
            .await?;
        Ok(serde_json::to_string(&result)?)
    }
}
