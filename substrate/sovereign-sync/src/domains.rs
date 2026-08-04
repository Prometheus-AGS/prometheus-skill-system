//! Domain adapters bridging real local data (skill index, learner model, KBD
//! authority) to the generic CRDT sync machinery in `crdt.rs`.
//!
//! Each domain family has a fixed [`storage_provider::PrivacyClass`] and a
//! [`DomainAdapter`] that exports local state as JSON (for merging into a
//! `LoroDoc` via [`storage_provider::LoroAdapter`]'s `apply_json`) and
//! imports a merged JSON view back into the real local store.
//!
//! Adapters are deliberately synchronous-CRDT / async-storage: a `LoroDoc` is
//! never held across an `.await` here, since Loro's document type is not
//! `Send` and axum handler futures must be. Callers do all Loro manipulation
//! in a plain synchronous block, and only exchange JSON/bytes with adapters.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;
use storage_provider::{DomainConfig, PrivacyClass, SyncDomain, SyncManifest};

use crate::error::SyncError;
use crate::mcp_server::{SkillEntry, SkillIndex};

/// Split a parametrized domain string (e.g. `"learner-model:did:plc:abc"`)
/// into its family (`"learner-model"`). A domain with no `:` is its own family.
pub fn domain_family(domain: &SyncDomain) -> &str {
    domain.0.split(':').next().unwrap_or(domain.0.as_str())
}

/// Fixed privacy classification per domain family. `None` means unregistered
/// — never syncable, matching `SyncManifest::is_syncable`'s default for an
/// absent domain.
pub fn privacy_for_family(family: &str) -> Option<PrivacyClass> {
    match family {
        "skill-index" => Some(PrivacyClass::Public),
        "learner-model" => Some(PrivacyClass::Trusted),
        // Matches `kbd_sync::domain()`'s existing, tested naming convention
        // (`"kbd-control:<project-id>"`) — not a second, inconsistent
        // control-domain scheme.
        "kbd-control" => Some(PrivacyClass::Trusted),
        // Explicitly registered as Local so `is_syncable` structurally
        // rejects it, matching data-scope.md's policy table — never leaves
        // this device via P2P regardless of what a caller requests.
        "surreal-memory" => Some(PrivacyClass::Local),
        _ => None,
    }
}

/// Idempotently register a concrete domain instance's privacy/prefix into
/// `manifest` on first use, based on its family's fixed classification.
pub fn ensure_registered(manifest: &mut SyncManifest, domain: &SyncDomain) {
    if manifest.config_for(domain).is_some() {
        return;
    }
    if let Some(privacy) = privacy_for_family(domain_family(domain)) {
        let prefix = format!("{}/", domain.0);
        manifest.register(domain.clone(), DomainConfig::new(privacy, prefix));
    }
}

