use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path, PathBuf},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    hash_bytes, verify_receipt, Digest, ExecutionReceipt, SignatureAlgorithm, SignedExecRequest,
    VerificationKey, SCHEMA_VERSION,
};

const MAX_INDEXED_JSON_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceFile {
    pub path: String,
    pub hash: Digest,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceIdentity {
    pub sig_alg: SignatureAlgorithm,
    pub key_id: String,
    pub public_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactEvidence {
    /// Logical path recorded in the receipt.
    pub receipt_path: String,
    #[serde(flatten)]
    pub file: EvidenceFile,
}

/// Portable, deterministic manifest for a self-contained execution-evidence
/// bundle. Paths are relative to the bundle root and cannot traverse it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionEvidenceIndex {
    pub schema_version: String,
    pub requirement_id: String,
    pub run_id: Uuid,
    pub environment: String,
    pub receipt: EvidenceFile,
    pub request: EvidenceFile,
    pub verification_identity: EvidenceIdentity,
    #[serde(default)]
    pub artifacts: Vec<ArtifactEvidence>,
    #[serde(default)]
    pub environments: Vec<EvidenceFile>,
}

impl ExecutionEvidenceIndex {
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(format!(
                "unsupported execution-evidence schema: {}",
                self.schema_version
            ));
        }
        if self.requirement_id.trim().is_empty() || self.environment.trim().is_empty() {
            return Err("requirement ID and environment must be present".into());
        }
        let mut paths = BTreeSet::new();
        for file in self.files() {
            validate_relative_path(&file.path)?;
            if !paths.insert(file.path.as_str()) {
                return Err(format!("duplicate indexed path: {}", file.path));
            }
        }
        for artifact in &self.artifacts {
            validate_relative_path(&artifact.receipt_path)?;
        }
        let verification_key = VerificationKey::from_base64url(
            self.verification_identity.sig_alg,
            &self.verification_identity.public_key,
        )
        .map_err(|error| error.to_string())?;
        if self.verification_identity.key_id != verification_key.key_id() {
            return Err("verification identity key ID does not match public key".into());
        }
        Ok(())
    }

    pub fn index_hash(&self) -> std::result::Result<Digest, String> {
        self.validate()?;
        crate::hash_serializable(self).map_err(|error| error.to_string())
    }

    fn files(&self) -> impl Iterator<Item = &EvidenceFile> {
        std::iter::once(&self.receipt)
            .chain(std::iter::once(&self.request))
            .chain(self.artifacts.iter().map(|artifact| &artifact.file))
            .chain(self.environments.iter())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceVerificationCheck {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceVerificationFailure {
    pub code: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceVerificationResult {
    pub valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_hash: Option<Digest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_hash: Option<Digest>,
    #[serde(default)]
    pub checks: Vec<EvidenceVerificationCheck>,
    #[serde(default)]
    pub failures: Vec<EvidenceVerificationFailure>,
}

impl EvidenceVerificationResult {
    fn check(&mut self, code: &str, message: impl Into<String>) {
        self.checks.push(EvidenceVerificationCheck {
            code: code.into(),
            message: message.into(),
        });
    }

    fn fail(&mut self, code: &str, message: impl Into<String>, path: Option<String>) {
        self.valid = false;
        self.failures.push(EvidenceVerificationFailure {
            code: code.into(),
            message: message.into(),
            path,
        });
    }
}

/// Verify a bundle using only indexed files and the public identity embedded in
/// the index. This function performs no network access and mutates no state.
pub fn verify_evidence_bundle(
    index: &ExecutionEvidenceIndex,
    bundle_root: &Path,
) -> EvidenceVerificationResult {
    let mut result = EvidenceVerificationResult {
        valid: true,
        ..EvidenceVerificationResult::default()
    };
    match index.index_hash() {
        Ok(hash) => {
            result.index_hash = Some(hash);
            result.check(
                "index.contract",
                "evidence index is canonical and internally valid",
            );
        }
        Err(error) => {
            result.fail("index.contract", error, None);
            return result;
        }
    }

    for file in index.files() {
        match read_indexed_file(bundle_root, file, false) {
            Ok(_) => result.check("file.hash", format!("verified {}", file.path)),
            Err(error) => result.fail("file.hash", error, Some(file.path.clone())),
        }
    }
    if !result.valid {
        return result;
    }

    let receipt: ExecutionReceipt = match read_indexed_json(bundle_root, &index.receipt) {
        Ok(value) => value,
        Err(error) => {
            result.fail("receipt.json", error, Some(index.receipt.path.clone()));
            return result;
        }
    };
    let request: SignedExecRequest = match read_indexed_json(bundle_root, &index.request) {
        Ok(value) => value,
        Err(error) => {
            result.fail("request.json", error, Some(index.request.path.clone()));
            return result;
        }
    };
    let key = match VerificationKey::from_base64url(
        index.verification_identity.sig_alg,
        &index.verification_identity.public_key,
    ) {
        Ok(key) => key,
        Err(error) => {
            result.fail("identity.key", error.to_string(), None);
            return result;
        }
    };
    let receipt_verification = verify_receipt(&receipt, &key, Some(&request), None);
    result.receipt_hash = receipt_verification.receipt_hash;
    if receipt_verification.valid {
        result.check(
            "receipt.signature",
            "receipt signature and request binding are valid",
        );
    } else {
        for failure in receipt_verification.failures {
            result.fail(
                &format!("receipt.{}", failure.code),
                failure.message,
                failure.subject,
            );
        }
    }
    if receipt.run_id == index.run_id {
        result.check("run.id", "index run ID matches the signed receipt");
    } else {
        result.fail(
            "run.id",
            "index run ID does not match the signed receipt",
            Some(index.run_id.to_string()),
        );
    }

    for artifact in &receipt.outputs.artifacts {
        if !index.artifacts.iter().any(|indexed| {
            indexed.receipt_path == artifact.path && indexed.file.hash == artifact.hash
        }) {
            result.fail(
                "artifact.index",
                format!(
                    "receipt artifact {} is absent or hash-mismatched",
                    artifact.path
                ),
                Some(artifact.path.clone()),
            );
        }
    }
    if receipt.outputs.artifacts.len() == index.artifacts.len() {
        result.check(
            "artifact.index",
            "every receipt artifact has one indexed bundle file",
        );
    } else {
        result.fail(
            "artifact.cardinality",
            "indexed artifact count differs from the receipt",
            None,
        );
    }

    if index
        .environments
        .iter()
        .any(|environment| environment.hash == receipt.env_hash)
    {
        result.check(
            "environment.hash",
            "receipt environment hash resolves in the bundle",
        );
    } else {
        result.fail(
            "environment.hash",
            "receipt environment hash is absent from the evidence index",
            None,
        );
    }
    result.valid = result.failures.is_empty();
    result
}

fn read_indexed_json<T: for<'de> Deserialize<'de>>(
    root: &Path,
    file: &EvidenceFile,
) -> std::result::Result<T, String> {
    let bytes = read_indexed_file(root, file, true)?;
    serde_json::from_slice(&bytes).map_err(|error| error.to_string())
}

fn read_indexed_file(
    root: &Path,
    file: &EvidenceFile,
    json: bool,
) -> std::result::Result<Vec<u8>, String> {
    validate_relative_path(&file.path)?;
    let canonical_root = root
        .canonicalize()
        .map_err(|error| format!("cannot resolve bundle root: {error}"))?;
    let candidate = root.join(&file.path);
    let metadata = fs::symlink_metadata(&candidate)
        .map_err(|error| format!("cannot inspect {}: {error}", file.path))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!("indexed path is not a regular file: {}", file.path));
    }
    if metadata.len() != file.size_bytes {
        return Err(format!(
            "size mismatch for {}: expected {}, got {}",
            file.path,
            file.size_bytes,
            metadata.len()
        ));
    }
    if json && metadata.len() > MAX_INDEXED_JSON_BYTES {
        return Err(format!("indexed JSON exceeds size ceiling: {}", file.path));
    }
    let canonical = candidate
        .canonicalize()
        .map_err(|error| format!("cannot resolve {}: {error}", file.path))?;
    if !canonical.starts_with(&canonical_root) {
        return Err(format!("indexed path escapes bundle root: {}", file.path));
    }
    let bytes =
        fs::read(&canonical).map_err(|error| format!("cannot read {}: {error}", file.path))?;
    let actual = hash_bytes(&bytes);
    if actual != file.hash {
        return Err(format!(
            "hash mismatch for {}: expected {}, got {}",
            file.path, file.hash, actual
        ));
    }
    Ok(bytes)
}

fn validate_relative_path(path: &str) -> std::result::Result<(), String> {
    let path = PathBuf::from(path);
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe evidence path: {}", path.display()));
    }
    Ok(())
}
