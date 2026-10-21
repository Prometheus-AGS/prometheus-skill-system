//! Operator authority for material phase-scope changes (issue #192).
//!
//! A scope amendment or a successor-phase start is admitted only with a
//! canonical JSON manifest signed under the `prometheus-kbd-scope-amendment`
//! SSH signature namespace by a principal listed in the operator's
//! allowed-signers file. Model-written "approved" fields, reviewer output,
//! memory and imported text have no path into this check.
//!
//! The signature is verified once, at command admission, by `ssh-keygen -Y
//! verify`. The journal keeps the raw manifest, its digest and the signature;
//! replay re-checks the manifest bindings deterministically without spawning
//! processes, and integrity after admission comes from the signed event chain.

use crate::{Result, RuntimeError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub const SCOPE_AUTHORITY_NAMESPACE: &str = "prometheus-kbd-scope-amendment";
pub const SCOPE_AUTHORITY_KIND: &str = "kbd.scope-authority";
const ALLOWED_SIGNERS_FILE: &str = "prometheus/kbd-scope-allowed-signers";
const MIN_REASON_CHARS: usize = 10;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ScopeAction {
    AmendContract,
    StartSuccessor,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScenarioChanges {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub removed: Vec<String>,
}

impl ScenarioChanges {
    pub fn normalized(&self) -> Self {
        let sorted = |items: &[String]| -> Vec<String> {
            items
                .iter()
                .cloned()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect()
        };
        Self {
            added: sorted(&self.added),
            modified: sorted(&self.modified),
            removed: sorted(&self.removed),
        }
    }
}

/// The signed statement. Every field is a string (or string list) so the
/// canonical byte form is identical under RFC 8785 JCS, `jq -S -c` and the
/// JavaScript `canonical()` used by the protected-test approval script.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScopeAuthorityManifest {
    pub schema_version: String,
    pub kind: String,
    pub namespace: String,
    pub action: ScopeAction,
    pub project_id: String,
    pub run_id: String,
    pub phase_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_contract_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_contract_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario_changes: Option<ScenarioChanges>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predecessor_closure_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub successor_phase_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub successor_title: Option<String>,
    pub reason: String,
    pub approver: String,
    pub timestamp: String,
    pub nonce: String,
}

/// What a caller submits: the exact manifest bytes and the armored
/// `ssh-keygen -Y sign` output over `manifest_raw + "\n"`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScopeAuthorityInput {
    pub manifest_raw: String,
    pub signature: String,
}

/// What the journal keeps after admission verified the signature.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ScopeAuthorityRecord {
    pub manifest_raw: String,
    pub manifest_sha256: String,
    pub signature: String,
    pub approver: String,
    pub namespace: String,
    pub allowed_signers_sha256: String,
}

fn authority_error(message: impl Into<String>) -> RuntimeError {
    RuntimeError::ScopeAuthority(message.into())
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// Parse a manifest and require it to be its own canonical form, so the
/// bytes that were signed are the bytes every pack reproduces.
pub fn parse_manifest(raw: &str) -> Result<ScopeAuthorityManifest> {
    if !raw.bytes().all(|b| (0x20..0x7f).contains(&b)) {
        return Err(authority_error(
            "manifest must be printable ASCII canonical JSON with no line breaks",
        ));
    }
    let value: serde_json::Value = serde_json::from_str(raw)
        .map_err(|error| authority_error(format!("manifest is not JSON: {error}")))?;
    let canonical = serde_jcs::to_string(&value)
        .map_err(|error| authority_error(format!("manifest cannot be canonicalized: {error}")))?;
    if canonical != raw {
        return Err(authority_error(
            "manifest is not canonical JSON (sorted keys, compact, no trailing newline)",
        ));
    }
    let manifest: ScopeAuthorityManifest = serde_json::from_value(value)
        .map_err(|error| authority_error(format!("manifest has an invalid shape: {error}")))?;
    validate_common_fields(&manifest)?;
    Ok(manifest)
}

fn validate_common_fields(manifest: &ScopeAuthorityManifest) -> Result<()> {
    if manifest.schema_version != "1"
        || manifest.kind != SCOPE_AUTHORITY_KIND
        || manifest.namespace != SCOPE_AUTHORITY_NAMESPACE
    {
        return Err(authority_error(format!(
            "manifest must declare schemaVersion 1, kind {SCOPE_AUTHORITY_KIND} and namespace {SCOPE_AUTHORITY_NAMESPACE}"
        )));
    }
    let required = [
        ("projectId", &manifest.project_id),
        ("runId", &manifest.run_id),
        ("phaseId", &manifest.phase_id),
        ("approver", &manifest.approver),
        ("timestamp", &manifest.timestamp),
        ("nonce", &manifest.nonce),
    ];
    if let Some((name, _)) = required.iter().find(|(_, value)| value.trim().is_empty()) {
        return Err(authority_error(format!("manifest field {name} is empty")));
    }
    if manifest.reason.trim().chars().count() < MIN_REASON_CHARS {
        return Err(authority_error(format!(
            "manifest reason must be at least {MIN_REASON_CHARS} characters"
        )));
    }
    if chrono::DateTime::parse_from_rfc3339(&manifest.timestamp).is_err() {
        return Err(authority_error("manifest timestamp must be RFC 3339"));
    }
    match manifest.action {
        ScopeAction::AmendContract => {
            let digests = [
                manifest.previous_contract_digest.as_deref(),
                manifest.new_contract_digest.as_deref(),
            ];
            if manifest.contract_path.as_deref().is_none_or(str::is_empty)
                || digests.iter().any(|d| d.is_none_or(|d| !is_digest(d)))
                || manifest.scenario_changes.is_none()
                || manifest.predecessor_closure_key.is_some()
                || manifest.successor_phase_id.is_some()
                || manifest.successor_title.is_some()
            {
                return Err(authority_error(
                    "amend-contract manifest requires contractPath, previous/new 64-hex digests and scenarioChanges, and no successor fields",
                ));
            }
        }
        ScopeAction::StartSuccessor => {
            if manifest
                .predecessor_closure_key
                .as_deref()
                .is_none_or(|k| !is_digest(k))
                || manifest
                    .successor_phase_id
                    .as_deref()
                    .is_none_or(str::is_empty)
                || manifest
                    .successor_title
                    .as_deref()
                    .is_none_or(str::is_empty)
                || manifest.contract_path.is_some()
                || manifest.previous_contract_digest.is_some()
                || manifest.new_contract_digest.is_some()
                || manifest.scenario_changes.is_some()
            {
                return Err(authority_error(
                    "start-successor manifest requires predecessorClosureKey, successorPhaseId and successorTitle, and no contract fields",
                ));
            }
        }
    }
    Ok(())
}

/// The operator-owned trust root. There is deliberately no flag or
/// environment override beyond the standard XDG/HOME locations.
pub fn allowed_signers_path() -> Result<PathBuf> {
    let base = match std::env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
        Some(dir) => PathBuf::from(dir),
        None => std::env::var_os("HOME")
            .filter(|v| !v.is_empty())
            .map(|home| PathBuf::from(home).join(".config"))
            .ok_or_else(|| authority_error("HOME is unset; cannot locate allowed signers"))?,
    };
    Ok(base.join(ALLOWED_SIGNERS_FILE))
}