/// Wire envelope broadcast over the P2P gossip layer for one domain push.
/// Carries enough identity to let a receiver reject cross-project/learner
/// payloads before merging anything (see data-scope.md's requirement that a
/// completed protocol "rejects cross-project payloads").
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncEnvelope {
    pub schema_version: String,
    pub domain: String,
    /// Project/learner identity scope. Required (checked by the receiver)
    /// for non-Public domains; `None` is only meaningful for `skill-index`.
    pub identity: Option<String>,
    /// CRDT delta or snapshot bytes, produced by `CrdtEngine::apply_json`'s
    /// delta output or by `CrdtEngine::merge`'s remote-delta input.
    pub payload: Vec<u8>,
    /// Ed25519 signer for `kbd-control` authority pushes — the enrolled
    /// device's `key_id` (see `kbd_runtime::DeviceRecord`). `None` for
    /// families that don't require peer authentication (`skill-index`,
    /// `learner-model`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_key_id: Option<String>,
    /// Base64 Ed25519 signature over `signable_bytes()`, from the signer's
    /// `DeviceSigner::sign_base64`. Verified against the signer's public key
    /// in the receiver's own (already-replicated) `KbdStateV2.devices` —
    /// self-authenticating, no transport-level peer identity required.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl SyncEnvelope {
    /// Deterministic bytes to sign/verify — every field except the signature
    /// itself (`signer_key_id` IS covered, so a signature can't be replayed
    /// under a different claimed signer). Plain length-prefix-free
    /// concatenation with NUL separators is sufficient here: every field is
    /// either a fixed-format string with no embedded NULs in practice
    /// (schema_version, domain, key_id) or the final field (payload), so
    /// there's no ambiguity to exploit.
    pub fn signable_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(
            self.schema_version.len() + self.domain.len() + self.payload.len() + 8,
        );
        bytes.extend_from_slice(self.schema_version.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(self.domain.as_bytes());
        bytes.push(0);
        if let Some(identity) = &self.identity {
            bytes.extend_from_slice(identity.as_bytes());
        }
        bytes.push(0);
        if let Some(signer_key_id) = &self.signer_key_id {
            bytes.extend_from_slice(signer_key_id.as_bytes());
        }
        bytes.push(0);
        bytes.extend_from_slice(&self.payload);
        bytes
    }

    /// Sign this envelope with the given device identity, setting
    /// `signer_key_id`/`signature`.
    pub fn sign(&mut self, signer: &kbd_runtime::DeviceSigner) {
        self.signer_key_id = Some(signer.key_id().to_string());
        self.signature = Some(signer.sign_base64(&self.signable_bytes()));
    }

    /// Verify this envelope's signature against a candidate public key —
    /// the caller is responsible for resolving `signer_key_id` to a public
    /// key (and confirming the device is Active) before calling this.
    pub fn verify(&self, public_key_base64: &str) -> bool {
        let Some(signature) = &self.signature else {
            return false;
        };
        kbd_runtime::verify_ed25519_signature(public_key_base64, &self.signable_bytes(), signature)
    }
}

/// Bridges a syncable domain's real local data to/from plain JSON.
#[async_trait]
pub trait DomainAdapter: Send + Sync {
    /// Current local state as a JSON object, for merging into the domain's
    /// CRDT document ahead of export.
    async fn export_json(&self) -> Result<serde_json::Value, SyncError>;
    /// Persist a merged CRDT document's JSON view back into the real local
    /// store, after importing an incoming delta.
    async fn import_json(&self, value: serde_json::Value) -> Result<(), SyncError>;
}

// ---------------------------------------------------------------------------
// skill-index — Public, self-contained
// ---------------------------------------------------------------------------

pub struct SkillIndexAdapter {
    index: Arc<SkillIndex>,
}

impl SkillIndexAdapter {
    pub fn new(index: Arc<SkillIndex>) -> Self {
        Self { index }
    }
}

#[async_trait]
impl DomainAdapter for SkillIndexAdapter {
    async fn export_json(&self) -> Result<serde_json::Value, SyncError> {
        let mut skills = serde_json::Map::new();
        for entry in self.index.local_entries() {
            skills.insert(
                entry.name.clone(),
                json!({
                    "description": entry.description,
                    "keywords": entry.keywords,
                }),
            );
        }
        Ok(serde_json::Value::Object(skills))
    }

