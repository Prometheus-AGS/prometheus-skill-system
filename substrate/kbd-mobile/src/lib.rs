//! Filesystem-free KBD replica for embedded mobile hosts.
//!
//! The host owns secure keys and persistence. This crate returns exact bytes
//! to sign, accepts only verified signatures, stores authority in an in-memory
//! Loro document, and exchanges signed deltas over iroh gossip. It intentionally
//! has no Git, adoption, submodule-scan, audit-branch, registry, or database API.

use bytes::Bytes;
use iroh::{endpoint::presets, protocol::Router, Endpoint, EndpointId, SecretKey};
use iroh_gossip::{
    api::{Event as GossipEvent, GossipSender},
    net::Gossip,
    proto::TopicId,
};
use kbd_runtime::{
    prepare_host_signed_event, project_document::fold_project_events, verify_ed25519_signature,
    CommandEnvelope, CommandKind, DeviceStatus, Event, KbdStateV2, RuntimeError,
};
use loro::{ExportMode, LoroDoc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;

const MAX_GOSSIP_MESSAGE_SIZE: usize = 16 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum MobileError {
    #[error("KBD event error: {0}")]
    Runtime(#[from] RuntimeError),
    #[error("invalid mobile authority: {0}")]
    Authority(String),
    #[error("mobile transport error: {0}")]
    Transport(String),
    #[error("mobile capability is unavailable: {0}")]
    Capability(String),
}

pub type Result<T> = std::result::Result<T, MobileError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MobileCapabilities {
    pub signed_events: bool,
    pub claims: bool,
    pub adjudications: bool,
    pub loro_sync: bool,
    pub iroh_sync: bool,
    pub git: bool,
    pub adoption: bool,
    pub submodule_scan: bool,
    pub audit_branch_write: bool,
}

impl Default for MobileCapabilities {
    fn default() -> Self {
        Self {
            signed_events: true,
            claims: true,
            adjudications: true,
            loro_sync: true,
            iroh_sync: true,
            git: false,
            adoption: false,
            submodule_scan: false,
            audit_branch_write: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PreparedMobileEvent {
    pub event: Event,
    pub signing_payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MobileCommit {
    pub event: Event,
    pub state: KbdStateV2,
    pub project_updates: Vec<u8>,
}

pub struct MobileProject {
    project_id: String,
    replica_id: String,
    doc: LoroDoc,
}

impl MobileProject {
    pub fn new(project_id: impl Into<String>, replica_id: impl Into<String>) -> Result<Self> {
        let project_id = project_id.into();
        let replica_id = replica_id.into();
        if project_id.trim().is_empty() || replica_id.trim().is_empty() {
            return Err(MobileError::Authority(
                "projectId and replicaId must not be empty".into(),
            ));
        }
        Ok(Self {
            project_id,
            replica_id,
            doc: LoroDoc::new(),
        })
    }

    pub fn from_events(
        project_id: impl Into<String>,
        replica_id: impl Into<String>,
        events: &[Event],
    ) -> Result<Self> {
        let mut project = Self::new(project_id, replica_id)?;
        for event in events {
            project.insert_event(event)?;
        }
        project.doc.commit();
        project.state()?;
        Ok(project)
    }

    pub fn from_updates(
        project_id: impl Into<String>,
        replica_id: impl Into<String>,
        updates: &[u8],
    ) -> Result<Self> {
        let mut project = Self::new(project_id, replica_id)?;
        project.import_updates(updates)?;
        Ok(project)
    }

    pub fn capabilities(&self) -> MobileCapabilities {
        MobileCapabilities::default()
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub fn replica_id(&self) -> &str {
        &self.replica_id
    }

    pub fn events(&self) -> Result<Vec<Event>> {
        events_from_doc(&self.doc, &self.project_id)
    }

    pub fn state(&self) -> Result<KbdStateV2> {
        Ok(fold_project_events(&self.events()?)?)
    }

    pub fn export_updates(&self) -> Result<Vec<u8>> {
        self.doc
            .export(ExportMode::all_updates())
            .map_err(|error| MobileError::Authority(error.to_string()))
    }

    pub fn prepare_command(
        &self,
        envelope: CommandEnvelope,
        signer_key_id: &str,
        signer_public_key: &str,
    ) -> Result<PreparedMobileEvent> {
        if matches!(
            envelope.command,
            CommandKind::SubmodulePinSet { .. }
                | CommandKind::DeviceEnroll { .. }
                | CommandKind::DeviceRevoke { .. }
                | CommandKind::DeviceRotate { .. }
        ) {
            return Err(MobileError::Capability(
                "mobile replicas cannot scan Git, pin submodules, or administer device membership"
                    .into(),
            ));
        }
        let (event, signing_payload) = prepare_host_signed_event(
            &self.state()?,
            &self.replica_id,
            envelope,
            signer_key_id,
            signer_public_key,
        )?;
        Ok(PreparedMobileEvent {
            event,
            signing_payload,
        })
    }

    pub fn commit_prepared(
        &mut self,
        mut prepared: PreparedMobileEvent,
        host_signature_base64: impl Into<String>,
    ) -> Result<MobileCommit> {
        if prepared.event.project_id != self.project_id
            || prepared.event.replica_id != self.replica_id
        {
            return Err(MobileError::Authority(
                "prepared event targets a different project or replica".into(),
            ));
        }
        let expected_payload = prepared.event.prepare_host_signature(
            prepared
                .event
                .signer_key_id
                .clone()
                .ok_or_else(|| MobileError::Authority("missing signerKeyId".into()))?,
            prepared
                .event
                .signer_public_key
                .clone()
                .ok_or_else(|| MobileError::Authority("missing signerPublicKey".into()))?,
        )?;
        if expected_payload != prepared.signing_payload {
            return Err(MobileError::Authority(
                "prepared event signing payload was modified".into(),
            ));
        }
        prepared
            .event
            .attach_host_signature(host_signature_base64)?;
        let mut candidate = clone_doc(&self.doc)?;
        insert_event_into(&mut candidate, &prepared.event)?;
        candidate.commit();
        let events = events_from_doc(&candidate, &self.project_id)?;
        let state = fold_project_events(&events)?;
        self.doc = candidate;
        Ok(MobileCommit {
            event: prepared.event,
            state,
            project_updates: self.export_updates()?,
        })
    }

    pub fn import_updates(&mut self, updates: &[u8]) -> Result<KbdStateV2> {
        let before = self
            .events()?
            .into_iter()
            .map(|event| (event.event_id.clone(), event))
            .collect::<BTreeMap<_, _>>();
        let candidate = clone_doc(&self.doc)?;
        candidate
            .import(updates)
            .map_err(|error| MobileError::Authority(error.to_string()))?;
        candidate.commit();
        let events = events_from_doc(&candidate, &self.project_id)?;
        let after = events
            .iter()
            .map(|event| (event.event_id.clone(), event))
            .collect::<BTreeMap<_, _>>();
        if before
            .iter()
            .any(|(event_id, event)| after.get(event_id).copied() != Some(event))
        {
            return Err(MobileError::Authority(
                "Loro update attempted to mutate or delete a committed event".into(),
            ));
        }
        let state = fold_project_events(&events)?;
        self.doc = candidate;
        Ok(state)
    }

    pub fn prepare_signed_delta(
        &self,
        signer_key_id: impl Into<String>,
    ) -> Result<PreparedMobileDelta> {
        let signer_key_id = signer_key_id.into();
        let state = self.state()?;
        if !state
            .devices
            .get(&signer_key_id)
            .is_some_and(|device| device.status == DeviceStatus::Active)
        {
            return Err(MobileError::Authority(
                "delta signer is not an active enrolled device".into(),
            ));
        }
        let authority = MobileAuthorityPayload {
            schema_version: "2".into(),
            project_id: self.project_id.clone(),
            project_updates: self.export_updates()?,
            presence: Vec::new(),
        };
        let delta = MobileSignedDelta {
            schema_version: "2".into(),
            domain: format!("kbd-control:{}", self.project_id),
            identity: Some(self.project_id.clone()),
            payload: serde_json::to_vec(&authority)
                .map_err(|error| MobileError::Authority(error.to_string()))?,
            signer_key_id: Some(signer_key_id),
            signature: None,
        };
        let signing_payload = delta.signable_bytes();
        Ok(PreparedMobileDelta {
            delta,
            signing_payload,
        })
    }

    pub fn import_signed_delta(&mut self, encoded: &[u8]) -> Result<KbdStateV2> {
        let delta: MobileSignedDelta = serde_json::from_slice(encoded)
            .map_err(|error| MobileError::Authority(error.to_string()))?;
        if delta.schema_version != "2"
            || delta.domain != format!("kbd-control:{}", self.project_id)
            || delta.identity.as_deref() != Some(self.project_id.as_str())
        {
            return Err(MobileError::Authority(
                "signed delta targets an unsupported schema or different project".into(),
            ));
        }
        let state = self.state()?;
        let signer_key_id = delta
            .signer_key_id
            .as_deref()
            .ok_or_else(|| MobileError::Authority("signed delta omitted signerKeyId".into()))?;
        let device = state.devices.get(signer_key_id).ok_or_else(|| {
            MobileError::Authority("signed delta came from an unknown device".into())
        })?;
        if device.status != DeviceStatus::Active {
            return Err(MobileError::Authority(
                "signed delta came from a revoked device".into(),
            ));
        }
        delta.verify(&device.public_key)?;
        let authority: MobileAuthorityPayload = serde_json::from_slice(&delta.payload)
            .map_err(|error| MobileError::Authority(error.to_string()))?;
        if authority.schema_version != "2" || authority.project_id != self.project_id {
            return Err(MobileError::Authority(
                "authority payload targets a different project or schema".into(),
            ));
        }
        self.import_updates(&authority.project_updates)
    }

    fn insert_event(&mut self, event: &Event) -> Result<()> {
        insert_event_into(&mut self.doc, event)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MobileSignedDelta {
    pub schema_version: String,
    pub domain: String,
    pub identity: Option<String>,
    pub payload: Vec<u8>,
    pub signer_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl MobileSignedDelta {
    fn signable_bytes(&self) -> Vec<u8> {
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

    pub fn signable_bytes_for_host(&self) -> Vec<u8> {
        self.signable_bytes()
    }

    pub fn attach_host_signature(
        &mut self,
        signer_public_key: &str,
        signature: impl Into<String>,
    ) -> Result<()> {
        let signature = signature.into();
        if !verify_ed25519_signature(signer_public_key, &self.signable_bytes(), &signature) {
            return Err(MobileError::Authority(
                "host-supplied delta signature is invalid".into(),
            ));
        }
        self.signature = Some(signature);
        Ok(())
    }

    pub fn verify(&self, signer_public_key: &str) -> Result<()> {
        let signature = self
            .signature
            .as_deref()
            .ok_or_else(|| MobileError::Authority("signed delta omitted signature".into()))?;
        if !verify_ed25519_signature(signer_public_key, &self.signable_bytes(), signature) {
            return Err(MobileError::Authority(
                "signed delta signature is invalid".into(),
            ));
        }
        Ok(())
    }

    pub fn encode(&self) -> Result<Vec<u8>> {
        if self.signature.is_none() {
            return Err(MobileError::Authority(
                "signed delta omitted signature".into(),
            ));
        }
        serde_json::to_vec(self).map_err(|error| MobileError::Authority(error.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PreparedMobileDelta {
    pub delta: MobileSignedDelta,
    pub signing_payload: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct MobileAuthorityPayload {
    schema_version: String,
    project_id: String,
    project_updates: Vec<u8>,
    #[serde(default)]
    presence: Vec<serde_json::Value>,
}

fn insert_event_into(doc: &mut LoroDoc, event: &Event) -> Result<()> {
    let map = doc.get_map("events");
    let existing = map.get(&event.event_id).and_then(|value| {
        serde_json::to_value(value.get_deep_value())
            .ok()?
            .as_str()
            .map(str::to_owned)
    });
    let canonical = if event.schema_version == "1" {
        serde_json::to_string(event).map_err(|error| MobileError::Authority(error.to_string()))?
    } else {
        let bytes =
            serde_jcs::to_vec(event).map_err(|error| MobileError::Authority(error.to_string()))?;
        String::from_utf8(bytes).map_err(|error| MobileError::Authority(error.to_string()))?
    };
    if let Some(existing) = existing {
        if existing != canonical {
            return Err(MobileError::Authority(format!(
                "event id {} already contains different bytes",
                event.event_id
            )));
        }
        return Ok(());
    }
    map.insert(&event.event_id, canonical)
        .map_err(|error| MobileError::Authority(error.to_string()))?;
    Ok(())
}

fn events_from_doc(doc: &LoroDoc, project_id: &str) -> Result<Vec<Event>> {
    let value = serde_json::to_value(doc.get_map("events").get_deep_value())
        .map_err(|error| MobileError::Authority(error.to_string()))?;
    let mut events = value
        .as_object()
        .into_iter()
        .flat_map(|object| object.values())
        .filter_map(|entry| entry.as_str())
        .map(serde_json::from_str::<Event>)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|error| MobileError::Authority(error.to_string()))?;
    if events.iter().any(|event| event.project_id != project_id) {
        return Err(MobileError::Authority(
            "Loro document contains an event for another project".into(),
        ));
    }
    events.sort_by(|left, right| {
        (
            left.frontier.derived_revision().saturating_add(1),
            left.lamport,
            &left.replica_id,
            &left.event_id,
        )
            .cmp(&(
                right.frontier.derived_revision().saturating_add(1),
                right.lamport,
                &right.replica_id,
                &right.event_id,
            ))
    });
    Ok(events)
}

fn clone_doc(doc: &LoroDoc) -> Result<LoroDoc> {
    let cloned = LoroDoc::new();
    let snapshot = doc
        .export(ExportMode::Snapshot)
        .map_err(|error| MobileError::Authority(error.to_string()))?;
    cloned
        .import(&snapshot)
        .map_err(|error| MobileError::Authority(error.to_string()))?;
    Ok(cloned)
}

#[derive(Debug, Clone)]
pub struct MobilePeerMessage {
    pub from: EndpointId,
    pub payload: Bytes,
}

pub struct MobilePeer {
    endpoint: Endpoint,
    router: Router,
    gossip: Gossip,
    topic: TopicId,
    sender: Arc<Mutex<Option<GossipSender>>>,
    message_tx: mpsc::Sender<MobilePeerMessage>,
}

impl MobilePeer {
    pub async fn bind(
        operator_id: &[u8; 32],
        host_endpoint_secret: [u8; 32],
    ) -> Result<(Self, mpsc::Receiver<MobilePeerMessage>)> {
        let endpoint = Endpoint::builder(presets::N0)
            .secret_key(SecretKey::from_bytes(&host_endpoint_secret))
            .bind()
            .await
            .map_err(|error| MobileError::Transport(error.to_string()))?;
        Ok(Self::from_endpoint(operator_id, endpoint))
    }

    #[cfg(test)]
    async fn bind_for_test(
        operator_id: &[u8; 32],
        lookup: iroh::address_lookup::MemoryLookup,
    ) -> Result<(Self, mpsc::Receiver<MobilePeerMessage>)> {
        let endpoint = Endpoint::builder(presets::Minimal)
            .bind_addr(std::net::SocketAddr::from(([127, 0, 0, 1], 0)))
            .map_err(|error| MobileError::Transport(error.to_string()))?
            .address_lookup(lookup.clone())
            .bind()
            .await
            .map_err(|error| MobileError::Transport(error.to_string()))?;
        lookup.add_endpoint_info(endpoint.addr());
        Ok(Self::from_endpoint(operator_id, endpoint))
    }

    fn from_endpoint(
        operator_id: &[u8; 32],
        endpoint: Endpoint,
    ) -> (Self, mpsc::Receiver<MobilePeerMessage>) {
        let topic = Self::derive_topic(operator_id);
        let gossip = Gossip::builder()
            .max_message_size(MAX_GOSSIP_MESSAGE_SIZE)
            .spawn(endpoint.clone());
        let router = Router::builder(endpoint.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();
        let (message_tx, message_rx) = mpsc::channel(64);
        (
            Self {
                endpoint,
                router,
                gossip,
                topic,
                sender: Arc::new(Mutex::new(None)),
                message_tx,
            },
            message_rx,
        )
    }

    pub fn derive_topic(operator_id: &[u8; 32]) -> TopicId {
        let mut input = operator_id.to_vec();
        input.extend_from_slice(b"sovereign-sync-v1");
        let hash = *blake3::hash(&input).as_bytes();
        TopicId::from_bytes(hash)
    }

    pub fn endpoint_id(&self) -> EndpointId {
        self.endpoint.id()
    }

    pub async fn start(&self, peers: Vec<EndpointId>) -> Result<()> {
        let topic = self
            .gossip
            .subscribe(self.topic, peers)
            .await
            .map_err(|error| MobileError::Transport(error.to_string()))?;
        let (sender, mut receiver) = topic.split();
        *self.sender.lock().await = Some(sender);
        let tx = self.message_tx.clone();
        tokio::spawn(async move {
            while let Some(Ok(event)) = receiver.next().await {
                if let GossipEvent::Received(message) = event {
                    if tx
                        .send(MobilePeerMessage {
                            from: message.delivered_from,
                            payload: message.content,
                        })
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        });
        Ok(())
    }

    pub async fn broadcast(&self, payload: Vec<u8>) -> Result<()> {
        if payload.len() > MAX_GOSSIP_MESSAGE_SIZE {
            return Err(MobileError::Transport(format!(
                "payload exceeds {MAX_GOSSIP_MESSAGE_SIZE} bytes"
            )));
        }
        self.sender
            .lock()
            .await
            .as_ref()
            .ok_or_else(|| MobileError::Transport("peer is not started".into()))?
            .broadcast(Bytes::from(payload))
            .await
            .map_err(|error| MobileError::Transport(error.to_string()))
    }

    pub async fn shutdown(self) -> Result<()> {
        self.router
            .shutdown()
            .await
            .map_err(|error| MobileError::Transport(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iroh::address_lookup::MemoryLookup;
    use kbd_runtime::{Actor, ActorKind, ClaimMode, DeviceSigner, Runtime, SubmodulePin};
    use std::time::Duration;
    use uuid::Uuid;

    fn actor() -> Actor {
        Actor {
            kind: ActorKind::Harness,
            id: "mobile-holder".into(),
            device: "mobile-device".into(),
            harness: "mobile".into(),
            session: "mobile-session".into(),
        }
    }

    #[tokio::test]
    async fn simulated_mobile_peer_signs_claim_syncs_over_iroh_and_has_no_git_capabilities() {
        let fixture = tempfile::tempdir().unwrap();
        let project_id = Uuid::new_v4().to_string();
        let desktop = Runtime::open(fixture.path());
        let initialized = desktop
            .initialize(
                &project_id,
                "run-a",
                Actor::operator("operator", "mobile-test"),
            )
            .unwrap();
        let signer: DeviceSigner = desktop.device_signer().unwrap();
        let events = desktop.events().unwrap();
        let mut first = MobileProject::from_events(&project_id, "mobile-a", &events).unwrap();
        let mut second = MobileProject::from_events(&project_id, "mobile-b", &events).unwrap();
        let capabilities = first.capabilities();
        assert!(capabilities.signed_events && capabilities.claims && capabilities.iroh_sync);
        assert!(
            !capabilities.git
                && !capabilities.adoption
                && !capabilities.submodule_scan
                && !capabilities.audit_branch_write
        );

        let actor = actor();
        let prepared = first
            .prepare_command(
                CommandEnvelope {
                    schema_version: "2".into(),
                    project_id: project_id.clone(),
                    run_id: initialized.run_id.clone(),
                    command_id: "mobile-claim".into(),
                    frontier: Some(initialized.frontier.clone()),
                    expected_revision: 0,
                    actor: actor.clone(),
                    command: CommandKind::ClaimAcquire {
                        scope: "phase:mobile".into(),
                        mode: ClaimMode::Exclusive,
                        ttl_seconds: 300,
                        holder_id: actor.id.clone(),
                    },
                },
                signer.key_id(),
                signer.public_key(),
            )
            .unwrap();
        let signature = signer.sign_base64(&prepared.signing_payload);
        let committed = first.commit_prepared(prepared, signature).unwrap();
        assert_eq!(committed.state.claims.len(), 1);

        let rejected = first.prepare_command(
            CommandEnvelope {
                schema_version: "2".into(),
                project_id: project_id.clone(),
                run_id: initialized.run_id,
                command_id: "mobile-submodule".into(),
                frontier: Some(committed.state.frontier.clone()),
                expected_revision: 0,
                actor,
                command: CommandKind::SubmodulePinSet {
                    pin: SubmodulePin {
                        path: "child".into(),
                        child_project_id: Uuid::new_v4().to_string(),
                        gitlink_sha: "a".repeat(40),
                    },
                },
            },
            signer.key_id(),
            signer.public_key(),
        );
        assert!(matches!(rejected, Err(MobileError::Capability(_))));

        let mut delta = first.prepare_signed_delta(signer.key_id()).unwrap();
        let delta_signature = signer.sign_base64(&delta.signing_payload);
        delta
            .delta
            .attach_host_signature(signer.public_key(), delta_signature)
            .unwrap();
        let encoded = delta.delta.encode().unwrap();

        let lookup = MemoryLookup::new();
        let operator_id = [42; 32];
        let (peer_a, _incoming_a) = MobilePeer::bind_for_test(&operator_id, lookup.clone())
            .await
            .unwrap();
        let (peer_b, mut incoming_b) = MobilePeer::bind_for_test(&operator_id, lookup)
            .await
            .unwrap();
        peer_a.start(Vec::new()).await.unwrap();
        peer_b.start(vec![peer_a.endpoint_id()]).await.unwrap();
        tokio::time::sleep(Duration::from_millis(300)).await;
        peer_a.broadcast(encoded).await.unwrap();
        let received = tokio::time::timeout(Duration::from_secs(5), incoming_b.recv())
            .await
            .unwrap()
            .unwrap();
        let converged = second.import_signed_delta(&received.payload).unwrap();
        assert!(converged
            .claims
            .values()
            .any(|claim| claim.scope == "phase:mobile"));
        peer_a.shutdown().await.unwrap();
        peer_b.shutdown().await.unwrap();
    }
}