/// Admission-time verification. Fails closed when the signers file or
/// `ssh-keygen` is unavailable.
pub fn verify(
    input: &ScopeAuthorityInput,
) -> Result<(ScopeAuthorityManifest, ScopeAuthorityRecord)> {
    let manifest = parse_manifest(&input.manifest_raw)?;
    if !input.signature.contains("BEGIN SSH SIGNATURE") {
        return Err(authority_error("signature is not an armored SSH signature"));
    }
    let signers_path = allowed_signers_path()?;
    let signers = std::fs::read(&signers_path).map_err(|error| {
        authority_error(format!(
            "allowed signers {} is unavailable ({error}); scope authority fails closed",
            signers_path.display()
        ))
    })?;
    let signature_file = tempfile_path("sig")?;
    std::fs::write(&signature_file, input.signature.as_bytes())?;
    let outcome = run_ssh_keygen_verify(
        &signers_path,
        &manifest.approver,
        &signature_file,
        &input.manifest_raw,
    );
    let _ = std::fs::remove_file(&signature_file);
    outcome?;
    let record = ScopeAuthorityRecord {
        manifest_sha256: sha256_hex(input.manifest_raw.as_bytes()),
        manifest_raw: input.manifest_raw.clone(),
        signature: input.signature.clone(),
        approver: manifest.approver.clone(),
        namespace: SCOPE_AUTHORITY_NAMESPACE.into(),
        allowed_signers_sha256: sha256_hex(&signers),
    };
    Ok((manifest, record))
}

fn tempfile_path(suffix: &str) -> Result<PathBuf> {
    let dir = std::env::temp_dir();
    for attempt in 0..16u32 {
        let candidate = dir.join(format!(
            "kbd-scope-{}-{}-{attempt}.{suffix}",
            std::process::id(),
            uuid::Uuid::new_v4()
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(authority_error(
        "cannot allocate a temporary signature file",
    ))
}

fn run_ssh_keygen_verify(
    signers: &std::path::Path,
    principal: &str,
    signature: &std::path::Path,
    manifest_raw: &str,
) -> Result<()> {
    let mut child = Command::new("ssh-keygen")
        .args(["-Y", "verify", "-f"])
        .arg(signers)
        .args(["-I", principal, "-n", SCOPE_AUTHORITY_NAMESPACE, "-s"])
        .arg(signature)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            authority_error(format!(
                "ssh-keygen is unavailable ({error}); scope authority fails closed"
            ))
        })?;
    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| authority_error("ssh-keygen stdin unavailable"))?;
        stdin.write_all(manifest_raw.as_bytes())?;
        stdin.write_all(b"\n")?;
    }
    let output = child.wait_with_output()?;
    if output.status.success() {
        return Ok(());
    }
    let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(authority_error(format!(
        "SSH signature verification failed for principal {principal}: {}",
        if detail.is_empty() {
            "no detail".into()
        } else {
            detail
        }
    )))
}

/// Replay-time check: the stored record still names the manifest it hashes,
/// and the manifest still parses canonically. No process is spawned.
pub fn verify_record(record: &ScopeAuthorityRecord) -> Result<ScopeAuthorityManifest> {
    if record.manifest_sha256 != sha256_hex(record.manifest_raw.as_bytes())
        || record.namespace != SCOPE_AUTHORITY_NAMESPACE
    {
        return Err(authority_error(
            "stored scope authority record does not match its manifest",
        ));
    }
    let manifest = parse_manifest(&record.manifest_raw)?;
    if manifest.approver != record.approver {
        return Err(authority_error("stored scope authority approver mismatch"));
    }
    Ok(manifest)
}
