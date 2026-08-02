//! Signed authoritative KBD project synchronization.
//!
//! The wire payload carries the complete Loro update set for one persisted
//! `project.loro` document plus optional ephemeral presence. Per-replica
//! journals remain local write-ahead logs and are never transmitted.

use loro::{ExportMode, LoroDoc};
use serde::{Deserialize, Serialize};
use storage_provider::{DomainConfig, PrivacyClass, SyncDomain, SyncManifest};

use crate::error::SyncError;

pub fn domain(project_id: &str) -> SyncDomain {
    SyncDomain::new(format!("kbd-control:{project_id}"))
}

pub fn trusted_manifest(project_id: &str) -> SyncManifest {
    let mut manifest = SyncManifest::new();
    manifest.register(
        domain(project_id),
        DomainConfig::new(
            PrivacyClass::Trusted,
            format!("kbd-control/{project_id}/authority/"),
        ),
    );
    manifest
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KbdPresence {
    pub device: String,
    pub harness: String,
    pub session: String,
    pub observed_revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KbdAuthorityPayload {
    pub schema_version: String,
    pub project_id: String,
    pub project_updates: Vec<u8>,
    #[serde(default)]
    pub presence: Vec<KbdPresence>,
}

impl KbdAuthorityPayload {
    pub fn encode(
        project_id: impl Into<String>,
        project_updates: Vec<u8>,
        presence: Vec<KbdPresence>,
    ) -> Result<Vec<u8>, SyncError> {
        serde_json::to_vec(&Self {
            schema_version: "2".into(),
            project_id: project_id.into(),
            project_updates,
            presence,
        })
        .map_err(|error| SyncError::Crdt(error.to_string()))
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, SyncError> {
        let payload: Self =
            serde_json::from_slice(bytes).map_err(|error| SyncError::Crdt(error.to_string()))?;
        if payload.schema_version != "2" || payload.project_id.trim().is_empty() {
            return Err(SyncError::Crdt(
                "KBD authority payload requires schemaVersion 2 and projectId".into(),
            ));
        }
        Ok(payload)
    }
}

pub struct KbdPresenceDocument {
    project_id: String,
    doc: LoroDoc,
}

impl KbdPresenceDocument {
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            doc: LoroDoc::new(),
        }
    }

    pub fn update(&self, presence: &KbdPresence) -> Result<(), SyncError> {
        let key = format!(
            "{}:{}:{}",
            presence.device, presence.harness, presence.session
        );
        let value =
            serde_json::to_string(presence).map_err(|error| SyncError::Crdt(error.to_string()))?;
        self.doc
            .get_map("presence")
            .insert(&key, value)
            .map_err(|error| SyncError::Crdt(error.to_string()))?;
        self.doc.commit();
        Ok(())
    }

    pub fn entries(&self) -> Result<Vec<KbdPresence>, SyncError> {
        let value = serde_json::to_value(self.doc.get_map("presence").get_deep_value())
            .map_err(|error| SyncError::Crdt(error.to_string()))?;
        let mut entries = value
            .as_object()
            .into_iter()
            .flat_map(|object| object.values())
            .filter_map(|entry| entry.as_str())
            .map(serde_json::from_str)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| SyncError::Crdt(error.to_string()))?;
        entries.sort_by(|left: &KbdPresence, right| {
            (&left.device, &left.harness, &left.session).cmp(&(
                &right.device,
                &right.harness,
                &right.session,
            ))
        });
        Ok(entries)
    }

    pub fn export_snapshot(&self) -> Result<Vec<u8>, SyncError> {
        self.doc
            .export(ExportMode::Snapshot)
            .map_err(|error| SyncError::Crdt(error.to_string()))
    }

    /// Import presence only after the authenticated transport has checked the
    /// remote endpoint and signer against committed membership.
    pub fn import_authenticated(
        &self,
        bytes: &[u8],
        peer_authorized: bool,
    ) -> Result<(), SyncError> {
        if !peer_authorized {
            return Err(SyncError::PrivacyViolation(
                domain(&self.project_id).to_string(),
            ));
        }
        self.doc
            .import(bytes)
            .map_err(|error| SyncError::Crdt(error.to_string()))?;
        self.doc.commit();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn presence() -> KbdPresence {
        KbdPresence {
            device: "device-a".into(),
            harness: "codex".into(),
            session: "session-a".into(),
            observed_revision: 12,
        }
    }

    #[test]
    fn kbd_domain_is_trusted_and_project_scoped() {
        let project = "project-a";
        let domain = domain(project);
        let manifest = trusted_manifest(project);
        assert_eq!(domain.to_string(), "kbd-control:project-a");
        assert!(manifest.is_syncable(&domain));
        assert_eq!(
            manifest.config_for(&domain).unwrap().privacy,
            PrivacyClass::Trusted
        );
    }

    #[test]
    fn presence_requires_an_authenticated_peer() {
        let first = KbdPresenceDocument::new("project-a");
        first.update(&presence()).unwrap();
        let snapshot = first.export_snapshot().unwrap();

        let second = KbdPresenceDocument::new("project-a");
        assert!(matches!(
            second.import_authenticated(&snapshot, false),
            Err(SyncError::PrivacyViolation(_))
        ));
        second.import_authenticated(&snapshot, true).unwrap();
        assert_eq!(second.entries().unwrap(), vec![presence()]);
    }

    #[test]
    fn authority_payload_round_trips_loro_updates_and_presence() {
        let encoded =
            KbdAuthorityPayload::encode("project-a", vec![1, 2, 3], vec![presence()]).unwrap();
        let decoded = KbdAuthorityPayload::decode(&encoded).unwrap();
        assert_eq!(decoded.project_id, "project-a");
        assert_eq!(decoded.project_updates, vec![1, 2, 3]);
        assert_eq!(decoded.presence, vec![presence()]);
    }
}
