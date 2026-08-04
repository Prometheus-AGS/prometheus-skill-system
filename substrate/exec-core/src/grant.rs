use std::{
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signer as _, SigningKey};
use prometheus_exec_contracts::{
    canonical_bytes, hash_serializable, key_id, Digest, ExecutionGrant, GrantKind,
    SignatureAlgorithm, SignedExecRequest, VerificationKey, SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub const GRANT_NAMESPACE: &str = "prometheus-exec-grant";
const MAX_GRANT_VALIDITY_HOURS: i64 = 24;
const MAX_SIGNATURE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SshGrantManifest {
    pub schema_version: String,
    pub grant_id: Uuid,
    pub request_hash: Digest,
    pub capabilities_hash: Digest,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub approver_identity: String,
    pub reason: String,
}

impl SshGrantManifest {
    pub fn for_request(
        request: &SignedExecRequest,
        grant_id: Uuid,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        approver_identity: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<Self, GrantValidationError> {
        Ok(Self {
            schema_version: SCHEMA_VERSION.into(),
            grant_id,
            request_hash: request.request_hash()?,
            capabilities_hash: hash_serializable(&request.capabilities)?,
            issued_at,
            expires_at,
            approver_identity: approver_identity.into(),
            reason: reason.into(),
        })
    }

    pub fn canonical_bytes(&self) -> Result<Vec<u8>, GrantValidationError> {
        canonical_bytes(self).map_err(GrantValidationError::from)
    }

    pub fn canonical_hash(&self) -> Result<Digest, GrantValidationError> {
        hash_serializable(self).map_err(GrantValidationError::from)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractiveGrantStatement {
    pub schema_version: String,
    pub grant_id: Uuid,
    pub request_hash: Digest,
    pub capabilities_hash: Digest,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub host_instance_id: String,
    pub approver_label: String,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractiveGrantToken {
    pub statement: InteractiveGrantStatement,
    pub key_id: String,
    pub sig_alg: SignatureAlgorithm,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedGrant {
    pub grant: ExecutionGrant,
    pub approver: String,
    pub valid_until: DateTime<Utc>,
}

#[derive(Debug, Error)]
pub enum GrantValidationError {
    #[error("grant contract failed: {0}")]
    Contract(#[from] prometheus_exec_contracts::ContractError),
    #[error("grant schema version is unsupported: {0}")]
    UnsupportedSchema(String),
    #[error("grant is not bound to this request")]
    RequestMismatch,
    #[error("grant capability hash does not match the request")]
    CapabilitiesMismatch,
    #[error("grant issuance is too far in the future")]
    IssuedInFuture,
    #[error("grant is expired")]
    Expired,
    #[error("grant validity must be positive and at most 24 hours")]
    InvalidValidity,
    #[error("grant identity, host, approver, and reason fields must be non-empty")]
    MissingPurpose,
    #[error("detached SSH signature exceeds the size limit")]
    SignatureTooLarge,
    #[error("SSH signature verification failed: {0}")]
    SshRejected(String),
    #[error("SSH verifier I/O failed at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("interactive grant key or algorithm does not match the trusted host")]
    InteractiveKeyMismatch,
    #[error("interactive grant signature is malformed")]
    InteractiveSignatureMalformed,
    #[error("interactive grant signature verification failed")]
    InteractiveSignatureRejected,
}

pub struct SshGrantVerifier {
    ssh_keygen: PathBuf,
    allowed_signers: PathBuf,
    allowed_clock_skew: Duration,
}

impl SshGrantVerifier {
    pub fn new(ssh_keygen: impl Into<PathBuf>, allowed_signers: impl Into<PathBuf>) -> Self {
        Self {
            ssh_keygen: ssh_keygen.into(),
            allowed_signers: allowed_signers.into(),
            allowed_clock_skew: Duration::minutes(5),
        }
    }

    pub fn verify(
        &self,
        request: &SignedExecRequest,
        manifest: &SshGrantManifest,
        detached_signature: &[u8],
        now: DateTime<Utc>,
    ) -> Result<ValidatedGrant, GrantValidationError> {
        validate_binding(
            request,
            &manifest.schema_version,
            &manifest.request_hash,
            &manifest.capabilities_hash,
            manifest.issued_at,
            manifest.expires_at,
            now,
            self.allowed_clock_skew,
        )?;
        if manifest.approver_identity.trim().is_empty() || manifest.reason.trim().is_empty() {
            return Err(GrantValidationError::MissingPurpose);
        }
        if detached_signature.len() > MAX_SIGNATURE_BYTES {
            return Err(GrantValidationError::SignatureTooLarge);
        }
        let canonical = manifest.canonical_bytes()?;
        let signature_file = tempfile::NamedTempFile::new()
            .map_err(|source| io_error(Path::new("<temporary-signature>"), source))?;
        fs::write(signature_file.path(), detached_signature)
            .map_err(|source| io_error(signature_file.path(), source))?;

        let mut child = Command::new(&self.ssh_keygen)
            .args(["-Y", "verify", "-f"])
            .arg(&self.allowed_signers)
            .args([
                "-I",
                &manifest.approver_identity,
                "-n",
                GRANT_NAMESPACE,
                "-s",
            ])
            .arg(signature_file.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| io_error(&self.ssh_keygen, source))?;
        child
            .stdin
            .take()
            .ok_or_else(|| GrantValidationError::SshRejected("stdin unavailable".into()))?
            .write_all(&canonical)
            .map_err(|source| io_error(&self.ssh_keygen, source))?;
        let output = child
            .wait_with_output()
            .map_err(|source| io_error(&self.ssh_keygen, source))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GrantValidationError::SshRejected(
                stderr.chars().take(512).collect(),
            ));
        }

        Ok(ValidatedGrant {
            grant: ExecutionGrant {
                kind: GrantKind::SshManifest,
                r#ref: Some(manifest.canonical_hash()?),
            },
            approver: manifest.approver_identity.clone(),
            valid_until: manifest.expires_at,
        })
    }
}

pub struct InteractiveGrantIssuer {
    signing_key: SigningKey,
    host_instance_id: String,
}

impl InteractiveGrantIssuer {
    pub fn new(signing_key: SigningKey, host_instance_id: impl Into<String>) -> Self {
        Self {
            signing_key,
            host_instance_id: host_instance_id.into(),
        }
    }

    pub fn public_key(&self) -> VerificationKey {
        VerificationKey::ed25519(self.signing_key.verifying_key().to_bytes())
    }

    pub fn issue(
        &self,
        request: &SignedExecRequest,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
        approver_label: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<InteractiveGrantToken, GrantValidationError> {
        let statement = InteractiveGrantStatement {
            schema_version: SCHEMA_VERSION.into(),
            grant_id: Uuid::new_v4(),
            request_hash: request.request_hash()?,
            capabilities_hash: hash_serializable(&request.capabilities)?,
            issued_at,
            expires_at,
            host_instance_id: self.host_instance_id.clone(),
            approver_label: approver_label.into(),
            reason: reason.into(),
        };
        if statement.host_instance_id.trim().is_empty()
            || statement.approver_label.trim().is_empty()
            || statement.reason.trim().is_empty()
        {
            return Err(GrantValidationError::MissingPurpose);
        }
        let canonical = canonical_bytes(&statement)?;
        let signature: ed25519_dalek::Signature = self.signing_key.sign(&canonical);
        Ok(InteractiveGrantToken {
            statement,
            key_id: key_id(
                SignatureAlgorithm::Ed25519,
                &self.signing_key.verifying_key().to_bytes(),
            ),
            sig_alg: SignatureAlgorithm::Ed25519,
            signature: URL_SAFE_NO_PAD.encode(signature.to_bytes()),
        })
    }
}

pub fn verify_interactive_grant(
    request: &SignedExecRequest,
    token: &InteractiveGrantToken,
    trusted_host_key: &VerificationKey,
    expected_host_instance_id: &str,
    now: DateTime<Utc>,
) -> Result<ValidatedGrant, GrantValidationError> {
    validate_binding(
        request,
        &token.statement.schema_version,
        &token.statement.request_hash,
        &token.statement.capabilities_hash,
        token.statement.issued_at,
        token.statement.expires_at,
        now,
        Duration::minutes(5),
    )?;
    if token.statement.host_instance_id != expected_host_instance_id
        || token.statement.approver_label.trim().is_empty()
        || token.statement.reason.trim().is_empty()
    {
        return Err(GrantValidationError::MissingPurpose);
    }
    if token.sig_alg != SignatureAlgorithm::Ed25519
        || trusted_host_key.algorithm != SignatureAlgorithm::Ed25519
        || token.key_id != trusted_host_key.key_id()
    {
        return Err(GrantValidationError::InteractiveKeyMismatch);
    }
    let public_bytes: [u8; 32] = trusted_host_key
        .bytes
        .as_slice()
        .try_into()
        .map_err(|_| GrantValidationError::InteractiveKeyMismatch)?;
    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&public_bytes)
        .map_err(|_| GrantValidationError::InteractiveKeyMismatch)?;
    let signature_bytes = URL_SAFE_NO_PAD
        .decode(&token.signature)
        .map_err(|_| GrantValidationError::InteractiveSignatureMalformed)?;
    let signature = ed25519_dalek::Signature::from_slice(&signature_bytes)
        .map_err(|_| GrantValidationError::InteractiveSignatureMalformed)?;
    verifying_key
        .verify_strict(&canonical_bytes(&token.statement)?, &signature)
        .map_err(|_| GrantValidationError::InteractiveSignatureRejected)?;

    Ok(ValidatedGrant {
        grant: ExecutionGrant {
            kind: GrantKind::Interactive,
            r#ref: Some(hash_serializable(token)?),
        },
        approver: token.statement.approver_label.clone(),
        valid_until: token.statement.expires_at,
    })
}

#[allow(clippy::too_many_arguments)]
fn validate_binding(
    request: &SignedExecRequest,
    schema_version: &str,
    request_hash: &Digest,
    capabilities_hash: &Digest,
    issued_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    now: DateTime<Utc>,
    allowed_clock_skew: Duration,
) -> Result<(), GrantValidationError> {
    if schema_version != SCHEMA_VERSION {
        return Err(GrantValidationError::UnsupportedSchema(
            schema_version.into(),
        ));
    }
    if request_hash != &request.request_hash()? {
        return Err(GrantValidationError::RequestMismatch);
    }
    if capabilities_hash != &hash_serializable(&request.capabilities)? {
        return Err(GrantValidationError::CapabilitiesMismatch);
    }
    if issued_at > now + allowed_clock_skew {
        return Err(GrantValidationError::IssuedInFuture);
    }
    if expires_at <= now {
        return Err(GrantValidationError::Expired);
    }
    let validity = expires_at - issued_at;
    if validity <= Duration::zero() || validity > Duration::hours(MAX_GRANT_VALIDITY_HOURS) {
        return Err(GrantValidationError::InvalidValidity);
    }
    Ok(())
}

fn io_error(path: impl AsRef<Path>, source: std::io::Error) -> GrantValidationError {
    GrantValidationError::Io {
        path: path.as_ref().to_path_buf(),
        source,
    }
}
