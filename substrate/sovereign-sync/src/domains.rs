//! Domain adapters bridging real local data (skill index, learner model, KBD
//! presence) to the generic CRDT sync machinery in `crdt.rs`.
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
use crate::kbd_control::KbdControlPlane;
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
        // presence-domain scheme.
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
    store: learner_model::LearnerModelStore<storage_provider::LocalDirAdapter, storage_provider::LoroAdapter>,
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
// kbd-control:<project-id> — Trusted, ephemeral, non-authoritative
// ---------------------------------------------------------------------------
//
// Reuses `kbd_sync::KbdPresence`'s existing, tested schema rather than
// inventing a second shape. Note: `kbd_sync::KbdPresenceDocument` gates
// import on `peer_authorized: bool`, tying presence merge to authenticated
// peer transport — that authentication is not yet wired into the gossip
// layer (see data-scope.md: "no P2P presence wiring"). This adapter is a
// first pass that exports real presence and merges it as any other Trusted
// domain via `handle_incoming_message`'s existing identity check
// (project-id match); real peer authentication is follow-up work before
// this should be considered a hardened transport.

pub struct KbdPresenceAdapter {
    kbd_control: Arc<KbdControlPlane>,
    device_id: String,
}

impl KbdPresenceAdapter {
    pub fn new(kbd_control: Arc<KbdControlPlane>, device_id: impl Into<String>) -> Self {
        Self {
            kbd_control,
            device_id: device_id.into(),
        }
    }
}

#[async_trait]
impl DomainAdapter for KbdPresenceAdapter {
    async fn export_json(&self) -> Result<serde_json::Value, SyncError> {
        let status = self
            .kbd_control
            .status()
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        let presence = crate::kbd_sync::KbdPresence {
            device: self.device_id.clone(),
            harness: "sovereign-sync".to_string(),
            session: "daemon".to_string(),
            observed_revision: status.revision,
            leader_term: None,
            lease_healthy: status.lease.is_some(),
        };
        serde_json::to_value(&presence).map_err(|e| SyncError::Crdt(e.to_string()))
    }

    async fn import_json(&self, _value: serde_json::Value) -> Result<(), SyncError> {
        // Presence is purely the CRDT document itself — the merged state
        // lives only in the in-memory `docs` map. Nothing is ever written
        // back to any authoritative KBD file; the Raft/event-journal
        // authority must never be CRDT-merged (see data-scope.md).
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domain_family_splits_on_first_colon() {
        assert_eq!(
            domain_family(&SyncDomain::new("learner-model:did:plc:abc")),
            "learner-model"
        );
        assert_eq!(domain_family(&SyncDomain::new("skill-index")), "skill-index");
    }

    #[test]
    fn surreal_memory_is_never_syncable() {
        assert_eq!(privacy_for_family("surreal-memory"), Some(PrivacyClass::Local));
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
