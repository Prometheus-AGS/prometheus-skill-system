use std::collections::HashMap;

use loro::{ExportMode, LoroDoc, VersionVector};
use std::borrow::Cow;
use storage_provider::{SyncDomain, SyncManifest};

use crate::error::SyncError;

/// Apply an incoming delta for a domain. Rejects LocalOnly domains (KB content invariant).
pub fn apply_incoming_delta(
    manifest: &SyncManifest,
    domain: &SyncDomain,
    delta: &[u8],
    docs: &mut HashMap<SyncDomain, LoroDoc>,
) -> Result<(), SyncError> {
    if !manifest.is_syncable(domain) {
        return Err(SyncError::PrivacyViolation(domain.to_string()));
    }
    let doc = docs.entry(domain.clone()).or_default();
    doc.import(delta)
        .map_err(|e| SyncError::Crdt(format!("Failed to apply delta for {domain}: {e}")))?;
    Ok(())
}

/// Export outgoing delta since a given version vector.
pub fn export_outgoing_delta(
    manifest: &SyncManifest,
    domain: &SyncDomain,
    since_version: Option<&[u8]>,
    docs: &HashMap<SyncDomain, LoroDoc>,
) -> Result<Vec<u8>, SyncError> {
    if !manifest.is_syncable(domain) {
        return Err(SyncError::PrivacyViolation(domain.to_string()));
    }
    let doc = match docs.get(domain) {
        Some(d) => d,
        None => return Ok(vec![]),
    };
    let bytes = if let Some(vv_bytes) = since_version {
        let vv: VersionVector = serde_json::from_slice(vv_bytes)
            .map_err(|e| SyncError::Crdt(format!("Invalid version vector: {e}")))?;
        doc.export(ExportMode::Updates {
            from: Cow::Owned(vv),
        })
        .map_err(|e| SyncError::Crdt(format!("Export updates failed: {e}")))?
    } else {
        doc.export(ExportMode::Snapshot)
            .map_err(|e| SyncError::Crdt(format!("Export snapshot failed: {e}")))?
    };
    Ok(bytes)
}

/// Get the current version vector for a domain (for tracking what we've synced).
pub fn current_version(
    domain: &SyncDomain,
    docs: &HashMap<SyncDomain, LoroDoc>,
) -> Option<Vec<u8>> {
    docs.get(domain)
        .and_then(|doc| serde_json::to_vec(&doc.oplog_vv()).ok())
}
