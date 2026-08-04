use std::{collections::BTreeSet, fs, path::Path};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    hash_bytes, verify_receipt_signature, ContractError, ExecutionReceipt, SignedExecRequest,
    VerificationKey,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerificationCheck {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerificationFailure {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerificationResult {
    pub valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_hash: Option<crate::Digest>,
    #[serde(default)]
    pub checks: Vec<VerificationCheck>,
    #[serde(default)]
    pub failures: Vec<VerificationFailure>,
}

impl VerificationResult {
    pub fn success(receipt_hash: crate::Digest) -> Self {
        Self {
            valid: true,
            receipt_hash: Some(receipt_hash),
            checks: Vec::new(),
            failures: Vec::new(),
        }
    }

    pub fn record_check(&mut self, code: &str, message: impl Into<String>) {
        self.checks.push(VerificationCheck {
            code: code.into(),
            message: message.into(),
        });
    }

    pub fn record_failure(
        &mut self,
        code: &str,
        message: impl Into<String>,
        subject: Option<String>,
    ) {
        self.valid = false;
        self.failures.push(VerificationFailure {
            code: code.into(),
            message: message.into(),
            subject,
        });
    }
}

pub fn verify_receipt(
    receipt: &ExecutionReceipt,
    key: &VerificationKey,
    request: Option<&SignedExecRequest>,
    artifact_root: Option<&Path>,
) -> VerificationResult {
    let receipt_hash = receipt.receipt_hash().ok();
    let mut result = receipt_hash
        .map(VerificationResult::success)
        .unwrap_or_default();
    if result.receipt_hash.is_none() {
        result.record_failure(
            "receipt.canonicalization",
            "receipt cannot be canonicalized",
            None,
        );
    }

    record_contract_check(
        &mut result,
        "receipt.invariants",
        receipt.validate(),
        "receipt semantic invariants are valid",
    );
    record_contract_check(
        &mut result,
        "receipt.signature",
        verify_receipt_signature(receipt, key),
        "receipt signature and key identity are valid",
    );

    if let Some(request) = request {
        match request.request_hash() {
            Ok(hash) if hash == receipt.request_hash => {
                result.record_check("request.hash", "receipt matches the supplied request");
            }
            Ok(hash) => result.record_failure(
                "request.hash_mismatch",
                format!(
                    "receipt expects {}, supplied request is {}",
                    receipt.request_hash, hash
                ),
                Some(request.request_id.to_string()),
            ),
            Err(error) => result.record_failure(
                "request.canonicalization",
                error.to_string(),
                Some(request.request_id.to_string()),
            ),
        }
    }

    if let Some(root) = artifact_root {
        verify_artifacts(receipt, root, &mut result);
    }
    result
}

fn record_contract_check(
    result: &mut VerificationResult,
    code: &str,
    check: crate::Result<()>,
    success: &str,
) {
    match check {
        Ok(()) => result.record_check(code, success),
        Err(error) => result.record_failure(code, error.to_string(), None),
    }
}

fn verify_artifacts(receipt: &ExecutionReceipt, root: &Path, result: &mut VerificationResult) {
    let canonical_root = match root.canonicalize() {
        Ok(path) => path,
        Err(error) => {
            result.record_failure(
                "artifact.root",
                error.to_string(),
                Some(root.display().to_string()),
            );
            return;
        }
    };
    let mut observed = BTreeSet::new();
    for artifact in &receipt.outputs.artifacts {
        if !observed.insert(artifact.path.as_str()) {
            result.record_failure(
                "artifact.duplicate",
                "duplicate artifact path",
                Some(artifact.path.clone()),
            );
            continue;
        }
        if let Err(error) = crate::validate_artifact_path(&artifact.path) {
            result.record_failure(
                "artifact.path",
                error.to_string(),
                Some(artifact.path.clone()),
            );
            continue;
        }
        let candidate = root.join(&artifact.path);
        let canonical_candidate = match candidate.canonicalize() {
            Ok(path) if path.starts_with(&canonical_root) => path,
            Ok(_) => {
                result.record_failure(
                    "artifact.symlink_escape",
                    "artifact resolves outside the selected root",
                    Some(artifact.path.clone()),
                );
                continue;
            }
            Err(error) => {
                result.record_failure(
                    "artifact.read",
                    error.to_string(),
                    Some(artifact.path.clone()),
                );
                continue;
            }
        };
        match fs::read(&canonical_candidate) {
            Ok(bytes) if hash_bytes(&bytes) == artifact.hash => {
                result.record_check(
                    "artifact.hash",
                    format!("{} matches {}", artifact.path, artifact.hash),
                );
            }
            Ok(bytes) => result.record_failure(
                "artifact.hash_mismatch",
                format!("expected {}, got {}", artifact.hash, hash_bytes(&bytes)),
                Some(artifact.path.clone()),
            ),
            Err(error) => result.record_failure(
                "artifact.read",
                ContractError::ArtifactIo {
                    path: artifact.path.clone(),
                    source: error,
                }
                .to_string(),
                Some(artifact.path.clone()),
            ),
        }
    }
}