    async fn import_json(&self, value: serde_json::Value) -> Result<(), SyncError> {
        let skills = value.as_object().cloned().unwrap_or_default();
        let mut remote = Vec::with_capacity(skills.len());
        for (name, entry) in skills {
            let description = entry
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let keywords = entry
                .get("keywords")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|k| k.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default();
            remote.push(SkillEntry {
                name: name.clone(),
                description,
                // Remote entries carry only synced metadata (name,
                // description, keywords) — no local source file exists for
                // them, so `path` is a sentinel rather than a real one.
                path: PathBuf::from(format!("remote:{name}")),
                keywords,
            });
        }
        self.index.replace_remote(remote);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// learner-model:<learner-id> — Trusted, cross-crate
// ---------------------------------------------------------------------------

pub struct LearnerModelAdapter {
    store: learner_model::LearnerModelStore<
        storage_provider::LocalDirAdapter,
        storage_provider::LoroAdapter,
    >,
    learner_id: String,
}

impl LearnerModelAdapter {
    pub fn new(base_dir: PathBuf, learner_id: impl Into<String>) -> Self {
        Self {
            store: learner_model::LearnerModelStore::new(
                storage_provider::LocalDirAdapter::new(base_dir),
                storage_provider::LoroAdapter,
            ),
            learner_id: learner_id.into(),
        }
    }
}

#[async_trait]
impl DomainAdapter for LearnerModelAdapter {
    async fn export_json(&self) -> Result<serde_json::Value, SyncError> {
        match self.store.load(&self.learner_id).await {
            Ok(model) => serde_json::to_value(model).map_err(|e| SyncError::Crdt(e.to_string())),
            // No local model yet is not an error for export — an empty
            // object merges as a no-op.
            Err(learner_model::StoreError::NotFound(_)) => Ok(json!({})),
            Err(e) => Err(SyncError::Storage(e.to_string())),
        }
    }

    async fn import_json(&self, value: serde_json::Value) -> Result<(), SyncError> {
        if value.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            return Ok(());
        }
        let model: learner_model::LearnerModel =
            serde_json::from_value(value).map_err(|e| SyncError::Crdt(e.to_string()))?;
        self.store
            .save(&model)
            .await
            .map_err(|e| SyncError::Storage(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// kbd-control:<project-id> — Trusted, signed authoritative Loro updates
// ---------------------------------------------------------------------------
//
// Deliberately NOT a generic `DomainAdapter`: KBD already persists its own
// Loro document with signed event validation and fsync ordering. The REST
// sync path exports/imports that document directly and carries presence only
// as auxiliary metadata.
//
// Peer authentication is real: each push is signed with the sending node's
// own `kbd_runtime::DeviceSigner` (`SyncEnvelope::sign`), and the receiver
// verifies the signature against the claimed signer's public key in its own
// `KbdStateV2.devices` (`SyncEnvelope::verify`), requiring `DeviceStatus::Active`.
// This reuses the same Ed25519 device identity `Event` signing already
// relies on. Transport pairing and endpoint enrollment are enforced separately
// from the signed KBD command authority.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_family_splits_on_first_colon() {
        assert_eq!(
            domain_family(&SyncDomain::new("learner-model:did:plc:abc")),
            "learner-model"
        );
        assert_eq!(
            domain_family(&SyncDomain::new("skill-index")),
            "skill-index"
        );
    }

    #[test]
    fn surreal_memory_is_never_syncable() {
        assert_eq!(
            privacy_for_family("surreal-memory"),
            Some(PrivacyClass::Local)
        );
        let mut manifest = SyncManifest::new();
        let domain = SyncDomain::new("surreal-memory");
        ensure_registered(&mut manifest, &domain);
        assert!(!manifest.is_syncable(&domain));
    }

    #[test]
    fn unknown_domain_families_stay_unregistered() {
        let mut manifest = SyncManifest::new();
        let domain = SyncDomain::new("something-unrecognized");
        ensure_registered(&mut manifest, &domain);
        assert!(manifest.config_for(&domain).is_none());
        assert!(!manifest.is_syncable(&domain));
    }

    #[test]
    fn public_and_trusted_families_become_syncable_on_first_use() {
        let mut manifest = SyncManifest::new();
        let skill_domain = SyncDomain::new("skill-index");
        let learner_domain = SyncDomain::new("learner-model:did:plc:abc");
        ensure_registered(&mut manifest, &skill_domain);
        ensure_registered(&mut manifest, &learner_domain);
        assert!(manifest.is_syncable(&skill_domain));
        assert!(manifest.is_syncable(&learner_domain));
    }
}
