//! Non-authoritative KBD presence.
//!
//! During journal stabilization, canonical workflow events remain in the
//! fsynced runtime journal. This Loro document carries only ephemeral presence;
//! authoritative Loro deltas are introduced by the project-document layer.

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
            format!("kbd-control/{project_id}/presence/"),
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
    fn presence_requires_an_authenticated_peer_and_contains_no_authority() {
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

        let json = serde_json::to_value(presence()).unwrap();
        for forbidden in ["events", "eventId", "command", "projectDocument", "transcript", "prompt"] {
            assert!(json.get(forbidden).is_none());
        }
    }
}
