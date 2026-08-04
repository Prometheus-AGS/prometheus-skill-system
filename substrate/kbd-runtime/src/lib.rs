use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use fs2::FileExt;
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use thiserror::Error;
use uuid::Uuid;

use project_document::ProjectDocument;

pub mod project_document;
pub mod registry;
pub mod rollout;

#[cfg(test)]
mod live_certification_proof;
#[cfg(test)]
mod live_migration_proof;

pub const EVENT_SCHEMA_VERSION: &str = "2";
pub const AUDIT_GIT_REF: &str = "refs/heads/audit/kbd";

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid event JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("runtime has not been initialized")]
    NotInitialized,
    #[error("runtime is already initialized")]
    AlreadyInitialized,
    #[error("expected revision {expected}, current revision is {actual}")]
    RevisionConflict { expected: u64, actual: u64 },
    #[error("causal frontier conflict: supplied {supplied:?}, current {current:?}")]
    FrontierConflict {
        supplied: CausalFrontier,
        current: CausalFrontier,
    },
    #[error("event integrity check failed at revision {revision}")]
    Integrity { revision: u64 },
    #[error("event signature check failed at revision {revision}: {reason}")]
    Signature { revision: u64, reason: String },
    #[error("event signer {0} is not enrolled")]
    UnknownSigner(String),
    #[error("event signer {0} has been revoked")]
    RevokedSigner(String),
    #[error("event chain is broken at revision {revision}")]
    CausalChain { revision: u64 },
    #[error("duplicate event id {0}")]
    DuplicateEvent(String),
    #[error("duplicate command id {0}")]
    DuplicateCommand(String),
    #[error("invalid lifecycle transition from {from:?} to {to:?}")]
    InvalidTransition {
        from: LifecycleState,
        to: LifecycleState,
    },
    #[error("plan revision mismatch: supplied {supplied}, current {current}")]
    PlanRevision { supplied: u64, current: u64 },
    #[error("command targets project {supplied}, runtime project is {current}")]
    ProjectMismatch { supplied: String, current: String },
    #[error("command targets run {supplied}, runtime run is {current}")]
    RunMismatch { supplied: String, current: String },
    #[error("a non-empty reason is required")]
    ReasonRequired,
    #[error("invalid compatibility state: {0}")]
    InvalidState(String),
    #[error("{kind} {id} was not found")]
    WorkItemNotFound { kind: &'static str, id: String },
    #[error("{kind} {id} already exists")]
    WorkItemExists { kind: &'static str, id: String },
    #[error("work item {item_id} cannot transition from {from:?} to {to:?}")]
    InvalidWorkTransition {
        item_id: String,
        from: WorkStatus,
        to: WorkStatus,
    },
    #[error("claim scope {scope} is held by {holder_id}; refresh frontier {frontier:?} and rebase before retrying")]
    ClaimBlocked {
        scope: String,
        holder_id: String,
        frontier: CausalFrontier,
    },
    #[error("replica {replica_id} is read-only: {reason}")]
    ReplicaReadOnly { replica_id: String, reason: String },
    #[error("replica must halt writes intersecting {scope}; winner {winner_event_id}, frontier {frontier:?}; rebase manually before retrying")]
    ReplicaRebaseRequired {
        scope: String,
        winner_event_id: String,
        frontier: CausalFrontier,
    },
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitAuditExport {
    pub ref_name: String,
    pub tree_path: String,
    pub commit_id: String,
    pub event_count: u64,
    pub sha256: String,
    pub unchanged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LifecycleState {
    Ready,
    Running,
    PauseRequested,
    Paused,
    Blocked,
    Completed,
    Cancelled,
    Failed,
}

impl LifecycleState {
    pub fn is_suspended(&self) -> bool {
        matches!(self, Self::PauseRequested | Self::Paused | Self::Blocked)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Failed)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    Operator,
    Harness,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    Active,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRecord {
    pub device_id: String,
    pub key_id: String,
    pub public_key: String,
    pub status: DeviceStatus,
    pub enrolled_at_revision: u64,
    pub revoked_at_revision: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct DeviceSigner {
    key_id: String,
    public_key: String,
    signing_key: SigningKey,
}

impl DeviceSigner {
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        Self::from_signing_key(signing_key)
    }

    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        Self::from_signing_key(SigningKey::from_bytes(bytes))
    }

    fn from_signing_key(signing_key: SigningKey) -> Self {
        let public = signing_key.verifying_key().to_bytes();
        Self {
            key_id: format!("ed25519:{:x}", Sha256::digest(public)),
            public_key: BASE64.encode(public),
            signing_key,
        }
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }

    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    /// Sign arbitrary bytes with this device's Ed25519 key, base64-encoded.
    /// Exposed for callers outside the committed-event log (e.g. signing a
    /// P2P gossip envelope) that want the same device identity `Event`
    /// signing already uses, without duplicating key material handling.
    pub fn sign_base64(&self, message: &[u8]) -> String {
        BASE64.encode(self.signing_key.sign(message).to_bytes())
    }
}

/// Verify an Ed25519 signature against a base64-encoded public key, for
/// callers that only have a candidate `signer_key_id`/public key (e.g. from
/// `DeviceRecord`) and message bytes — not a full `Event` to run
/// `Event::verify_signature`'s revocation-aware path over.
pub fn verify_ed25519_signature(
    public_key_base64: &str,
    message: &[u8],
    signature_base64: &str,
) -> bool {
    let Ok(public_bytes) = BASE64.decode(public_key_base64) else {
        return false;
    };
    let Ok(public_array) = <[u8; 32]>::try_from(public_bytes.as_slice()) else {
        return false;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&public_array) else {
        return false;
    };
    let Ok(signature_bytes) = BASE64.decode(signature_base64) else {
        return false;
    };
    let Ok(signature) = Signature::from_slice(&signature_bytes) else {
        return false;
    };
    verifying_key.verify(message, &signature).is_ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredDeviceKey {
    schema_version: String,
    key_id: String,
    private_key: String,
}

fn stored_device_key(signer: &DeviceSigner) -> StoredDeviceKey {
    StoredDeviceKey {
        schema_version: "1".into(),
        key_id: signer.key_id.clone(),
        private_key: BASE64.encode(signer.signing_key.to_bytes()),
    }
}

fn signer_from_stored(stored: StoredDeviceKey) -> Result<DeviceSigner> {
    if stored.schema_version != "1" {
        return Err(RuntimeError::InvalidState(format!(
            "unsupported stored device key schema {}",
            stored.schema_version
        )));
    }
    let bytes = BASE64
        .decode(stored.private_key)
        .map_err(|error| RuntimeError::InvalidState(error.to_string()))?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| {
        RuntimeError::InvalidState("stored Ed25519 key must contain 32 bytes".into())
    })?;
    let signer = DeviceSigner::from_bytes(&bytes);
    if signer.key_id != stored.key_id {
        return Err(RuntimeError::InvalidState(
            "stored device key id does not match the private key".into(),
        ));
    }
    Ok(signer)
}

fn load_device_key(path: &Path) -> Result<DeviceSigner> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(RuntimeError::InvalidState(format!(
            "{} must be a regular, non-symlink device key file",
            path.display()
        )));
    }
    #[cfg(unix)]
    if metadata.permissions().mode() & 0o077 != 0 {
        return Err(RuntimeError::InvalidState(format!(
            "{} must not be readable, writable, or executable by group or other users (use mode 0600)",
            path.display()
        )));
    }
    signer_from_stored(serde_json::from_reader(File::open(path)?)?)
}

pub fn ensure_device_key_file(path: &Path) -> Result<DeviceSigner> {
    if path.exists() {
        return load_device_key(path);
    }
    let parent = path.parent().ok_or_else(|| {
        RuntimeError::InvalidState(format!("{} has no parent directory", path.display()))
    })?;
    fs::create_dir_all(parent)?;
    let signer = DeviceSigner::generate();
    let stored = stored_device_key(&signer);
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    match options.open(path) {
        Ok(mut file) => {
            serde_json::to_writer_pretty(&mut file, &stored)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
            File::open(parent)?.sync_all()?;
            Ok(signer)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => load_device_key(path),
        Err(error) => Err(error.into()),
    }
}

fn is_zero(value: &u64) -> bool {
    *value == 0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Actor {
    pub kind: ActorKind,
    pub id: String,
    pub device: String,
    pub harness: String,
    pub session: String,
}

impl Actor {
    pub fn operator(id: impl Into<String>, harness: impl Into<String>) -> Self {
        Self {
            kind: ActorKind::Operator,
            id: id.into(),
            device: device_identity(),
            harness: harness.into(),
            session: std::env::var("CODEX_THREAD_ID")
                .or_else(|_| std::env::var("CLAUDE_SESSION_ID"))
                .unwrap_or_else(|_| "unknown".into()),
        }
    }
}

fn device_identity() -> String {
    std::env::var("PROMETHEUS_DEVICE_ID")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown-device".into())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Checkpoint {
    pub reason: String,
    pub previous_state: LifecycleState,
    pub last_completed: Option<String>,
    pub exact_next_work: Option<String>,
    pub decisions: Vec<String>,
    pub blockers: Vec<String>,
    pub dirty_work_summary: Option<String>,
    pub plan_revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Copy, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CompletionDimension {
    Implementation,
    Evidence,
    Certification,
    Publication,
}

impl CompletionDimension {
    fn all() -> [Self; 4] {
        [
            Self::Implementation,
            Self::Evidence,
            Self::Certification,
            Self::Publication,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkStatus {
    Pending,
    InProgress,
    Blocked,
    Complete,
    Cancelled,
}

impl WorkStatus {
    fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Completion {
    pub completed: u64,
    pub total: u64,
    pub status: WorkStatus,
    pub summary: Option<String>,
    pub blockers: Vec<String>,
}

impl Completion {
    fn not_tracked() -> Self {
        Self {
            completed: 0,
            total: 0,
            status: WorkStatus::Pending,
            summary: None,
            blockers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub sequence: u64,
    pub status: WorkStatus,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub sequence: u64,
    pub status: WorkStatus,
    pub implementation_status: WorkStatus,
    pub tasks: BTreeMap<String, Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Stage {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub sequence: u64,
    pub status: WorkStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Phase {
    pub id: String,
    pub slug: String,
    pub title: String,
    #[serde(default)]
    pub parent_phase_id: Option<String>,
    pub status: WorkStatus,
    pub stages: BTreeMap<String, Stage>,
    pub changes: BTreeMap<String, Change>,
    pub legacy_read_only: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActivePath {
    #[serde(default)]
    pub phase_path: Vec<String>,
    pub phase_id: Option<String>,
    pub stage_id: Option<String>,
    pub change_id: Option<String>,
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SubmodulePin {
    pub path: String,
    pub child_project_id: String,
    pub gitlink_sha: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReplicaCommitStatus {
    Current,
    AheadOfMe,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubmoduleChildStatus {
    Current,
    AheadOfParent,
    Diverged,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaView {
    pub replica_id: String,
    pub local_head: Option<String>,
    pub active_path_status: ReplicaCommitStatus,
    pub submodules: BTreeMap<String, SubmoduleChildStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Decision {
    pub id: String,
    pub summary: String,
    pub plan_revision: u64,
    pub supersedes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Blocker {
    pub id: String,
    pub summary: String,
    pub resolved: bool,
    pub resolution: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaimMode {
    Shared,
    Exclusive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClaimRecord {
    pub claim_id: String,
    pub scope: String,
    pub replica_id: String,
    pub holder_id: String,
    pub mode: ClaimMode,
    pub expires_at: DateTime<Utc>,
    pub monotonic_token: u64,
    pub acquired_event_id: String,
    pub last_event_id: String,
    pub released: bool,
}

impl ClaimRecord {
    pub fn active_at(&self, now: DateTime<Utc>) -> bool {
        !self.released && self.expires_at > now
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictKind {
    Phase,
    Lifecycle,
    ActivePath,
    Completion,
    Decision,
    Blocker,
    Claim,
    SubmodulePin,
    Fold,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConflictCandidate {
    pub event_id: String,
    pub replica_id: String,
    pub lamport: u64,
    pub actor_id: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConflictRecord {
    pub id: String,
    pub slot: String,
    pub kind: ConflictKind,
    pub candidates: Vec<ConflictCandidate>,
    pub winner_event_id: String,
    pub resolved_by_event_id: Option<String>,
    pub resolution_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum EventKind {
    RunInitialized {
        initial_state: LifecycleState,
        exact_next_work: Option<String>,
        plan_revision: u64,
    },
    LifecycleTransition {
        from: LifecycleState,
        to: LifecycleState,
        reason: String,
    },
    CheckpointCreated {
        checkpoint: Checkpoint,
    },
    PauseCheckpointed {
        checkpoint: Checkpoint,
    },
    PlanRevised {
        from_revision: u64,
        to_revision: u64,
        reason: String,
        superseded_next_work: Option<String>,
        exact_next_work: Option<String>,
    },
    LegacyStateImported {
        phases: BTreeMap<String, Phase>,
        active_path: ActivePath,
        completion: BTreeMap<CompletionDimension, Completion>,
        decisions: BTreeMap<String, Decision>,
        blockers: BTreeMap<String, Blocker>,
    },
    PhaseDefined {
        phase: Phase,
    },
    PhaseTransitioned {
        phase_id: String,
        from: WorkStatus,
        to: WorkStatus,
    },
    StageEntered {
        phase_id: String,
        stage: Stage,
    },
    StageTransitioned {
        phase_id: String,
        stage_id: String,
        from: WorkStatus,
        to: WorkStatus,
    },
    ChangeRegistered {
        phase_id: String,
        change: Change,
    },
    ChangeTransitioned {
        phase_id: String,
        change_id: String,
        from: WorkStatus,
        to: WorkStatus,
    },
    TaskRegistered {
        phase_id: String,
        change_id: String,
        task: Task,
    },
    TaskTransitioned {
        phase_id: String,
        change_id: String,
        task_id: String,
        from: WorkStatus,
        to: WorkStatus,
        summary: Option<String>,
    },
    ActivePathChanged {
        active_path: ActivePath,
        exact_next_work: Option<String>,
    },
    CompletionUpdated {
        dimension: CompletionDimension,
        completion: Completion,
    },
    DecisionRecorded {
        decision: Decision,
    },
    BlockerRecorded {
        blocker: Blocker,
    },
    BlockerCleared {
        blocker_id: String,
        resolution: String,
    },
    ClaimAcquired {
        claim_id: String,
        scope: String,
        holder_id: String,
        mode: ClaimMode,
        expires_at: DateTime<Utc>,
        monotonic_token: u64,
    },
    ClaimRenewed {
        claim_id: String,
        expires_at: DateTime<Utc>,
        monotonic_token: u64,
    },
    ClaimReleased {
        claim_id: String,
        monotonic_token: u64,
    },
    SubmodulePinRecorded {
        pin: SubmodulePin,
    },
    DeviceEnrolled {
        device: DeviceRecord,
    },
    DeviceRevoked {
        key_id: String,
        reason: String,
    },
    DeviceKeyRotated {
        previous_key_id: String,
        replacement: DeviceRecord,
    },
    ConflictRecorded {
        conflict: ConflictRecord,
    },
    ConflictResolved {
        conflict_id: String,
        winner_event_id: String,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[serde(transparent)]
pub struct CausalFrontier(pub BTreeMap<String, u64>);

impl CausalFrontier {
    pub fn empty() -> Self {
        Self(BTreeMap::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn lamport(&self, replica_id: &str) -> u64 {
        self.0.get(replica_id).copied().unwrap_or(0)
    }

    pub fn next_lamport(&self, replica_id: &str) -> u64 {
        self.lamport(replica_id).saturating_add(1)
    }

    pub fn advance(&mut self, replica_id: impl Into<String>, lamport: u64) {
        let replica_id = replica_id.into();
        let current = self.0.entry(replica_id).or_default();
        *current = (*current).max(lamport);
    }

    pub fn dominates(&self, other: &Self) -> bool {
        other
            .0
            .iter()
            .all(|(replica_id, lamport)| self.lamport(replica_id) >= *lamport)
    }

    pub fn contains_event(&self, event: &Event) -> bool {
        !event.replica_id.is_empty() && self.lamport(&event.replica_id) >= event.lamport
    }

    pub fn derived_revision(&self) -> u64 {
        self.0.values().copied().sum()
    }
}

impl Default for CausalFrontier {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaHead {
    pub event_id: String,
    pub integrity_hash: String,
    pub lamport: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub schema_version: String,
    pub project_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub replica_id: String,
    pub run_id: String,
    pub event_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
    pub revision: u64,
    pub expected_revision: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub lamport: u64,
    #[serde(default, skip_serializing_if = "CausalFrontier::is_empty")]
    pub frontier: CausalFrontier,
    pub causal_parent: Option<String>,
    pub actor: Actor,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub actor_id: String,
    pub timestamp: DateTime<Utc>,
    pub kind: EventKind,
    pub previous_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub migration_provenance: Option<MigrationProvenance>,
    pub integrity_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_public_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MigrationProvenance {
    pub source_project_id: String,
    pub source_replica_id: Option<String>,
    pub source_event_id: String,
    pub source_integrity_hash: String,
    pub source_previous_hash: Option<String>,
}

impl Event {
    fn canonical_unsigned_bytes(&self) -> Result<Vec<u8>> {
        let mut unsigned = self.clone();
        unsigned.integrity_hash.clear();
        unsigned.signature = None;
        if self.schema_version == "1" {
            Ok(serde_json::to_vec(&unsigned)?)
        } else {
            serde_jcs::to_vec(&unsigned)
                .map_err(|error| RuntimeError::InvalidState(error.to_string()))
        }
    }

    fn calculate_hash(&self) -> Result<String> {
        let bytes = self.canonical_unsigned_bytes()?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }

    pub fn prepare_host_signature(
        &mut self,
        signer_key_id: impl Into<String>,
        signer_public_key: impl Into<String>,
    ) -> Result<Vec<u8>> {
        self.signer_key_id = Some(signer_key_id.into());
        self.signer_public_key = Some(signer_public_key.into());
        self.signature = None;
        let bytes = self.canonical_unsigned_bytes()?;
        self.integrity_hash = self.calculate_hash()?;
        Ok(bytes)
    }

    pub fn attach_host_signature(&mut self, signature_base64: impl Into<String>) -> Result<()> {
        let signature = signature_base64.into();
        let key_id = self
            .signer_key_id
            .as_deref()
            .ok_or_else(|| RuntimeError::InvalidState("event signerKeyId is missing".into()))?;
        let public_key = self
            .signer_public_key
            .as_deref()
            .ok_or_else(|| RuntimeError::InvalidState("event signerPublicKey is missing".into()))?;
        let bytes = self.canonical_unsigned_bytes()?;
        if self.integrity_hash != self.calculate_hash()?
            || !verify_ed25519_signature(public_key, &bytes, &signature)
        {
            return Err(RuntimeError::Signature {
                revision: self.revision,
                reason: "host-supplied event signature is invalid".into(),
            });
        }
        let public_bytes = BASE64
            .decode(public_key)
            .map_err(|error| RuntimeError::Signature {
                revision: self.revision,
                reason: error.to_string(),
            })?;
        let derived_key_id = format!("ed25519:{:x}", Sha256::digest(public_bytes));
        if derived_key_id != key_id {
            return Err(RuntimeError::Signature {
                revision: self.revision,
                reason: "signerKeyId does not match signerPublicKey".into(),
            });
        }
        self.signature = Some(signature);
        Ok(())
    }

    fn seal(&mut self, signer: &DeviceSigner) -> Result<()> {
        let bytes = self.prepare_host_signature(signer.key_id(), signer.public_key())?;
        self.attach_host_signature(signer.sign_base64(&bytes))
    }

    fn verify_signature(&self, devices: &BTreeMap<String, DeviceRecord>) -> Result<()> {
        if self.schema_version == "1" {
            return Ok(());
        }
        let key_id = self
            .signer_key_id
            .as_ref()
            .ok_or_else(|| RuntimeError::Signature {
                revision: self.revision,
                reason: "missing signerKeyId".into(),
            })?;
        let public_key = if let Some(device) = devices.get(key_id) {
            if device.status == DeviceStatus::Revoked {
                return Err(RuntimeError::RevokedSigner(key_id.clone()));
            }
            device.public_key.as_str()
        } else {
            // Trust any not-yet-enrolled signer, regardless of actor kind —
            // not just the very first one this runtime ever sees. ActorKind
            // is a self-declared routing label (harness vs. operator vs.
            // system), not a cryptographic identity claim, so gating trust on
            // it adds no real protection; the actual trust boundary is the
            // signature itself (proving local OS-keychain/filesystem key
            // possession) plus revocation (above), which remains fully
            // enforced. A second local identity (for example a headless
            // daemon and an interactive CLI) may legitimately sign events;
            // the signature and revocation records are the trust boundary.
            self.signer_public_key
                .as_deref()
                .ok_or_else(|| RuntimeError::UnknownSigner(key_id.clone()))?
        };
        let public_bytes = BASE64
            .decode(public_key)
            .map_err(|error| RuntimeError::Signature {
                revision: self.revision,
                reason: error.to_string(),
            })?;
        let public_array: [u8; 32] =
            public_bytes
                .try_into()
                .map_err(|_| RuntimeError::Signature {
                    revision: self.revision,
                    reason: "Ed25519 public key must contain 32 bytes".into(),
                })?;
        let verifying_key =
            VerifyingKey::from_bytes(&public_array).map_err(|error| RuntimeError::Signature {
                revision: self.revision,
                reason: error.to_string(),
            })?;
        let derived_key_id = format!("ed25519:{:x}", Sha256::digest(public_array));
        if &derived_key_id != key_id {
            return Err(RuntimeError::Signature {
                revision: self.revision,
                reason: "signerKeyId does not match the public key".into(),
            });
        }
        let signature_bytes = BASE64
            .decode(self.signature.as_deref().unwrap_or_default())
            .map_err(|error| RuntimeError::Signature {
                revision: self.revision,
                reason: error.to_string(),
            })?;
        let signature =
            Signature::from_slice(&signature_bytes).map_err(|error| RuntimeError::Signature {
                revision: self.revision,
                reason: error.to_string(),
            })?;
        verifying_key
            .verify(&self.canonical_unsigned_bytes()?, &signature)
            .map_err(|error| RuntimeError::Signature {
                revision: self.revision,
                reason: error.to_string(),
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KbdStateV2 {
    pub schema_version: String,
    pub project_id: String,
    pub run_id: String,
    pub revision: u64,
    #[serde(default)]
    pub frontier: CausalFrontier,
    #[serde(default)]
    pub replica_heads: BTreeMap<String, ReplicaHead>,
    pub last_event_id: Option<String>,
    pub last_event_hash: Option<String>,
    pub lifecycle: LifecycleState,
    pub plan_revision: u64,
    pub checkpoint: Option<Checkpoint>,
    pub exact_next_work: Option<String>,
    pub active_path: ActivePath,
    pub phases: BTreeMap<String, Phase>,
    pub completion: BTreeMap<CompletionDimension, Completion>,
    pub decisions: BTreeMap<String, Decision>,
    pub blockers: BTreeMap<String, Blocker>,
    #[serde(default)]
    pub claims: BTreeMap<String, ClaimRecord>,
    #[serde(default)]
    pub submodule_pins: BTreeMap<String, SubmodulePin>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replica_view: Option<ReplicaView>,
    #[serde(default)]
    pub conflicts: BTreeMap<String, ConflictRecord>,
    pub devices: BTreeMap<String, DeviceRecord>,
    pub command_revisions: BTreeMap<String, u64>,
}

pub type RuntimeState = KbdStateV2;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SignedFoldedCheckpoint {
    pub schema_version: String,
    pub event_count: u64,
    pub frontier_hash: String,
    pub last_event_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub state: KbdStateV2,
    pub signer_key_id: String,
    pub signer_public_key: String,
    pub signature: String,
}

impl SignedFoldedCheckpoint {
    fn canonical_unsigned_bytes(&self) -> Result<Vec<u8>> {
        let mut unsigned = serde_json::to_value(self)?;
        unsigned
            .as_object_mut()
            .expect("checkpoint is an object")
            .remove("signature");
        serde_jcs::to_vec(&unsigned).map_err(|error| RuntimeError::InvalidState(error.to_string()))
    }

    fn verify(&self) -> Result<()> {
        if self.schema_version != "1"
            || self.frontier_hash != frontier_hash(&self.state.frontier)?
            || self.event_count != self.state.revision
            || !verify_ed25519_signature(
                &self.signer_public_key,
                &self.canonical_unsigned_bytes()?,
                &self.signature,
            )
        {
            return Err(RuntimeError::InvalidState(
                "folded-state checkpoint signature or frontier is invalid".into(),
            ));
        }
        let key_id = format!(
            "ed25519:{:x}",
            Sha256::digest(
                BASE64
                    .decode(&self.signer_public_key)
                    .map_err(|error| RuntimeError::InvalidState(error.to_string()))?
            )
        );
        if key_id != self.signer_key_id {
            return Err(RuntimeError::InvalidState(
                "folded-state checkpoint signer key id is invalid".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JournalArchiveSummary {
    pub segment: PathBuf,
    pub manifest: PathBuf,
    pub archived_events: u64,
    pub retained_events: u64,
    pub payload_sha256: String,
    pub previous_manifest_sha256: Option<String>,
    pub rollback_metadata: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct JournalArchiveManifest {
    schema_version: String,
    segment: String,
    first_revision: u64,
    last_revision: u64,
    event_count: u64,
    payload_sha256: String,
    previous_manifest_sha256: Option<String>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckpointPointer {
    schema_version: String,
    checkpoint: String,
    authority_source_sha256: String,
}

fn frontier_hash(frontier: &CausalFrontier) -> Result<String> {
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(
            serde_jcs::to_vec(frontier)
                .map_err(|error| RuntimeError::InvalidState(error.to_string()))?
        )
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MigrationSummary {
    pub project_id: String,
    pub progress_files: u64,
    pub migrated_progress_files: u64,
    pub uncertain_rows: u64,
    pub invalid_files: u64,
    pub alias_conflicts: u64,
    pub legacy_read_only_phases: u64,
    pub stale_projections: u64,
    pub unreplayable_history: bool,
    pub backup_directory: Option<PathBuf>,
    pub backup_manifest: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JournalMigrationSummary {
    pub project_id: String,
    pub replica_id: String,
    pub source_journal: PathBuf,
    pub archive_journal: PathBuf,
    pub active_journal: PathBuf,
    pub project_document: PathBuf,
    pub rollback_instructions: PathBuf,
    pub original_events: usize,
    pub migrated_events: usize,
    pub archive_sha256: String,
    pub already_migrated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct MigrationBackupEntry {
    source: PathBuf,
    backup: PathBuf,
    bytes: u64,
    sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct MigrationBackupManifest {
    schema_version: String,
    project_id: String,
    created_at: DateTime<Utc>,
    files: Vec<MigrationBackupEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectManifest {
    pub schema_version: String,
    pub project_id: String,
    pub repository_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MutationContext {
    pub expected_revision: u64,
    pub command_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandEnvelope {
    pub schema_version: String,
    pub project_id: String,
    pub run_id: String,
    pub command_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frontier: Option<CausalFrontier>,
    #[serde(default)]
    pub expected_revision: u64,
    pub actor: Actor,
    pub command: CommandKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SignedCommandEnvelope {
    pub command: CommandEnvelope,
    pub signer_key_id: String,
    pub signature: String,
}

impl SignedCommandEnvelope {
    pub fn sign(command: CommandEnvelope, signer: &DeviceSigner) -> Result<Self> {
        let signer_key_id = signer.key_id().to_string();
        let bytes = remote_command_signable_bytes(&command, &signer_key_id)?;
        Ok(Self {
            command,
            signer_key_id,
            signature: signer.sign_base64(&bytes),
        })
    }

    pub fn verify(&self, state: &KbdStateV2) -> Result<()> {
        let device = state
            .devices
            .get(&self.signer_key_id)
            .filter(|device| device.status == DeviceStatus::Active)
            .ok_or_else(|| RuntimeError::UnknownSigner(self.signer_key_id.clone()))?;
        let bytes = remote_command_signable_bytes(&self.command, &self.signer_key_id)?;
        if !verify_ed25519_signature(&device.public_key, &bytes, &self.signature) {
            return Err(RuntimeError::Signature {
                revision: state.revision,
                reason: "remote command signature is invalid".into(),
            });
        }
        Ok(())
    }
}

fn remote_command_signable_bytes(
    command: &CommandEnvelope,
    signer_key_id: &str,
) -> Result<Vec<u8>> {
    let mut bytes = serde_jcs::to_vec(command)
        .map_err(|error| RuntimeError::InvalidState(error.to_string()))?;
    bytes.push(0);
    bytes.extend_from_slice(signer_key_id.as_bytes());
    Ok(bytes)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum CommandKind {
    Pause {
        checkpoint: Checkpoint,
    },
    Cancel {
        reason: String,
    },
    LifecycleTransition {
        to: LifecycleState,
        reason: String,
    },
    Resume {
        plan_revision: u64,
    },
    PlanRevise {
        reason: String,
        exact_next_work: Option<String>,
    },
    PhaseDefine {
        phase: Phase,
    },
    PhaseTransition {
        phase_id: String,
        to: WorkStatus,
    },
    StageEnter {
        phase_id: String,
        stage: Stage,
    },
    StageTransition {
        phase_id: String,
        stage_id: String,
        to: WorkStatus,
    },
    ChangeRegister {
        phase_id: String,
        change: Change,
    },
    ChangeTransition {
        phase_id: String,
        change_id: String,
        to: WorkStatus,
    },
    TaskRegister {
        phase_id: String,
        change_id: String,
        task: Task,
    },
    TaskTransition {
        phase_id: String,
        change_id: String,
        task_id: String,
        to: WorkStatus,
        summary: Option<String>,
    },
    ActivePathSet {
        active_path: ActivePath,
        exact_next_work: Option<String>,
    },
    CompletionSet {
        dimension: CompletionDimension,
        completion: Completion,
    },
    DecisionRecord {
        decision: Decision,
    },
    BlockerRecord {
        blocker: Blocker,
    },
    BlockerClear {
        blocker_id: String,
        resolution: String,
    },
    DeviceEnroll {
        device: DeviceRecord,
    },
    DeviceRevoke {
        key_id: String,
        reason: String,
    },
    DeviceRotate {
        previous_key_id: String,
        replacement: DeviceRecord,
    },
    ConflictResolve {
        conflict_id: String,
        winner_event_id: String,
        reason: String,
    },
    ClaimAcquire {
        scope: String,
        mode: ClaimMode,
        ttl_seconds: u64,
        holder_id: String,
    },
    ClaimRenew {
        claim_id: String,
        ttl_seconds: u64,
    },
    ClaimRelease {
        claim_id: String,
    },
    SubmodulePinSet {
        pin: SubmodulePin,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    pub command_id: String,
    pub committed_revision: u64,
    pub duplicate: bool,
    pub state: KbdStateV2,
    /// Set when this specific command failed business-logic validation in
    /// `KbdStateV2::apply` (e.g. an invalid work-item transition). `state`
    /// and `committed_revision` reflect the runtime UNCHANGED by this
    /// command in that case — the failure must never block the log
    /// position from advancing past this entry, or every later command
    /// (and a fresh replay on restart) gets stuck retrying the same
    /// permanently-invalid entry forever. `#[serde(default)]` keeps
    /// already-persisted `CommandResult`s (written before this field
    /// existed) deserializable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apply_error: Option<String>,
}

impl Default for KbdStateV2 {
    fn default() -> Self {
        let completion = CompletionDimension::all()
            .into_iter()
            .map(|dimension| (dimension, Completion::not_tracked()))
            .collect();
        Self {
            schema_version: EVENT_SCHEMA_VERSION.into(),
            project_id: String::new(),
            run_id: String::new(),
            revision: 0,
            frontier: CausalFrontier::empty(),
            replica_heads: BTreeMap::new(),
            last_event_id: None,
            last_event_hash: None,
            lifecycle: LifecycleState::Ready,
            plan_revision: 1,
            checkpoint: None,
            exact_next_work: None,
            active_path: ActivePath::default(),
            phases: BTreeMap::new(),
            completion,
            decisions: BTreeMap::new(),
            blockers: BTreeMap::new(),
            claims: BTreeMap::new(),
            submodule_pins: BTreeMap::new(),
            replica_view: None,
            conflicts: BTreeMap::new(),
            devices: BTreeMap::new(),
            command_revisions: BTreeMap::new(),
        }
    }
}

impl KbdStateV2 {
    pub fn apply(&mut self, event: &Event) -> Result<()> {
        self.apply_internal(event, true)
    }

    pub(crate) fn apply_folded(&mut self, event: &Event) -> Result<()> {
        self.apply_internal(event, false)
    }

    fn apply_internal(&mut self, event: &Event, validate_envelope: bool) -> Result<()> {
        let replica_aware = !event.replica_id.is_empty() && event.lamport > 0;
        if validate_envelope {
            if replica_aware {
                if event.actor_id != event.actor.id {
                    return Err(RuntimeError::InvalidState(format!(
                        "event {} actorId does not match actor.id",
                        event.event_id
                    )));
                }
                if event.frontier != self.frontier {
                    return Err(RuntimeError::InvalidState(format!(
                        "event {} was prepared from frontier {:?}, current frontier is {:?}",
                        event.event_id, event.frontier.0, self.frontier.0
                    )));
                }
                if event.lamport != self.frontier.next_lamport(&event.replica_id) {
                    return Err(RuntimeError::InvalidState(format!(
                        "event {} has Lamport {}, expected {} for replica {}",
                        event.event_id,
                        event.lamport,
                        self.frontier.next_lamport(&event.replica_id),
                        event.replica_id
                    )));
                }
                let replica_head = self.replica_heads.get(&event.replica_id);
                if event.causal_parent.as_deref() != replica_head.map(|head| head.event_id.as_str())
                {
                    return Err(RuntimeError::CausalChain {
                        revision: event.revision,
                    });
                }
                if event.previous_hash.as_deref()
                    != replica_head.map(|head| head.integrity_hash.as_str())
                {
                    return Err(RuntimeError::Integrity {
                        revision: event.revision,
                    });
                }
                let expected_revision = self.frontier.derived_revision().saturating_add(1);
                if event.revision != expected_revision {
                    return Err(RuntimeError::RevisionConflict {
                        expected: expected_revision,
                        actual: event.revision,
                    });
                }
            } else {
                if event.revision != self.revision + 1 || event.expected_revision != self.revision {
                    return Err(RuntimeError::RevisionConflict {
                        expected: event.expected_revision,
                        actual: self.revision,
                    });
                }
                if event.causal_parent != self.last_event_id {
                    return Err(RuntimeError::CausalChain {
                        revision: event.revision,
                    });
                }
                if event.previous_hash != self.last_event_hash {
                    return Err(RuntimeError::Integrity {
                        revision: event.revision,
                    });
                }
            }
        }
        event.verify_signature(&self.devices)?;
        if event.integrity_hash != event.calculate_hash()? {
            return Err(RuntimeError::Integrity {
                revision: event.revision,
            });
        }
        if self.revision == 0
            && (!matches!(event.kind, EventKind::RunInitialized { .. })
                || event.actor.kind != ActorKind::Operator)
        {
            return Err(RuntimeError::InvalidState(
                "the first signed event must initialize the run under operator authority".into(),
            ));
        }
        if let Some(command_id) = event.command_id.as_ref() {
            if self.command_revisions.contains_key(command_id) {
                return Err(RuntimeError::DuplicateCommand(command_id.clone()));
            }
        }
        let signer_already_enrolled = event
            .signer_key_id
            .as_deref()
            .map(|id| self.devices.contains_key(id))
            .unwrap_or(false);
        // Enroll any not-yet-known signer, not just the genesis signer —
        // mirrors the widened trust condition in `Event::verify_signature`
        // above. Keeps `devices` (and therefore revocation) accurate for
        // every local identity that legitimately touches this project,
        // instead of only the first one ever seen. ActorKind is a routing
        // label, while the signature is the cryptographic trust boundary.
        if event.schema_version != "1" && !signer_already_enrolled {
            let key_id = event
                .signer_key_id
                .clone()
                .ok_or_else(|| RuntimeError::Signature {
                    revision: event.revision,
                    reason: "missing bootstrap signerKeyId".into(),
                })?;
            let public_key =
                event
                    .signer_public_key
                    .clone()
                    .ok_or_else(|| RuntimeError::Signature {
                        revision: event.revision,
                        reason: "missing bootstrap signerPublicKey".into(),
                    })?;
            self.devices.insert(
                key_id.clone(),
                DeviceRecord {
                    device_id: event.actor.device.clone(),
                    key_id,
                    public_key,
                    status: DeviceStatus::Active,
                    enrolled_at_revision: event.revision,
                    revoked_at_revision: None,
                },
            );
        }

        match &event.kind {
            EventKind::RunInitialized {
                initial_state,
                exact_next_work,
                plan_revision,
            } => {
                if self.revision != 0 {
                    return Err(RuntimeError::AlreadyInitialized);
                }
                self.project_id.clone_from(&event.project_id);
                self.run_id.clone_from(&event.run_id);
                self.lifecycle = initial_state.clone();
                self.exact_next_work.clone_from(exact_next_work);
                self.plan_revision = *plan_revision;
            }
            EventKind::LifecycleTransition { from, to, .. } => {
                if &self.lifecycle != from || !valid_transition(from, to) {
                    return Err(RuntimeError::InvalidTransition {
                        from: self.lifecycle.clone(),
                        to: to.clone(),
                    });
                }
                self.lifecycle = to.clone();
            }
            EventKind::CheckpointCreated { checkpoint } => {
                self.exact_next_work.clone_from(&checkpoint.exact_next_work);
                self.checkpoint = Some(checkpoint.clone());
            }
            EventKind::PauseCheckpointed { checkpoint } => {
                if self.lifecycle != checkpoint.previous_state
                    || !valid_transition(&self.lifecycle, &LifecycleState::Paused)
                {
                    return Err(RuntimeError::InvalidTransition {
                        from: self.lifecycle.clone(),
                        to: LifecycleState::Paused,
                    });
                }
                self.exact_next_work.clone_from(&checkpoint.exact_next_work);
                self.checkpoint = Some(checkpoint.clone());
                self.lifecycle = LifecycleState::Paused;
            }
            EventKind::PlanRevised {
                from_revision,
                to_revision,
                exact_next_work,
                ..
            } => {
                if *from_revision != self.plan_revision || *to_revision != from_revision + 1 {
                    return Err(RuntimeError::PlanRevision {
                        supplied: *from_revision,
                        current: self.plan_revision,
                    });
                }
                self.plan_revision = *to_revision;
                self.exact_next_work.clone_from(exact_next_work);
            }
            EventKind::LegacyStateImported {
                phases,
                active_path,
                completion,
                decisions,
                blockers,
            } => {
                if !self.phases.is_empty() {
                    return Err(RuntimeError::InvalidState(
                        "legacy workflow state has already been imported".into(),
                    ));
                }
                self.phases.clone_from(phases);
                self.active_path.clone_from(active_path);
                self.completion.clone_from(completion);
                self.decisions.clone_from(decisions);
                self.blockers.clone_from(blockers);
                self.recalculate_implementation();
            }
            EventKind::PhaseDefined { phase } => {
                if phase.slug.is_empty()
                    || phase.slug == "."
                    || phase.slug == ".."
                    || phase.slug.contains('/')
                    || phase.slug.contains('\\')
                {
                    return Err(RuntimeError::InvalidState(format!(
                        "phase {} has invalid projection slug {}",
                        phase.id, phase.slug
                    )));
                }
                if self.phases.contains_key(&phase.id) {
                    return Err(RuntimeError::WorkItemExists {
                        kind: "phase",
                        id: phase.id.clone(),
                    });
                }
                if let Some(parent_id) = phase.parent_phase_id.as_ref() {
                    if parent_id == &phase.id || !self.phases.contains_key(parent_id) {
                        return Err(RuntimeError::InvalidState(format!(
                            "phase {} has unknown or cyclic parent {}",
                            phase.id, parent_id
                        )));
                    }
                }
                self.phases.insert(phase.id.clone(), phase.clone());
            }
            EventKind::PhaseTransitioned { phase_id, from, to } => {
                let phase = self.phase_mut(phase_id)?;
                if phase.status != *from || !valid_work_transition(from, to) {
                    return Err(RuntimeError::InvalidWorkTransition {
                        item_id: phase_id.clone(),
                        from: phase.status.clone(),
                        to: to.clone(),
                    });
                }
                phase.status = to.clone();
            }
            EventKind::StageEntered { phase_id, stage } => {
                let phase = self.phase_mut(phase_id)?;
                if phase.stages.contains_key(&stage.id) {
                    return Err(RuntimeError::WorkItemExists {
                        kind: "stage",
                        id: stage.id.clone(),
                    });
                }
                phase.stages.insert(stage.id.clone(), stage.clone());
                self.active_path.phase_id = Some(phase_id.clone());
                self.active_path.stage_id = Some(stage.id.clone());
                self.active_path.change_id = None;
                self.active_path.task_id = None;
            }
            EventKind::StageTransitioned {
                phase_id,
                stage_id,
                from,
                to,
            } => {
                let stage = self
                    .phase_mut(phase_id)?
                    .stages
                    .get_mut(stage_id)
                    .ok_or_else(|| RuntimeError::WorkItemNotFound {
                        kind: "stage",
                        id: stage_id.clone(),
                    })?;
                if stage.status != *from || !valid_work_transition(from, to) {
                    return Err(RuntimeError::InvalidWorkTransition {
                        item_id: stage_id.clone(),
                        from: stage.status.clone(),
                        to: to.clone(),
                    });
                }
                stage.status = to.clone();
            }
            EventKind::ChangeRegistered { phase_id, change } => {
                let phase = self.phase_mut(phase_id)?;
                if phase.changes.contains_key(&change.id) {
                    return Err(RuntimeError::WorkItemExists {
                        kind: "change",
                        id: change.id.clone(),
                    });
                }
                phase.changes.insert(change.id.clone(), change.clone());
                self.recalculate_implementation();
            }
            EventKind::ChangeTransitioned {
                phase_id,
                change_id,
                from,
                to,
            } => {
                let change = self.change_mut(phase_id, change_id)?;
                if change.status != *from || !valid_work_transition(from, to) {
                    return Err(RuntimeError::InvalidWorkTransition {
                        item_id: change_id.clone(),
                        from: change.status.clone(),
                        to: to.clone(),
                    });
                }
                change.status = to.clone();
                change.implementation_status = to.clone();
                self.recalculate_implementation();
            }
            EventKind::TaskRegistered {
                phase_id,
                change_id,
                task,
            } => {
                let change = self.change_mut(phase_id, change_id)?;
                if change.tasks.contains_key(&task.id) {
                    return Err(RuntimeError::WorkItemExists {
                        kind: "task",
                        id: task.id.clone(),
                    });
                }
                change.tasks.insert(task.id.clone(), task.clone());
                self.recalculate_change(phase_id, change_id)?;
                self.recalculate_implementation();
            }
            EventKind::TaskTransitioned {
                phase_id,
                change_id,
                task_id,
                from,
                to,
                summary,
            } => {
                let task = self.task_mut(phase_id, change_id, task_id)?;
                if task.status != *from || !valid_work_transition(from, to) {
                    return Err(RuntimeError::InvalidWorkTransition {
                        item_id: task_id.clone(),
                        from: task.status.clone(),
                        to: to.clone(),
                    });
                }
                task.status = to.clone();
                if summary.is_some() {
                    task.summary.clone_from(summary);
                }
                self.recalculate_change(phase_id, change_id)?;
                self.recalculate_implementation();
            }
            EventKind::ActivePathChanged {
                active_path,
                exact_next_work,
            } => {
                self.validate_active_path(active_path)?;
                self.active_path.clone_from(active_path);
                self.exact_next_work.clone_from(exact_next_work);
            }
            EventKind::CompletionUpdated {
                dimension,
                completion,
            } => {
                self.completion.insert(*dimension, completion.clone());
            }
            EventKind::DecisionRecorded { decision } => {
                if self.decisions.contains_key(&decision.id) {
                    return Err(RuntimeError::WorkItemExists {
                        kind: "decision",
                        id: decision.id.clone(),
                    });
                }
                self.decisions.insert(decision.id.clone(), decision.clone());
            }
            EventKind::BlockerRecorded { blocker } => {
                if self.blockers.contains_key(&blocker.id) {
                    return Err(RuntimeError::WorkItemExists {
                        kind: "blocker",
                        id: blocker.id.clone(),
                    });
                }
                self.blockers.insert(blocker.id.clone(), blocker.clone());
            }
            EventKind::BlockerCleared {
                blocker_id,
                resolution,
            } => {
                let blocker = self.blockers.get_mut(blocker_id).ok_or_else(|| {
                    RuntimeError::WorkItemNotFound {
                        kind: "blocker",
                        id: blocker_id.clone(),
                    }
                })?;
                blocker.resolved = true;
                blocker.resolution = Some(resolution.clone());
            }
            EventKind::ClaimAcquired {
                claim_id,
                scope,
                holder_id,
                mode,
                expires_at,
                monotonic_token,
            } => {
                if claim_id.trim().is_empty()
                    || scope.trim().is_empty()
                    || holder_id.trim().is_empty()
                    || *monotonic_token == 0
                {
                    return Err(RuntimeError::InvalidState(
                        "claim acquisition requires non-empty IDs/scope and a positive token"
                            .into(),
                    ));
                }
                if self.claims.contains_key(claim_id) {
                    return Err(RuntimeError::WorkItemExists {
                        kind: "claim",
                        id: claim_id.clone(),
                    });
                }
                self.claims.insert(
                    claim_id.clone(),
                    ClaimRecord {
                        claim_id: claim_id.clone(),
                        scope: scope.clone(),
                        replica_id: if event.replica_id.is_empty() {
                            "legacy".into()
                        } else {
                            event.replica_id.clone()
                        },
                        holder_id: holder_id.clone(),
                        mode: *mode,
                        expires_at: *expires_at,
                        monotonic_token: *monotonic_token,
                        acquired_event_id: event.event_id.clone(),
                        last_event_id: event.event_id.clone(),
                        released: false,
                    },
                );
            }
            EventKind::ClaimRenewed {
                claim_id,
                expires_at,
                monotonic_token,
            } => {
                let claim = self.claims.get_mut(claim_id).ok_or_else(|| {
                    RuntimeError::WorkItemNotFound {
                        kind: "claim",
                        id: claim_id.clone(),
                    }
                })?;
                if claim.released || *monotonic_token <= claim.monotonic_token {
                    return Err(RuntimeError::InvalidState(format!(
                        "claim {claim_id} renewal token must increase monotonically"
                    )));
                }
                claim.expires_at = *expires_at;
                claim.monotonic_token = *monotonic_token;
                claim.last_event_id = event.event_id.clone();
            }
            EventKind::ClaimReleased {
                claim_id,
                monotonic_token,
            } => {
                let claim = self.claims.get_mut(claim_id).ok_or_else(|| {
                    RuntimeError::WorkItemNotFound {
                        kind: "claim",
                        id: claim_id.clone(),
                    }
                })?;
                if *monotonic_token <= claim.monotonic_token {
                    return Err(RuntimeError::InvalidState(format!(
                        "claim {claim_id} release token must increase monotonically"
                    )));
                }
                claim.monotonic_token = *monotonic_token;
                claim.last_event_id = event.event_id.clone();
                claim.released = true;
            }
            EventKind::SubmodulePinRecorded { pin } => {
                if pin.path.trim().is_empty()
                    || pin.path.starts_with('/')
                    || pin
                        .path
                        .split('/')
                        .any(|part| part.is_empty() || part == "..")
                    || Uuid::parse_str(&pin.child_project_id).is_err()
                    || !matches!(pin.gitlink_sha.len(), 40 | 64)
                    || !pin.gitlink_sha.bytes().all(|byte| byte.is_ascii_hexdigit())
                {
                    return Err(RuntimeError::InvalidState(
                        "submodule pin requires a relative normalized path, child UUID, and valid gitlink SHA".into(),
                    ));
                }
                self.submodule_pins.insert(pin.path.clone(), pin.clone());
            }
            EventKind::DeviceEnrolled { device } => {
                if self.devices.contains_key(&device.key_id) {
                    return Err(RuntimeError::WorkItemExists {
                        kind: "device key",
                        id: device.key_id.clone(),
                    });
                }
                validate_device_record(device, event.revision)?;
                self.devices.insert(device.key_id.clone(), device.clone());
            }
            EventKind::DeviceRevoked { key_id, .. } => {
                let device =
                    self.devices
                        .get_mut(key_id)
                        .ok_or_else(|| RuntimeError::WorkItemNotFound {
                            kind: "device key",
                            id: key_id.clone(),
                        })?;
                device.status = DeviceStatus::Revoked;
                device.revoked_at_revision = Some(event.revision);
            }
            EventKind::DeviceKeyRotated {
                previous_key_id,
                replacement,
            } => {
                validate_device_record(replacement, event.revision)?;
                let previous = self.devices.get_mut(previous_key_id).ok_or_else(|| {
                    RuntimeError::WorkItemNotFound {
                        kind: "device key",
                        id: previous_key_id.clone(),
                    }
                })?;
                previous.status = DeviceStatus::Revoked;
                previous.revoked_at_revision = Some(event.revision);
                self.devices
                    .insert(replacement.key_id.clone(), replacement.clone());
            }
            EventKind::ConflictRecorded { conflict } => {
                self.conflicts
                    .entry(conflict.id.clone())
                    .or_insert_with(|| conflict.clone());
            }
            EventKind::ConflictResolved {
                conflict_id,
                winner_event_id,
                reason,
            } => {
                if event.actor.kind != ActorKind::Operator {
                    return Err(RuntimeError::InvalidState(
                        "only an operator may resolve a conflict".into(),
                    ));
                }
                let conflict = self.conflicts.get_mut(conflict_id).ok_or_else(|| {
                    RuntimeError::WorkItemNotFound {
                        kind: "conflict",
                        id: conflict_id.clone(),
                    }
                })?;
                if !conflict
                    .candidates
                    .iter()
                    .any(|candidate| candidate.event_id == *winner_event_id)
                {
                    return Err(RuntimeError::InvalidState(format!(
                        "event {winner_event_id} is not a candidate for conflict {conflict_id}"
                    )));
                }
                conflict.winner_event_id.clone_from(winner_event_id);
                conflict.resolved_by_event_id = Some(event.event_id.clone());
                conflict.resolution_reason = Some(reason.clone());
            }
        }

        let replica_id = if replica_aware {
            event.replica_id.clone()
        } else {
            "legacy".into()
        };
        let lamport = if replica_aware {
            event.lamport
        } else {
            event.revision
        };
        self.frontier.advance(replica_id.clone(), lamport);
        self.replica_heads.insert(
            replica_id,
            ReplicaHead {
                event_id: event.event_id.clone(),
                integrity_hash: event.integrity_hash.clone(),
                lamport,
            },
        );
        self.revision = self.frontier.derived_revision();
        self.last_event_id = Some(event.event_id.clone());
        self.last_event_hash = Some(event.integrity_hash.clone());
        if let Some(command_id) = event.command_id.as_ref() {
            self.command_revisions
                .insert(command_id.clone(), event.revision);
        }
        Ok(())
    }

    fn phase_mut(&mut self, phase_id: &str) -> Result<&mut Phase> {
        self.phases
            .get_mut(phase_id)
            .ok_or_else(|| RuntimeError::WorkItemNotFound {
                kind: "phase",
                id: phase_id.to_string(),
            })
    }

    fn change_mut(&mut self, phase_id: &str, change_id: &str) -> Result<&mut Change> {
        self.phase_mut(phase_id)?
            .changes
            .get_mut(change_id)
            .ok_or_else(|| RuntimeError::WorkItemNotFound {
                kind: "change",
                id: change_id.to_string(),
            })
    }

    fn task_mut(&mut self, phase_id: &str, change_id: &str, task_id: &str) -> Result<&mut Task> {
        self.change_mut(phase_id, change_id)?
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| RuntimeError::WorkItemNotFound {
                kind: "task",
                id: task_id.to_string(),
            })
    }

    fn validate_active_path(&self, path: &ActivePath) -> Result<()> {
        if path.commit.as_ref().is_some_and(|commit| {
            !matches!(commit.len(), 40 | 64) || !commit.bytes().all(|byte| byte.is_ascii_hexdigit())
        }) {
            return Err(RuntimeError::InvalidState(
                "active path commit must be a valid Git object ID".into(),
            ));
        }
        if !path.phase_path.is_empty() {
            let mut previous: Option<&str> = None;
            for phase_id in &path.phase_path {
                let phase =
                    self.phases
                        .get(phase_id)
                        .ok_or_else(|| RuntimeError::WorkItemNotFound {
                            kind: "phase",
                            id: phase_id.clone(),
                        })?;
                if phase.parent_phase_id.as_deref() != previous {
                    return Err(RuntimeError::InvalidState(format!(
                        "phase path is not a parent chain at {}",
                        phase_id
                    )));
                }
                previous = Some(phase_id);
            }
            if path.phase_id.as_ref() != path.phase_path.last() {
                return Err(RuntimeError::InvalidState(
                    "phaseId must equal the final phasePath entry".into(),
                ));
            }
        }
        let Some(phase_id) = path.phase_id.as_ref() else {
            if !path.phase_path.is_empty()
                || path.stage_id.is_some()
                || path.change_id.is_some()
                || path.task_id.is_some()
            {
                return Err(RuntimeError::InvalidState(
                    "active path descendants require a phase".into(),
                ));
            }
            return Ok(());
        };
        let phase = self
            .phases
            .get(phase_id)
            .ok_or_else(|| RuntimeError::WorkItemNotFound {
                kind: "phase",
                id: phase_id.clone(),
            })?;
        if let Some(stage_id) = path.stage_id.as_ref() {
            if !phase.stages.contains_key(stage_id) {
                return Err(RuntimeError::WorkItemNotFound {
                    kind: "stage",
                    id: stage_id.clone(),
                });
            }
        }
        if let Some(change_id) = path.change_id.as_ref() {
            let change =
                phase
                    .changes
                    .get(change_id)
                    .ok_or_else(|| RuntimeError::WorkItemNotFound {
                        kind: "change",
                        id: change_id.clone(),
                    })?;
            if let Some(task_id) = path.task_id.as_ref() {
                if !change.tasks.contains_key(task_id) {
                    return Err(RuntimeError::WorkItemNotFound {
                        kind: "task",
                        id: task_id.clone(),
                    });
                }
            }
        } else if path.task_id.is_some() {
            return Err(RuntimeError::InvalidState(
                "active task requires an active change".into(),
            ));
        }
        Ok(())
    }

    fn recalculate_change(&mut self, phase_id: &str, change_id: &str) -> Result<()> {
        let change = self.change_mut(phase_id, change_id)?;
        if change.tasks.is_empty() {
            return Ok(());
        }
        change.implementation_status =
            if change.tasks.values().all(|task| task.status.is_complete()) {
                WorkStatus::Complete
            } else if change
                .tasks
                .values()
                .any(|task| task.status == WorkStatus::Blocked)
            {
                WorkStatus::Blocked
            } else if change
                .tasks
                .values()
                .any(|task| task.status == WorkStatus::InProgress)
            {
                WorkStatus::InProgress
            } else {
                WorkStatus::Pending
            };
        change.status = change.implementation_status.clone();
        Ok(())
    }

    fn recalculate_implementation(&mut self) {
        let total = self
            .phases
            .values()
            .map(|phase| phase.changes.len() as u64)
            .sum();
        let completed = self
            .phases
            .values()
            .flat_map(|phase| phase.changes.values())
            .filter(|change| change.implementation_status.is_complete())
            .count() as u64;
        let status = if total > 0 && completed == total {
            WorkStatus::Complete
        } else if self
            .phases
            .values()
            .flat_map(|phase| phase.changes.values())
            .any(|change| change.implementation_status == WorkStatus::Blocked)
        {
            WorkStatus::Blocked
        } else if completed > 0 {
            WorkStatus::InProgress
        } else {
            WorkStatus::Pending
        };
        self.completion.insert(
            CompletionDimension::Implementation,
            Completion {
                completed,
                total,
                status,
                summary: None,
                blockers: Vec::new(),
            },
        );
    }
}

fn validate_device_record(device: &DeviceRecord, revision: u64) -> Result<()> {
    let public_key =
        BASE64
            .decode(&device.public_key)
            .map_err(|error| RuntimeError::Signature {
                revision,
                reason: error.to_string(),
            })?;
    let public_key: [u8; 32] = public_key.try_into().map_err(|_| RuntimeError::Signature {
        revision,
        reason: "Ed25519 public key must contain 32 bytes".into(),
    })?;
    let expected = format!("ed25519:{:x}", Sha256::digest(public_key));
    if expected != device.key_id {
        return Err(RuntimeError::Signature {
            revision,
            reason: "device key id does not match its public key".into(),
        });
    }
    Ok(())
}

pub fn replay_events(events: &[Event]) -> Result<RuntimeState> {
    let mut state = RuntimeState::default();
    let mut ids = HashSet::new();
    for event in events {
        if !ids.insert(event.event_id.clone()) {
            return Err(RuntimeError::DuplicateEvent(event.event_id.clone()));
        }
        state.apply(event)?;
    }
    Ok(state)
}

fn read_event_file(path: &Path) -> Result<Vec<Event>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut events = Vec::new();
    for line in BufReader::new(File::open(path)?).lines() {
        let line = line?;
        if !line.trim().is_empty() {
            events.push(serde_json::from_str(&line)?);
        }
    }
    Ok(events)
}

/// Verify the exact pre-replica v2 wire representation before migration.
///
/// Those journals included two now-obsolete nullable envelope keys. Dropping
/// unknown keys during `Event` deserialization changes the canonical bytes and
/// makes an otherwise valid historical signature appear forged. This verifier
/// is deliberately private to migration: it authenticates the raw JSON object,
/// permits only the one known historical key set, and validates the scalar
/// chain before the events are re-signed into the current replica schema.
fn validate_pre_replica_v2_journal(path: &Path, events: &[Event]) -> Result<()> {
    const LEGACY_KEYS: &[&str] = &[
        "actor",
        "causalParent",
        "commandId",
        "eventId",
        "expectedRevision",
        "fencingToken",
        "integrityHash",
        "kind",
        "leaseId",
        "previousHash",
        "projectId",
        "revision",
        "runId",
        "schemaVersion",
        "signature",
        "signerKeyId",
        "signerPublicKey",
        "timestamp",
    ];
    let allowed = LEGACY_KEYS.iter().copied().collect::<HashSet<_>>();
    let lines = BufReader::new(File::open(path)?)
        .lines()
        .filter_map(|line| match line {
            Ok(line) if line.trim().is_empty() => None,
            other => Some(other),
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if lines.len() != events.len() {
        return Err(RuntimeError::InvalidState(format!(
            "legacy journal line count {} does not match decoded event count {}",
            lines.len(),
            events.len()
        )));
    }

    let mut previous_event_id: Option<&str> = None;
    let mut previous_hash: Option<&str> = None;
    let mut project_id: Option<&str> = None;
    let mut run_id: Option<&str> = None;
    let mut event_ids = HashSet::new();
    let mut command_ids = HashSet::new();
    for (index, (line, event)) in lines.iter().zip(events).enumerate() {
        if event.schema_version != EVENT_SCHEMA_VERSION
            || !event.replica_id.is_empty()
            || event.lamport != 0
            || !event.frontier.is_empty()
        {
            return Err(RuntimeError::InvalidState(format!(
                "event {} is not a pre-replica v2 migration event",
                event.event_id
            )));
        }
        let mut raw: serde_json::Value = serde_json::from_str(line)?;
        let object = raw.as_object_mut().ok_or_else(|| {
            RuntimeError::InvalidState("legacy journal event must be a JSON object".into())
        })?;
        if let Some(unexpected) = object.keys().find(|key| !allowed.contains(key.as_str())) {
            return Err(RuntimeError::InvalidState(format!(
                "legacy journal event contains unsupported key {unexpected}"
            )));
        }
        let stored_hash = object
            .get("integrityHash")
            .and_then(serde_json::Value::as_str)
            .ok_or(RuntimeError::Integrity {
                revision: event.revision,
            })?
            .to_owned();
        let signature = object
            .get("signature")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| RuntimeError::Signature {
                revision: event.revision,
                reason: "legacy signature is missing".into(),
            })?
            .to_owned();
        let signer_key_id = object
            .get("signerKeyId")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| RuntimeError::Signature {
                revision: event.revision,
                reason: "legacy signerKeyId is missing".into(),
            })?
            .to_owned();
        let signer_public_key = object
            .get("signerPublicKey")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| RuntimeError::Signature {
                revision: event.revision,
                reason: "legacy signerPublicKey is missing".into(),
            })?
            .to_owned();
        object.insert(
            "integrityHash".into(),
            serde_json::Value::String(String::new()),
        );
        object.remove("signature");
        let unsigned = serde_jcs::to_vec(&raw)
            .map_err(|error| RuntimeError::InvalidState(error.to_string()))?;
        if format!("{:x}", Sha256::digest(&unsigned)) != stored_hash {
            return Err(RuntimeError::Integrity {
                revision: event.revision,
            });
        }
        if !verify_ed25519_signature(&signer_public_key, &unsigned, &signature) {
            return Err(RuntimeError::Signature {
                revision: event.revision,
                reason: "pre-replica v2 signature is invalid".into(),
            });
        }
        let public_key =
            BASE64
                .decode(&signer_public_key)
                .map_err(|error| RuntimeError::Signature {
                    revision: event.revision,
                    reason: error.to_string(),
                })?;
        if format!("ed25519:{:x}", Sha256::digest(public_key)) != signer_key_id {
            return Err(RuntimeError::Signature {
                revision: event.revision,
                reason: "legacy signerKeyId does not match signerPublicKey".into(),
            });
        }

        let expected_revision = index as u64 + 1;
        if event.revision != expected_revision || event.expected_revision + 1 != event.revision {
            return Err(RuntimeError::RevisionConflict {
                expected: expected_revision,
                actual: event.revision,
            });
        }
        if event.causal_parent.as_deref() != previous_event_id {
            return Err(RuntimeError::CausalChain {
                revision: event.revision,
            });
        }
        if event.previous_hash.as_deref() != previous_hash {
            return Err(RuntimeError::Integrity {
                revision: event.revision,
            });
        }
        if project_id.is_some_and(|project_id| project_id != event.project_id)
            || run_id.is_some_and(|run_id| run_id != event.run_id)
        {
            return Err(RuntimeError::InvalidState(
                "legacy journal changes projectId or runId mid-chain".into(),
            ));
        }
        if !event_ids.insert(event.event_id.as_str()) {
            return Err(RuntimeError::DuplicateEvent(event.event_id.clone()));
        }
        if let Some(command_id) = event.command_id.as_deref() {
            if !command_ids.insert(command_id) {
                return Err(RuntimeError::DuplicateCommand(command_id.into()));
            }
        }
        project_id.get_or_insert(&event.project_id);
        run_id.get_or_insert(&event.run_id);
        previous_event_id = Some(&event.event_id);
        previous_hash = Some(&event.integrity_hash);
    }
    Ok(())
}

fn validate_journal_for_migration(path: &Path, events: &[Event]) -> Result<()> {
    if events.iter().all(|event| {
        event.schema_version == EVENT_SCHEMA_VERSION
            && event.replica_id.is_empty()
            && event.lamport == 0
            && event.frontier.is_empty()
    }) {
        validate_pre_replica_v2_journal(path, events)
    } else {
        replay_events(events).map(|_| ())
    }
}

fn resign_journal_events(
    source_events: &[Event],
    project_id: &str,
    replica_id: &str,
    signer: &DeviceSigner,
) -> Result<Vec<Event>> {
    let mut migrated = Vec::with_capacity(source_events.len());
    let mut frontier = CausalFrontier::empty();
    let mut previous_event_id = None;
    let mut previous_hash = None;
    for source in source_events {
        let lamport = frontier.next_lamport(replica_id);
        let mut event = Event {
            schema_version: EVENT_SCHEMA_VERSION.into(),
            project_id: project_id.into(),
            replica_id: replica_id.into(),
            run_id: source.run_id.clone(),
            event_id: Uuid::new_v4().to_string(),
            command_id: source.command_id.clone(),
            revision: frontier.derived_revision().saturating_add(1),
            expected_revision: frontier.derived_revision(),
            lamport,
            frontier: frontier.clone(),
            causal_parent: previous_event_id.clone(),
            actor: source.actor.clone(),
            actor_id: source.actor.id.clone(),
            timestamp: source.timestamp,
            kind: source.kind.clone(),
            previous_hash: previous_hash.clone(),
            migration_provenance: Some(MigrationProvenance {
                source_project_id: source.project_id.clone(),
                source_replica_id: (!source.replica_id.is_empty())
                    .then(|| source.replica_id.clone()),
                source_event_id: source.event_id.clone(),
                source_integrity_hash: source.integrity_hash.clone(),
                source_previous_hash: source.previous_hash.clone(),
            }),
            integrity_hash: String::new(),
            signer_key_id: None,
            signer_public_key: None,
            signature: None,
        };
        event.seal(signer)?;
        previous_event_id = Some(event.event_id.clone());
        previous_hash = Some(event.integrity_hash.clone());
        frontier.advance(replica_id, lamport);
        migrated.push(event);
    }
    replay_events(&migrated)?;
    Ok(migrated)
}

fn validate_migrated_provenance(
    migrated: &[Event],
    original: &[Event],
    replica_id: &str,
) -> Result<()> {
    if migrated.len() != original.len() {
        return Err(RuntimeError::InvalidState(format!(
            "active replica journal has {} events but archived v1 journal has {}",
            migrated.len(),
            original.len()
        )));
    }
    for (migrated, original) in migrated.iter().zip(original) {
        let provenance = migrated.migration_provenance.as_ref().ok_or_else(|| {
            RuntimeError::InvalidState(format!(
                "migrated event {} has no source provenance",
                migrated.event_id
            ))
        })?;
        if migrated.replica_id != replica_id
            || provenance.source_event_id != original.event_id
            || provenance.source_integrity_hash != original.integrity_hash
        {
            return Err(RuntimeError::InvalidState(format!(
                "migrated event {} does not match archived source event {}",
                migrated.event_id, original.event_id
            )));
        }
    }
    replay_events(migrated)?;
    Ok(())
}

fn write_event_file_atomic(path: &Path, events: &[Event]) -> Result<()> {
    if path.exists() {
        return Err(RuntimeError::InvalidState(format!(
            "refusing to overwrite existing replica journal {}",
            path.display()
        )));
    }
    let parent = path.parent().ok_or_else(|| {
        RuntimeError::InvalidState(format!("{} has no parent directory", path.display()))
    })?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".events.jsonl.{}.tmp", Uuid::new_v4()));
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&temporary)?;
    for event in events {
        serde_json::to_writer(&mut file, event)?;
        file.write_all(b"\n")?;
    }
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

fn replace_event_file_atomic(path: &Path, events: &[Event]) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        RuntimeError::InvalidState(format!("{} has no parent directory", path.display()))
    })?;
    fs::create_dir_all(parent)?;
    let temporary = parent.join(format!(".events.jsonl.{}.tmp", Uuid::new_v4()));
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(&temporary)?;
    for event in events {
        serde_json::to_writer(&mut file, event)?;
        file.write_all(b"\n")?;
    }
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

/// Apply one already committed event to an in-memory state machine.
///
/// Storage integrations use this entry point after an event is durable. It
/// deliberately does not create, sign, append, or project anything: those are
/// journal-writer and projection-worker concerns.
pub fn apply_committed_event(state: &mut RuntimeState, event: &Event) -> Result<()> {
    state.apply(event)
}

fn valid_work_transition(from: &WorkStatus, to: &WorkStatus) -> bool {
    use WorkStatus::*;
    matches!(
        (from, to),
        (Pending, InProgress)
            | (Pending, Blocked)
            | (Pending, Cancelled)
            | (InProgress, Blocked)
            | (InProgress, Complete)
            | (InProgress, Cancelled)
            | (Blocked, InProgress)
            | (Blocked, Cancelled)
    )
}

fn valid_transition(from: &LifecycleState, to: &LifecycleState) -> bool {
    use LifecycleState::*;
    matches!(
        (from, to),
        (Ready, Running)
            | (Ready, Paused)
            | (Running, PauseRequested)
            | (PauseRequested, Paused)
            | (Running, Paused)
            | (Running, Blocked)
            | (Blocked, Paused)
            | (Paused, Running)
            | (Blocked, Running)
            | (Running, Completed)
            | (Ready, Cancelled)
            | (Running, Cancelled)
            | (PauseRequested, Cancelled)
            | (Paused, Cancelled)
            | (Blocked, Cancelled)
            | (Ready, Failed)
            | (Running, Failed)
            | (PauseRequested, Failed)
            | (Paused, Failed)
            | (Blocked, Failed)
    )
}

#[derive(Debug, Clone)]
pub struct Runtime {
    root: PathBuf,
    project_root: PathBuf,
    replica_id: String,
    key_storage: KeyStorage,
    read_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyStorage {
    /// Compatibility mode for repositories that have not established a
    /// canonical project identity yet.
    LegacyRuntimeFile,
    /// Canonical runtimes use the host credential store, unless an explicit
    /// headless key file is configured.
    PlatformCredentialStore,
}

fn canonical_runtime_root(project_id: &str) -> PathBuf {
    let data_root = std::env::var_os("PROMETHEUS_DATA_DIR")
        .map(PathBuf::from)
        .or_else(dirs_next::data_local_dir)
        .unwrap_or_else(|| std::env::temp_dir().join("prometheus-data"));
    canonical_runtime_root_at(&data_root, project_id)
}

fn canonical_runtime_root_at(data_root: &Path, project_id: &str) -> PathBuf {
    data_root
        .join("prometheus")
        .join("kbd")
        .join("projects")
        .join(project_id)
}

fn repository_fingerprint(project_root: &Path) -> String {
    let repository_identity = fs::read_to_string(project_root.join(".git/config"))
        .ok()
        .and_then(|config| {
            config
                .lines()
                .map(str::trim)
                .find_map(|line| line.strip_prefix("url = ").map(str::to_owned))
        })
        .unwrap_or_else(|| {
            project_root
                .canonicalize()
                .unwrap_or_else(|_| project_root.to_path_buf())
                .to_string_lossy()
                .into_owned()
        });
    format!(
        "sha256:{:x}",
        Sha256::digest(repository_identity.as_bytes())
    )
}

fn read_project_manifest(project_root: &Path) -> Result<Option<ProjectManifest>> {
    let path = project_root.join(".prometheus").join("project.json");
    if !path.exists() {
        return Ok(None);
    }
    let manifest: ProjectManifest = serde_json::from_reader(File::open(&path)?)?;
    if manifest.schema_version != "1"
        || Uuid::parse_str(&manifest.project_id).is_err()
        || !manifest.repository_fingerprint.starts_with("sha256:")
    {
        return Err(RuntimeError::InvalidState(format!(
            "{} is not a valid immutable project identity manifest",
            path.display()
        )));
    }
    Ok(Some(manifest))
}

fn ensure_project_manifest(project_root: &Path) -> Result<ProjectManifest> {
    if let Some(manifest) = read_project_manifest(project_root)? {
        return Ok(manifest);
    }
    let manifest = ProjectManifest {
        schema_version: "1".into(),
        project_id: Uuid::new_v4().to_string(),
        repository_fingerprint: repository_fingerprint(project_root),
    };
    let path = project_root.join(".prometheus").join("project.json");
    atomic_json(&path, &serde_json::to_value(&manifest)?)?;
    Ok(manifest)
}

impl Runtime {
    /// Complete the durable authority startup sequence exactly once.
    ///
    /// Recovery must precede reconciliation: a final interrupted JSONL append
    /// can otherwise make the first document import fail before the tail has
    /// been normalized or archived. Keeping this sequence in the runtime
    /// constructors also prevents higher-level facades from replaying the same
    /// journal into Loro a second time during startup.
    fn finish_authority_open(runtime: Self) -> Result<Self> {
        runtime.recover_journal_tail()?;
        runtime.reconcile_project_document()?;
        Ok(runtime)
    }

    pub fn open(project_root: impl AsRef<Path>) -> Self {
        let project_root = project_root.as_ref().to_path_buf();
        let manifest = read_project_manifest(&project_root).ok().flatten();
        let (root, key_storage) = manifest
            .map(|manifest| {
                (
                    canonical_runtime_root(&manifest.project_id),
                    KeyStorage::PlatformCredentialStore,
                )
            })
            .unwrap_or_else(|| {
                (
                    project_root.join(".kbd-orchestrator").join("runtime"),
                    KeyStorage::LegacyRuntimeFile,
                )
            });
        Self {
            root,
            project_root,
            replica_id: "legacy".into(),
            key_storage,
            read_only: false,
        }
    }

    /// Open the platform-owned canonical runtime, creating the repository's
    /// immutable identity manifest when it does not yet exist.
    pub fn open_canonical(project_root: impl AsRef<Path>) -> Result<Self> {
        let project_root = fs::canonicalize(project_root.as_ref())?;
        let manifest = ensure_project_manifest(&project_root)?;
        let registration = registry::ProjectRegistry::open().register_existing(&project_root)?;
        let runtime = Self {
            root: canonical_runtime_root(&manifest.project_id),
            project_root,
            replica_id: registration.registration.replica_id,
            key_storage: KeyStorage::PlatformCredentialStore,
            read_only: registration.registration.read_only,
        };
        Self::finish_authority_open(runtime)
    }

    /// Open a canonical runtime beneath an explicit application-data root.
    /// This is used by hermetic tests and managed/headless deployments.
    pub fn open_canonical_at(
        project_root: impl AsRef<Path>,
        data_root: impl AsRef<Path>,
    ) -> Result<Self> {
        let project_root = fs::canonicalize(project_root.as_ref())?;
        let manifest = ensure_project_manifest(&project_root)?;
        let registration = registry::ProjectRegistry::open_at(data_root.as_ref())
            .register_existing(&project_root)?;
        let runtime = Self {
            root: canonical_runtime_root_at(data_root.as_ref(), &manifest.project_id),
            project_root,
            replica_id: registration.registration.replica_id,
            key_storage: KeyStorage::PlatformCredentialStore,
            read_only: registration.registration.read_only,
        };
        Self::finish_authority_open(runtime)
    }

    /// Open an already-registered replica from the platform data root without
    /// changing its registry classification.
    pub fn open_registered(project_root: &Path, expected_project_id: &str) -> Result<Self> {
        let data_root = std::env::var_os("PROMETHEUS_DATA_DIR")
            .map(PathBuf::from)
            .or_else(dirs_next::data_local_dir)
            .unwrap_or_else(|| std::env::temp_dir().join("prometheus-data"));
        Self::open_registered_at(project_root, &data_root, expected_project_id)
    }

    /// Open an already-registered replica without changing its classification.
    /// Intended for daemon routing and offline migration utilities.
    pub fn open_registered_at(
        project_root: &Path,
        data_root: &Path,
        expected_project_id: &str,
    ) -> Result<Self> {
        let project_root = fs::canonicalize(project_root)?;
        let manifest = read_project_manifest(&project_root)?.ok_or_else(|| {
            RuntimeError::InvalidState(format!(
                "registered project {} has no identity manifest",
                project_root.display()
            ))
        })?;
        if manifest.project_id != expected_project_id {
            return Err(RuntimeError::ProjectMismatch {
                supplied: manifest.project_id,
                current: expected_project_id.into(),
            });
        }
        let registration = registry::ProjectRegistry::open_at(data_root)
            .lookup_path(&project_root)?
            .ok_or_else(|| {
                RuntimeError::InvalidState(format!(
                    "project {} is not registered",
                    project_root.display()
                ))
            })?;
        let runtime = Self {
            root: canonical_runtime_root_at(data_root, expected_project_id),
            project_root,
            replica_id: registration.replica_id,
            key_storage: KeyStorage::PlatformCredentialStore,
            read_only: registration.read_only,
        };
        Self::finish_authority_open(runtime)
    }

    pub fn runtime_root(&self) -> &Path {
        &self.root
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn replica_id(&self) -> &str {
        &self.replica_id
    }

    pub fn events_path(&self) -> PathBuf {
        self.journal_root().join("events.jsonl")
    }

    fn lock_path(&self) -> PathBuf {
        self.journal_root().join("runtime.lock")
    }

    pub fn journal_lock_path(&self) -> PathBuf {
        self.lock_path()
    }

    fn journal_root(&self) -> PathBuf {
        match self.key_storage {
            KeyStorage::LegacyRuntimeFile => self.root.clone(),
            KeyStorage::PlatformCredentialStore => {
                self.root.join("replicas").join(&self.replica_id)
            }
        }
    }

    pub fn project_document(&self) -> Option<ProjectDocument> {
        (self.key_storage == KeyStorage::PlatformCredentialStore).then(|| {
            let project_id = self
                .project_manifest(false)
                .ok()
                .flatten()
                .map(|manifest| manifest.project_id)
                .unwrap_or_default();
            ProjectDocument::open(&self.root, project_id)
        })
    }

    fn device_key_path(&self) -> PathBuf {
        self.root.join("device-key.json")
    }

    pub fn device_signer(&self) -> Result<DeviceSigner> {
        if let Some(path) = std::env::var_os("PROMETHEUS_DEVICE_KEY_FILE") {
            return load_device_key(Path::new(&path));
        }
        match self.key_storage {
            KeyStorage::LegacyRuntimeFile => self.legacy_file_device_signer(),
            KeyStorage::PlatformCredentialStore => self.platform_device_signer(),
        }
    }

    fn legacy_file_device_signer(&self) -> Result<DeviceSigner> {
        let path = self.device_key_path();
        if path.exists() {
            return load_device_key(&path);
        }
        fs::create_dir_all(&self.root)?;
        let signer = DeviceSigner::generate();
        let stored = stored_device_key(&signer);
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        options.mode(0o600);
        match options.open(&path) {
            Ok(mut file) => {
                serde_json::to_writer_pretty(&mut file, &stored)?;
                file.write_all(b"\n")?;
                file.sync_all()?;
                File::open(&self.root)?.sync_all()?;
                Ok(signer)
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => self.device_signer(),
            Err(error) => Err(error.into()),
        }
    }

    fn platform_device_signer(&self) -> Result<DeviceSigner> {
        #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
        {
            fs::create_dir_all(&self.root)?;
            let lock = OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(self.root.join("key-storage.lock"))?;
            lock.lock_exclusive()?;
            let result = self.platform_device_signer_locked();
            FileExt::unlock(&lock)?;
            result
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err(RuntimeError::InvalidState(
                "this platform has no supported OS credential store; configure an existing mode-0600 PROMETHEUS_DEVICE_KEY_FILE".into(),
            ))
        }
    }

    #[cfg(any(target_os = "macos", target_os = "linux", target_os = "windows"))]
    fn platform_device_signer_locked(&self) -> Result<DeviceSigner> {
        let manifest = read_project_manifest(&self.project_root)?.ok_or_else(|| {
            RuntimeError::InvalidState(
                "canonical key storage requires .prometheus/project.json".into(),
            )
        })?;
        let account = format!("{}:{}", manifest.project_id, device_identity());
        let entry = keyring::Entry::new("prometheus-kbd-device", &account).map_err(|error| {
            RuntimeError::InvalidState(format!("cannot open OS credential store: {error}"))
        })?;
        match entry.get_secret() {
            Ok(secret) => signer_from_stored(serde_json::from_slice(&secret)?),
            Err(keyring::Error::NoEntry) => {
                let signer = DeviceSigner::generate();
                let encoded = serde_json::to_vec(&stored_device_key(&signer))?;
                entry.set_secret(&encoded).map_err(|error| {
                    RuntimeError::InvalidState(format!(
                        "cannot persist device key in OS credential store: {error}; configure an existing mode-0600 PROMETHEUS_DEVICE_KEY_FILE for a headless host"
                    ))
                })?;
                Ok(signer)
            }
            Err(error) => Err(RuntimeError::InvalidState(format!(
                "cannot read device key from OS credential store: {error}; configure an existing mode-0600 PROMETHEUS_DEVICE_KEY_FILE for a headless host"
            ))),
        }
    }

    pub fn project_manifest(&self, create: bool) -> Result<Option<ProjectManifest>> {
        if create {
            Ok(Some(ensure_project_manifest(&self.project_root)?))
        } else {
            read_project_manifest(&self.project_root)
        }
    }

    pub fn events(&self) -> Result<Vec<Event>> {
        if let Some(document) = self.project_document() {
            let events = document.events()?;
            if !events.is_empty() {
                return Ok(events);
            }
        }
        self.journal_events()
    }

    fn journal_events(&self) -> Result<Vec<Event>> {
        let mut events = Vec::new();
        for path in self.journal_event_paths()? {
            let file = File::open(path)?;
            for line in BufReader::new(file).lines() {
                let line = line?;
                if !line.trim().is_empty() {
                    events.push(serde_json::from_str(&line)?);
                }
            }
        }
        Ok(events)
    }

    fn journal_event_paths(&self) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        let archive_dir = self.journal_root().join("archives");
        if archive_dir.exists() {
            for entry in fs::read_dir(archive_dir)? {
                let path = entry?.path();
                if path
                    .extension()
                    .is_some_and(|extension| extension == "jsonl")
                {
                    paths.push(path);
                }
            }
            paths.sort();
        }
        if self.events_path().exists() {
            paths.push(self.events_path());
        }
        Ok(paths)
    }

    pub fn replica_events(&self) -> Result<Vec<Event>> {
        self.journal_events()
    }

    /// Import every fsynced local journal entry that is not yet present in the
    /// authoritative project document. This is safe to call on every startup
    /// and closes a crash window between journal fsync and Loro snapshot fsync.
    pub fn reconcile_project_document(&self) -> Result<usize> {
        let Some(document) = self.project_document() else {
            return Ok(0);
        };
        document.ingest_events(&self.journal_events()?)
    }

    pub fn migrate_v1_journal(&self) -> Result<Option<JournalMigrationSummary>> {
        self.migrate_v1_journal_inner(None)
    }

    pub fn v1_journal_migration_required(&self) -> bool {
        self.key_storage == KeyStorage::PlatformCredentialStore
            && self.root.join("events.jsonl").is_file()
    }

    fn migrate_v1_journal_inner(
        &self,
        signer_override: Option<&DeviceSigner>,
    ) -> Result<Option<JournalMigrationSummary>> {
        if self.key_storage != KeyStorage::PlatformCredentialStore {
            return Ok(None);
        }
        fs::create_dir_all(&self.root)?;
        let migration_lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.root.join("journal-migration.lock"))?;
        migration_lock.lock_exclusive()?;
        let result = (|| {
            let source_journal = self.root.join("events.jsonl");
            let archive_journal = self.root.join("events.v1.jsonl.archive");
            let active_journal = self.events_path();
            if !source_journal.exists() && !archive_journal.exists() {
                return Ok(None);
            }
            let manifest = self.project_manifest(false)?.ok_or_else(|| {
                RuntimeError::InvalidState(
                    "journal migration requires .prometheus/project.json".into(),
                )
            })?;
            let original_events = if source_journal.exists() {
                read_event_file(&source_journal)?
            } else {
                read_event_file(&archive_journal)?
            };
            let original_path = if source_journal.exists() {
                &source_journal
            } else {
                &archive_journal
            };
            validate_journal_for_migration(original_path, &original_events)?;

            let already_migrated = archive_journal.exists();
            let migrated_events = if active_journal.exists() {
                let events = read_event_file(&active_journal)?;
                validate_migrated_provenance(&events, &original_events, &self.replica_id)?;
                events
            } else {
                let owned_signer;
                let signer = if let Some(signer) = signer_override {
                    signer
                } else {
                    owned_signer = self.device_signer()?;
                    &owned_signer
                };
                let events = resign_journal_events(
                    &original_events,
                    &manifest.project_id,
                    &self.replica_id,
                    signer,
                )?;
                write_event_file_atomic(&active_journal, &events)?;
                events
            };

            let document = self.project_document().ok_or_else(|| {
                RuntimeError::InvalidState("canonical project document is unavailable".into())
            })?;
            document.ingest_events(&migrated_events)?;

            if source_journal.exists() {
                if archive_journal.exists() {
                    return Err(RuntimeError::InvalidState(format!(
                        "both {} and {} exist; preserve both and adjudicate before retrying",
                        source_journal.display(),
                        archive_journal.display()
                    )));
                }
                fs::rename(&source_journal, &archive_journal)?;
                File::open(&self.root)?.sync_all()?;
            }
            let archive_bytes = fs::read(&archive_journal)?;
            let archive_sha256 = format!("{:x}", Sha256::digest(&archive_bytes));
            atomic_text(
                &archive_journal.with_extension("archive.sha256"),
                &format!(
                    "{}  {}\n",
                    archive_sha256,
                    archive_journal
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                ),
            )?;
            let rollback_instructions = self.root.join("JOURNAL-MIGRATION-ROLLBACK.md");
            atomic_text(
                &rollback_instructions,
                &format!(
                    "# KBD journal migration rollback\n\n\
                     1. Stop Sovereign Sync.\n\
                     2. Verify `{checksum}` against `{archive}`.\n\
                     3. Move `{active}` and `{document}` to timestamped archive names; do not delete them.\n\
                     4. Rename `{archive}` back to `{source}`.\n\
                     5. Restart the migration and verify journal/document equivalence before service startup.\n",
                    checksum = archive_journal.with_extension("archive.sha256").display(),
                    archive = archive_journal.display(),
                    active = active_journal.display(),
                    document = document.path().display(),
                    source = source_journal.display(),
                ),
            )?;
            Ok(Some(JournalMigrationSummary {
                project_id: manifest.project_id,
                replica_id: self.replica_id.clone(),
                source_journal,
                archive_journal,
                active_journal,
                project_document: document.path(),
                rollback_instructions,
                original_events: original_events.len(),
                migrated_events: migrated_events.len(),
                archive_sha256,
                already_migrated,
            }))
        })();
        FileExt::unlock(&migration_lock)?;
        result
    }

    /// Normalize or recover an interrupted final journal append.
    ///
    /// A complete JSON event without its trailing newline is normalized in
    /// place. An invalid unterminated tail is first archived beside the
    /// journal with a SHA-256 checksum, then the journal is truncated to its
    /// last complete newline. Interior corruption remains a hard replay error.
    pub fn recover_journal_tail(&self) -> Result<Option<PathBuf>> {
        fs::create_dir_all(self.journal_root())?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?;
        lock.lock_exclusive()?;
        self.recover_journal_tail_locked()
    }

    fn recover_journal_tail_locked(&self) -> Result<Option<PathBuf>> {
        let path = self.events_path();
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&path)?;
        if bytes.is_empty() || bytes.ends_with(b"\n") {
            return Ok(None);
        }

        let valid_len = bytes
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map(|index| index + 1)
            .unwrap_or(0);
        let tail = &bytes[valid_len..];
        if serde_json::from_slice::<Event>(tail).is_ok() {
            let mut file = OpenOptions::new().append(true).open(&path)?;
            file.write_all(b"\n")?;
            file.sync_data()?;
            return Ok(None);
        }

        let journal_root = self.journal_root();
        let archive = journal_root.join(format!(
            "events.jsonl.torn-{}-{}.archive",
            Utc::now().format("%Y%m%dT%H%M%S%.6fZ"),
            Uuid::new_v4()
        ));
        let mut archive_file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&archive)?;
        archive_file.write_all(tail)?;
        archive_file.sync_all()?;

        let checksum = format!("{:x}", Sha256::digest(tail));
        let checksum_path = archive.with_extension("archive.sha256");
        let mut checksum_file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&checksum_path)?;
        writeln!(
            checksum_file,
            "{}  {}",
            checksum,
            archive.file_name().unwrap_or_default().to_string_lossy()
        )?;
        checksum_file.sync_all()?;
        File::open(&journal_root)?.sync_all()?;

        let journal = OpenOptions::new().write(true).open(&path)?;
        journal.set_len(valid_len as u64)?;
        journal.sync_data()?;
        Ok(Some(archive))
    }

    /// Export the verified audit chain as RFC 8785 canonical JSON Lines.
    pub fn export_signed_audit(&self, mut writer: impl Write) -> Result<u64> {
        self.reconcile_project_document()?;
        let events = self.events()?;
        self.fold_authority_events(&events)?;
        for event in &events {
            let bytes = if event.schema_version == "1" {
                serde_json::to_vec(event)?
            } else {
                serde_jcs::to_vec(event)
                    .map_err(|error| RuntimeError::InvalidState(error.to_string()))?
            };
            writer.write_all(&bytes)?;
            writer.write_all(b"\n")?;
        }
        Ok(events.len() as u64)
    }

    pub fn signed_audit_jsonl(&self) -> Result<(Vec<u8>, u64)> {
        let mut bytes = Vec::new();
        let event_count = self.export_signed_audit(&mut bytes)?;
        Ok((bytes, event_count))
    }

    /// Write the converged audit chain to `refs/heads/audit/kbd` using only
    /// Git plumbing. The current branch, worktree, and ordinary Git index are
    /// never read as an authority and are never modified.
    pub fn export_audit_to_git(&self) -> Result<GitAuditExport> {
        let (bytes, event_count) = self.signed_audit_jsonl()?;
        if event_count == 0 {
            return Err(RuntimeError::NotInitialized);
        }
        let state = self.replay()?;
        let tree_path = format!("audit/kbd/{}.jsonl", state.project_id);
        let sha256 = format!("{:x}", Sha256::digest(&bytes));
        let blob = git_plumbing(
            &self.project_root,
            &["hash-object", "-w", "--stdin"],
            Some(&bytes),
            None,
        )?;
        let old_commit = git_plumbing_optional(
            &self.project_root,
            &["rev-parse", "--verify", AUDIT_GIT_REF],
            None,
        )?;
        if let Some(old_commit) = old_commit.as_deref() {
            let existing = git_plumbing_optional(
                &self.project_root,
                &["rev-parse", &format!("{old_commit}:{tree_path}")],
                None,
            )?;
            if existing.as_deref() == Some(blob.as_str()) {
                return Ok(GitAuditExport {
                    ref_name: AUDIT_GIT_REF.into(),
                    tree_path,
                    commit_id: old_commit.into(),
                    event_count,
                    sha256,
                    unchanged: true,
                });
            }
        }

        let index_path =
            std::env::temp_dir().join(format!("prometheus-kbd-audit-index-{}", Uuid::new_v4()));
        let index_guard = TemporaryGitIndex(index_path.clone());
        if let Some(old_commit) = old_commit.as_deref() {
            git_plumbing(
                &self.project_root,
                &["read-tree", old_commit],
                None,
                Some(&index_path),
            )?;
        } else {
            git_plumbing(
                &self.project_root,
                &["read-tree", "--empty"],
                None,
                Some(&index_path),
            )?;
        }
        git_plumbing(
            &self.project_root,
            &[
                "update-index",
                "--add",
                "--cacheinfo",
                "100644",
                &blob,
                &tree_path,
            ],
            None,
            Some(&index_path),
        )?;
        let tree = git_plumbing(&self.project_root, &["write-tree"], None, Some(&index_path))?;
        let mut commit_args = vec!["commit-tree", tree.as_str()];
        if let Some(old_commit) = old_commit.as_deref() {
            commit_args.extend(["-p", old_commit]);
        }
        let message = format!(
            "KBD audit export for {}\n\nEvents: {event_count}\nSHA-256: {sha256}\n",
            state.project_id
        );
        let commit_id = git_plumbing(
            &self.project_root,
            &commit_args,
            Some(message.as_bytes()),
            Some(&index_path),
        )?;
        let zero_oid = "0".repeat(blob.len());
        let expected_old = old_commit.as_deref().unwrap_or(&zero_oid);
        git_plumbing(
            &self.project_root,
            &["update-ref", AUDIT_GIT_REF, &commit_id, expected_old],
            None,
            None,
        )?;
        drop(index_guard);
        Ok(GitAuditExport {
            ref_name: AUDIT_GIT_REF.into(),
            tree_path,
            commit_id,
            event_count,
            sha256,
            unchanged: false,
        })
    }

    /// Replay only authoritative event state. This deliberately omits local
    /// git/submodule decoration so readiness checks cannot block on a checkout.
    pub fn replay_authority(&self) -> Result<RuntimeState> {
        if let Some(state) = self.load_folded_checkpoint()? {
            return Ok(state);
        }
        self.fold_authority_events(&self.events()?)
    }

    pub fn replay(&self) -> Result<RuntimeState> {
        let mut state = self.replay_authority()?;
        self.decorate_replica_view(&mut state);
        Ok(state)
    }

    pub fn export_project_updates(&self) -> Result<Vec<u8>> {
        self.reconcile_project_document()?;
        self.project_document()
            .ok_or_else(|| {
                RuntimeError::InvalidState(
                    "authoritative Loro sync requires a canonical runtime".into(),
                )
            })?
            .export_updates()
    }

    pub fn import_project_updates(&self, updates: &[u8]) -> Result<(usize, RuntimeState)> {
        self.reconcile_project_document()?;
        let (inserted, mut state) = self
            .project_document()
            .ok_or_else(|| {
                RuntimeError::InvalidState(
                    "authoritative Loro sync requires a canonical runtime".into(),
                )
            })?
            .import_updates(updates)?;
        self.persist_folded_checkpoint(&state)?;
        self.decorate_replica_view(&mut state);
        Ok((inserted, state))
    }

    fn checkpoint_dir(&self) -> PathBuf {
        self.root.join("checkpoints")
    }

    fn authority_source_sha256(&self) -> Result<String> {
        let mut digest = Sha256::new();
        if let Some(document) = self.project_document() {
            let path = document.path();
            if path.exists() {
                digest.update(fs::read(path)?);
            }
        } else {
            for path in self.journal_event_paths()? {
                digest.update(fs::read(path)?);
            }
        }
        Ok(format!("sha256:{:x}", digest.finalize()))
    }

    fn persist_folded_checkpoint(&self, state: &RuntimeState) -> Result<PathBuf> {
        if state.revision == 0 {
            return Err(RuntimeError::NotInitialized);
        }
        let mut authoritative = state.clone();
        authoritative.replica_view = None;
        let signer = self.device_signer()?;
        let mut checkpoint = SignedFoldedCheckpoint {
            schema_version: "1".into(),
            event_count: authoritative.revision,
            frontier_hash: frontier_hash(&authoritative.frontier)?,
            last_event_hash: authoritative.last_event_hash.clone(),
            created_at: Utc::now(),
            state: authoritative,
            signer_key_id: signer.key_id().into(),
            signer_public_key: signer.public_key().into(),
            signature: String::new(),
        };
        checkpoint.signature = signer.sign_base64(&checkpoint.canonical_unsigned_bytes()?);
        let directory = self.checkpoint_dir();
        fs::create_dir_all(&directory)?;
        let filename = format!(
            "checkpoint-{:020}-{}.json",
            checkpoint.event_count,
            checkpoint.frontier_hash.trim_start_matches("sha256:")
        );
        let path = directory.join(&filename);
        if !path.exists() {
            atomic_json(&path, &serde_json::to_value(&checkpoint)?)?;
        }
        let pointer = CheckpointPointer {
            schema_version: "1".into(),
            checkpoint: filename,
            authority_source_sha256: self.authority_source_sha256()?,
        };
        atomic_json(
            &directory.join("current.json"),
            &serde_json::to_value(pointer)?,
        )?;
        Ok(path)
    }

    fn load_folded_checkpoint(&self) -> Result<Option<RuntimeState>> {
        let pointer_path = self.checkpoint_dir().join("current.json");
        if !pointer_path.exists() {
            return Ok(None);
        }
        let pointer: CheckpointPointer = serde_json::from_reader(File::open(pointer_path)?)?;
        if pointer.schema_version != "1"
            || pointer.authority_source_sha256 != self.authority_source_sha256()?
        {
            return Ok(None);
        }
        let checkpoint_path = self.checkpoint_dir().join(pointer.checkpoint);
        let checkpoint: SignedFoldedCheckpoint =
            serde_json::from_reader(File::open(checkpoint_path)?)?;
        checkpoint.verify()?;
        Ok(Some(checkpoint.state))
    }

    /// Move an immutable prefix of the local journal into a hash-linked archive
    /// segment. Replay continues across archives plus the active suffix, and
    /// rollback metadata names every artifact required to restore the layout.
    pub fn compact_journal(&self, retain_active: usize) -> Result<Option<JournalArchiveSummary>> {
        self.ensure_writable_replica()?;
        if retain_active == 0 {
            return Err(RuntimeError::InvalidState(
                "journal compaction must retain at least one active event".into(),
            ));
        }
        fs::create_dir_all(self.journal_root())?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?;
        lock.lock_exclusive()?;
        self.recover_journal_tail_locked()?;
        let active = read_event_file(&self.events_path())?;
        if active.len() <= retain_active {
            return Ok(None);
        }
        let checkpoint_count = fs::read_dir(self.checkpoint_dir())?
            .filter_map(std::result::Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("checkpoint-")
            })
            .count();
        if checkpoint_count < 2 {
            return Err(RuntimeError::InvalidState(
                "journal compaction requires at least two signed checkpoints".into(),
            ));
        }
        let split = active.len() - retain_active;
        let archived = &active[..split];
        let retained = &active[split..];
        let directory = self.journal_root().join("archives");
        fs::create_dir_all(&directory)?;
        let first = archived.first().expect("non-empty archive").revision;
        let last = archived.last().expect("non-empty archive").revision;
        let segment_name = format!("segment-{first:020}-{last:020}.jsonl");
        let segment = directory.join(&segment_name);
        let mut payload = Vec::new();
        for event in archived {
            serde_json::to_writer(&mut payload, event)?;
            payload.push(b'\n');
        }
        let payload_sha256 = format!("sha256:{:x}", Sha256::digest(&payload));
        let manifest_name = format!("segment-{first:020}-{last:020}.manifest.json");
        let manifest = directory.join(&manifest_name);
        if segment.exists() || manifest.exists() {
            return Err(RuntimeError::InvalidState(format!(
                "immutable journal archive segment already exists: {}",
                segment.display()
            )));
        }
        let previous_manifest_sha256 = fs::read_dir(&directory)?
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .is_some_and(|name| name.to_string_lossy().ends_with(".manifest.json"))
            })
            .max()
            .map(fs::read)
            .transpose()?
            .map(|bytes| format!("sha256:{:x}", Sha256::digest(bytes)));
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut archive_file = options.open(&segment)?;
        archive_file.write_all(&payload)?;
        archive_file.sync_all()?;
        let archive_manifest = JournalArchiveManifest {
            schema_version: "1".into(),
            segment: segment_name,
            first_revision: first,
            last_revision: last,
            event_count: archived.len() as u64,
            payload_sha256: payload_sha256.clone(),
            previous_manifest_sha256: previous_manifest_sha256.clone(),
            created_at: Utc::now(),
        };
        atomic_json(&manifest, &serde_json::to_value(&archive_manifest)?)?;
        replace_event_file_atomic(&self.events_path(), retained)?;
        let rollback_metadata = directory.join(format!("rollback-{first:020}-{last:020}.json"));
        atomic_json(
            &rollback_metadata,
            &serde_json::json!({
                "schemaVersion": "1",
                "segment": segment,
                "manifest": manifest,
                "activeJournal": self.events_path(),
                "restoreBeforeRevision": retained.first().map(|event| event.revision),
                "checkpointDirectory": self.checkpoint_dir(),
            }),
        )?;
        let state = self.fold_authority_events(&self.events()?)?;
        self.persist_folded_checkpoint(&state)?;
        File::open(&directory)?.sync_all()?;
        Ok(Some(JournalArchiveSummary {
            segment,
            manifest,
            archived_events: archived.len() as u64,
            retained_events: retained.len() as u64,
            payload_sha256,
            previous_manifest_sha256,
            rollback_metadata,
        }))
    }

    fn fold_authority_events(&self, events: &[Event]) -> Result<RuntimeState> {
        if self.project_document().is_some() {
            project_document::fold_project_events(events)
        } else {
            replay_events(events)
        }
    }

    /// Import a replicated journal only when it is a valid strict extension of
    /// the local causal chain. Divergent/offline branches are rejected rather
    /// than resolved by timestamp or CRDT map order.
    pub fn import_events(&self, incoming: &[Event]) -> Result<RuntimeState> {
        fs::create_dir_all(self.journal_root())?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?;
        lock.lock_exclusive()?;
        self.recover_journal_tail_locked()?;
        self.reconcile_project_document()?;
        let local = self.events()?;
        let mut incoming = incoming.to_vec();
        incoming.sort_by_key(|event| event.revision);
        let mut imported = self.fold_authority_events(&incoming)?;
        self.decorate_replica_view(&mut imported);
        if local.len() > incoming.len() || local != incoming[..local.len()] {
            return Err(RuntimeError::InvalidState(
                "replicated journal is not a strict extension of local history".into(),
            ));
        }
        if incoming.len() == local.len() {
            return Ok(imported);
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.events_path())?;
        for event in &incoming[local.len()..] {
            serde_json::to_writer(&mut file, event)?;
            file.write_all(b"\n")?;
        }
        file.sync_data()?;
        File::open(self.journal_root())?.sync_all()?;
        if let Some(document) = self.project_document() {
            document.ingest_events(&incoming)?;
        }
        Ok(imported)
    }

    pub fn initialize(
        &self,
        project_id: impl Into<String>,
        run_id: impl Into<String>,
        actor: Actor,
    ) -> Result<RuntimeState> {
        self.ensure_writable_replica()?;
        fs::create_dir_all(self.journal_root())?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?;
        lock.lock_exclusive()?;
        self.recover_journal_tail_locked()?;
        self.reconcile_project_document()?;
        if !self.events()?.is_empty() {
            return Err(RuntimeError::AlreadyInitialized);
        }
        self.append_unchecked(
            RuntimeState::default(),
            project_id.into(),
            run_id.into(),
            actor,
            Uuid::new_v4().to_string(),
            EventKind::RunInitialized {
                initial_state: LifecycleState::Ready,
                exact_next_work: None,
                plan_revision: 1,
            },
        )
    }

    pub fn initialize_from_legacy(
        &self,
        project_id: impl Into<String>,
        run_id: impl Into<String>,
        actor: Actor,
        initial_state: LifecycleState,
        exact_next_work: Option<String>,
        plan_revision: u64,
    ) -> Result<RuntimeState> {
        fs::create_dir_all(self.journal_root())?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?;
        lock.lock_exclusive()?;
        self.recover_journal_tail_locked()?;
        self.reconcile_project_document()?;
        if !self.events()?.is_empty() {
            return Err(RuntimeError::AlreadyInitialized);
        }
        self.append_unchecked(
            RuntimeState::default(),
            project_id.into(),
            run_id.into(),
            actor,
            Uuid::new_v4().to_string(),
            EventKind::RunInitialized {
                initial_state,
                exact_next_work,
                plan_revision: plan_revision.max(1),
            },
        )
    }

    pub fn append(
        &self,
        actor: Actor,
        expected_revision: u64,
        kind: EventKind,
    ) -> Result<RuntimeState> {
        self.append_command(actor, expected_revision, Uuid::new_v4().to_string(), kind)
    }

    pub fn append_command(
        &self,
        actor: Actor,
        expected_revision: u64,
        command_id: impl Into<String>,
        kind: EventKind,
    ) -> Result<RuntimeState> {
        self.ensure_writable_replica()?;
        let command_id = command_id.into();
        if command_id.trim().is_empty() {
            return Err(RuntimeError::InvalidState(
                "commandId must not be empty".into(),
            ));
        }
        fs::create_dir_all(self.journal_root())?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?;
        lock.lock_exclusive()?;
        self.recover_journal_tail_locked()?;
        self.reconcile_project_document()?;
        let events = self.events()?;
        let state = self.fold_authority_events(&events)?;
        if state.revision == 0 {
            return Err(RuntimeError::NotInitialized);
        }
        if state.command_revisions.contains_key(&command_id) {
            let mut state = state;
            self.decorate_replica_view(&mut state);
            return Ok(state);
        }
        if state.revision != expected_revision {
            return Err(RuntimeError::RevisionConflict {
                expected: expected_revision,
                actual: state.revision,
            });
        }
        let project_id = state.project_id.clone();
        let run_id = state.run_id.clone();
        self.append_unchecked(state, project_id, run_id, actor, command_id, kind)
    }

    pub fn execute_command(&self, envelope: CommandEnvelope) -> Result<CommandResult> {
        self.ensure_writable_replica()?;
        if !matches!(envelope.schema_version.as_str(), "1" | "2") {
            return Err(RuntimeError::InvalidState(format!(
                "unsupported command schemaVersion {}",
                envelope.schema_version
            )));
        }
        if envelope.command_id.trim().is_empty() {
            return Err(RuntimeError::InvalidState(
                "commandId must not be empty".into(),
            ));
        }

        // The command envelope must be validated against the same state that
        // is used to prepare and append its event.  Holding one exclusive
        // flock across read, replay, validation, preparation, append, and
        // fsync prevents two processes from both preparing revision N + 1
        // from the same frontier.
        fs::create_dir_all(self.journal_root())?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?;
        lock.lock_exclusive()?;
        self.recover_journal_tail_locked()?;
        self.reconcile_project_document()?;
        let events = self.events()?;
        let state = self.fold_authority_events(&events)?;
        if state.revision == 0 {
            return Err(RuntimeError::NotInitialized);
        }
        if envelope.project_id != state.project_id {
            return Err(RuntimeError::ProjectMismatch {
                supplied: envelope.project_id,
                current: state.project_id,
            });
        }
        if envelope.run_id != state.run_id {
            return Err(RuntimeError::RunMismatch {
                supplied: envelope.run_id,
                current: state.run_id,
            });
        }
        if let Some(committed_revision) = state.command_revisions.get(&envelope.command_id).copied()
        {
            let mut state = state;
            self.decorate_replica_view(&mut state);
            return Ok(CommandResult {
                command_id: envelope.command_id,
                committed_revision,
                duplicate: true,
                state,
                apply_error: None,
            });
        }
        validate_command_frontier(&state, &envelope)?;
        validate_claim_write(&state, &envelope.actor, &envelope.command)?;
        self.validate_replica_write(&state, &envelope.actor, &envelope.command)?;
        let kind = prepare_command_event(&state, &envelope.actor, &envelope.command)?;
        let project_id = state.project_id.clone();
        let run_id = state.run_id.clone();
        let command_id = envelope.command_id;
        let next = self.append_unchecked(
            state,
            project_id,
            run_id,
            envelope.actor,
            command_id.clone(),
            kind,
        )?;
        Ok(CommandResult {
            command_id,
            committed_revision: next.revision,
            duplicate: false,
            state: next,
            apply_error: None,
        })
    }

    /// Validate a versioned command against a caller-supplied committed state
    /// and return the signed event that may be proposed to consensus. This
    /// method never mutates JSONL or projections.
    pub fn prepare_signed_command(
        &self,
        state: &KbdStateV2,
        envelope: CommandEnvelope,
    ) -> Result<Event> {
        self.ensure_writable_replica()?;
        if !matches!(envelope.schema_version.as_str(), "1" | "2") {
            return Err(RuntimeError::InvalidState(format!(
                "unsupported command schemaVersion {}",
                envelope.schema_version
            )));
        }
        if envelope.command_id.trim().is_empty() {
            return Err(RuntimeError::InvalidState(
                "commandId must not be empty".into(),
            ));
        }
        if envelope.project_id != state.project_id {
            return Err(RuntimeError::ProjectMismatch {
                supplied: envelope.project_id,
                current: state.project_id.clone(),
            });
        }
        if envelope.run_id != state.run_id {
            return Err(RuntimeError::RunMismatch {
                supplied: envelope.run_id,
                current: state.run_id.clone(),
            });
        }
        if state.command_revisions.contains_key(&envelope.command_id) {
            return Err(RuntimeError::DuplicateCommand(envelope.command_id));
        }
        validate_command_frontier(state, &envelope)?;
        validate_claim_write(state, &envelope.actor, &envelope.command)?;
        self.validate_replica_write(state, &envelope.actor, &envelope.command)?;
        let kind = prepare_command_event(state, &envelope.actor, &envelope.command)?;
        let replica_head = state.replica_heads.get(&self.replica_id);
        let mut event = Event {
            schema_version: EVENT_SCHEMA_VERSION.into(),
            project_id: state.project_id.clone(),
            replica_id: self.replica_id.clone(),
            run_id: state.run_id.clone(),
            event_id: Uuid::new_v4().to_string(),
            command_id: Some(envelope.command_id),
            revision: state.frontier.derived_revision().saturating_add(1),
            expected_revision: state.revision,
            lamport: state.frontier.next_lamport(&self.replica_id),
            frontier: state.frontier.clone(),
            causal_parent: replica_head.map(|head| head.event_id.clone()),
            actor_id: envelope.actor.id.clone(),
            actor: envelope.actor,
            timestamp: Utc::now(),
            kind,
            previous_hash: replica_head.map(|head| head.integrity_hash.clone()),
            migration_provenance: None,
            integrity_hash: String::new(),
            signer_key_id: None,
            signer_public_key: None,
            signature: None,
        };
        event.seal(&self.device_signer()?)?;
        Ok(event)
    }

    fn ensure_writable_replica(&self) -> Result<()> {
        if self.read_only {
            return Err(RuntimeError::ReplicaReadOnly {
                replica_id: self.replica_id.clone(),
                reason: "registry classification forbids authoritative writes".into(),
            });
        }
        Ok(())
    }

    fn decorate_replica_view(&self, state: &mut RuntimeState) {
        let local_head = git_stdout(&self.project_root, &["rev-parse", "HEAD"]);
        let active_path_status = match state.active_path.commit.as_deref() {
            Some(commit)
                if git_success(
                    &self.project_root,
                    &["cat-file", "-e", &format!("{commit}^{{commit}}")],
                ) =>
            {
                ReplicaCommitStatus::Current
            }
            Some(_) => ReplicaCommitStatus::AheadOfMe,
            None => ReplicaCommitStatus::Unknown,
        };
        let submodules = state
            .submodule_pins
            .iter()
            .map(|(path, pin)| {
                let child_root = self.project_root.join(path);
                let child_head = git_stdout(&child_root, &["rev-parse", "HEAD"]);
                let status = match child_head {
                    Some(ref head) if head == &pin.gitlink_sha => SubmoduleChildStatus::Current,
                    Some(ref head)
                        if git_success(
                            &child_root,
                            &["merge-base", "--is-ancestor", &pin.gitlink_sha, head],
                        ) =>
                    {
                        SubmoduleChildStatus::AheadOfParent
                    }
                    Some(_) => SubmoduleChildStatus::Diverged,
                    None => SubmoduleChildStatus::Unavailable,
                };
                (path.clone(), status)
            })
            .collect();
        state.replica_view = Some(ReplicaView {
            replica_id: self.replica_id.clone(),
            local_head,
            active_path_status,
            submodules,
        });
    }

    fn validate_replica_write(
        &self,
        state: &RuntimeState,
        actor: &Actor,
        command: &CommandKind,
    ) -> Result<()> {
        validate_replica_write_state(state, actor, command)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn import_legacy_state(
        &self,
        actor: Actor,
        expected_revision: u64,
        command_id: impl Into<String>,
        phases: BTreeMap<String, Phase>,
        active_path: ActivePath,
        completion: BTreeMap<CompletionDimension, Completion>,
        decisions: BTreeMap<String, Decision>,
        blockers: BTreeMap<String, Blocker>,
    ) -> Result<RuntimeState> {
        self.append_command(
            actor,
            expected_revision,
            command_id,
            EventKind::LegacyStateImported {
                phases,
                active_path,
                completion,
                decisions,
                blockers,
            },
        )
    }

    pub fn define_phase(
        &self,
        actor: Actor,
        context: MutationContext,
        phase: Phase,
    ) -> Result<RuntimeState> {
        self.append_command(
            actor,
            context.expected_revision,
            context.command_id,
            EventKind::PhaseDefined { phase },
        )
    }

    pub fn enter_stage(
        &self,
        actor: Actor,
        context: MutationContext,
        phase_id: impl Into<String>,
        stage: Stage,
    ) -> Result<RuntimeState> {
        self.append_command(
            actor,
            context.expected_revision,
            context.command_id,
            EventKind::StageEntered {
                phase_id: phase_id.into(),
                stage,
            },
        )
    }

    pub fn register_change(
        &self,
        actor: Actor,
        context: MutationContext,
        phase_id: impl Into<String>,
        change: Change,
    ) -> Result<RuntimeState> {
        self.append_command(
            actor,
            context.expected_revision,
            context.command_id,
            EventKind::ChangeRegistered {
                phase_id: phase_id.into(),
                change,
            },
        )
    }

    pub fn register_task(
        &self,
        actor: Actor,
        context: MutationContext,
        phase_id: impl Into<String>,
        change_id: impl Into<String>,
        task: Task,
    ) -> Result<RuntimeState> {
        self.append_command(
            actor,
            context.expected_revision,
            context.command_id,
            EventKind::TaskRegistered {
                phase_id: phase_id.into(),
                change_id: change_id.into(),
                task,
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn transition_task(
        &self,
        actor: Actor,
        context: MutationContext,
        phase_id: impl Into<String>,
        change_id: impl Into<String>,
        task_id: impl Into<String>,
        to: WorkStatus,
        summary: Option<String>,
    ) -> Result<RuntimeState> {
        let phase_id = phase_id.into();
        let change_id = change_id.into();
        let task_id = task_id.into();
        let state = self.replay()?;
        let from = state
            .phases
            .get(&phase_id)
            .and_then(|phase| phase.changes.get(&change_id))
            .and_then(|change| change.tasks.get(&task_id))
            .map(|task| task.status.clone())
            .ok_or_else(|| RuntimeError::WorkItemNotFound {
                kind: "task",
                id: task_id.clone(),
            })?;
        self.append_command(
            actor,
            context.expected_revision,
            context.command_id,
            EventKind::TaskTransitioned {
                phase_id,
                change_id,
                task_id,
                from,
                to,
                summary,
            },
        )
    }

    pub fn set_active_path(
        &self,
        actor: Actor,
        context: MutationContext,
        active_path: ActivePath,
        exact_next_work: Option<String>,
    ) -> Result<RuntimeState> {
        self.append_command(
            actor,
            context.expected_revision,
            context.command_id,
            EventKind::ActivePathChanged {
                active_path,
                exact_next_work,
            },
        )
    }

    pub fn update_completion(
        &self,
        actor: Actor,
        context: MutationContext,
        dimension: CompletionDimension,
        completion: Completion,
    ) -> Result<RuntimeState> {
        self.append_command(
            actor,
            context.expected_revision,
            context.command_id,
            EventKind::CompletionUpdated {
                dimension,
                completion,
            },
        )
    }

    pub fn record_decision(
        &self,
        actor: Actor,
        context: MutationContext,
        decision: Decision,
    ) -> Result<RuntimeState> {
        self.append_command(
            actor,
            context.expected_revision,
            context.command_id,
            EventKind::DecisionRecorded { decision },
        )
    }

    pub fn record_blocker(
        &self,
        actor: Actor,
        context: MutationContext,
        blocker: Blocker,
    ) -> Result<RuntimeState> {
        self.append_command(
            actor,
            context.expected_revision,
            context.command_id,
            EventKind::BlockerRecorded { blocker },
        )
    }

    pub fn clear_blocker(
        &self,
        actor: Actor,
        context: MutationContext,
        blocker_id: impl Into<String>,
        resolution: impl Into<String>,
    ) -> Result<RuntimeState> {
        self.append_command(
            actor,
            context.expected_revision,
            context.command_id,
            EventKind::BlockerCleared {
                blocker_id: blocker_id.into(),
                resolution: resolution.into(),
            },
        )
    }

    pub fn enroll_device(
        &self,
        actor: Actor,
        context: MutationContext,
        device: DeviceRecord,
    ) -> Result<RuntimeState> {
        if actor.kind != ActorKind::Operator {
            return Err(RuntimeError::InvalidState(
                "only an operator may enroll a device".into(),
            ));
        }
        self.append_command(
            actor,
            context.expected_revision,
            context.command_id,
            EventKind::DeviceEnrolled { device },
        )
    }

    pub fn revoke_device(
        &self,
        actor: Actor,
        context: MutationContext,
        key_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<RuntimeState> {
        if actor.kind != ActorKind::Operator {
            return Err(RuntimeError::InvalidState(
                "only an operator may revoke a device".into(),
            ));
        }
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(RuntimeError::ReasonRequired);
        }
        self.append_command(
            actor,
            context.expected_revision,
            context.command_id,
            EventKind::DeviceRevoked {
                key_id: key_id.into(),
                reason,
            },
        )
    }

    pub fn rotate_device_key(
        &self,
        actor: Actor,
        context: MutationContext,
        previous_key_id: impl Into<String>,
        replacement: DeviceRecord,
    ) -> Result<RuntimeState> {
        if actor.kind != ActorKind::Operator {
            return Err(RuntimeError::InvalidState(
                "only an operator may rotate a device key".into(),
            ));
        }
        self.append_command(
            actor,
            context.expected_revision,
            context.command_id,
            EventKind::DeviceKeyRotated {
                previous_key_id: previous_key_id.into(),
                replacement,
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn append_unchecked(
        &self,
        mut state: RuntimeState,
        project_id: String,
        run_id: String,
        actor: Actor,
        command_id: String,
        kind: EventKind,
    ) -> Result<RuntimeState> {
        let replica_head = state.replica_heads.get(&self.replica_id);
        let mut event = Event {
            schema_version: EVENT_SCHEMA_VERSION.into(),
            project_id,
            replica_id: self.replica_id.clone(),
            run_id,
            event_id: Uuid::new_v4().to_string(),
            command_id: Some(command_id),
            revision: state.frontier.derived_revision().saturating_add(1),
            expected_revision: state.revision,
            lamport: state.frontier.next_lamport(&self.replica_id),
            frontier: state.frontier.clone(),
            causal_parent: replica_head.map(|head| head.event_id.clone()),
            actor_id: actor.id.clone(),
            actor,
            timestamp: Utc::now(),
            kind,
            previous_hash: replica_head.map(|head| head.integrity_hash.clone()),
            migration_provenance: None,
            integrity_hash: String::new(),
            signer_key_id: None,
            signer_public_key: None,
            signature: None,
        };
        let signer = self.device_signer()?;
        event.seal(&signer)?;
        state.apply(&event)?;
        fs::create_dir_all(self.journal_root())?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.events_path())?;
        serde_json::to_writer(&mut file, &event)?;
        file.write_all(b"\n")?;
        file.sync_data()?;
        File::open(self.journal_root())?.sync_all()?;
        if let Some(document) = self.project_document() {
            document.ingest_events(std::slice::from_ref(&event))?;
        }
        self.persist_folded_checkpoint(&state)?;
        self.decorate_replica_view(&mut state);
        Ok(state)
    }

    pub fn transition(
        &self,
        actor: Actor,
        expected_revision: u64,
        to: LifecycleState,
        reason: impl Into<String>,
    ) -> Result<RuntimeState> {
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(RuntimeError::ReasonRequired);
        }
        let state = self.replay()?;
        self.append(
            actor,
            expected_revision,
            EventKind::LifecycleTransition {
                from: state.lifecycle,
                to,
                reason,
            },
        )
    }

    pub fn pause(
        &self,
        actor: Actor,
        expected_revision: u64,
        mut checkpoint: Checkpoint,
    ) -> Result<RuntimeState> {
        if checkpoint.reason.trim().is_empty() {
            return Err(RuntimeError::ReasonRequired);
        }
        let state = self.replay()?;
        checkpoint.previous_state = state.lifecycle.clone();
        checkpoint.plan_revision = state.plan_revision;
        self.append(
            actor,
            expected_revision,
            EventKind::PauseCheckpointed { checkpoint },
        )
    }

    pub fn resume(
        &self,
        actor: Actor,
        expected_revision: u64,
        plan_revision: u64,
    ) -> Result<RuntimeState> {
        let state = self.replay()?;
        if state.plan_revision != plan_revision {
            return Err(RuntimeError::PlanRevision {
                supplied: plan_revision,
                current: state.plan_revision,
            });
        }
        self.transition(
            actor,
            expected_revision,
            LifecycleState::Running,
            format!("resume plan revision {plan_revision}"),
        )
    }

    pub fn cancel(
        &self,
        actor: Actor,
        expected_revision: u64,
        reason: impl Into<String>,
    ) -> Result<RuntimeState> {
        self.transition(actor, expected_revision, LifecycleState::Cancelled, reason)
    }

    pub fn revise_plan(
        &self,
        actor: Actor,
        expected_revision: u64,
        reason: impl Into<String>,
        exact_next_work: Option<String>,
    ) -> Result<RuntimeState> {
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(RuntimeError::ReasonRequired);
        }
        let state = self.replay()?;
        self.append(
            actor,
            expected_revision,
            EventKind::PlanRevised {
                from_revision: state.plan_revision,
                to_revision: state.plan_revision + 1,
                reason,
                superseded_next_work: state.exact_next_work,
                exact_next_work,
            },
        )
    }

    pub fn write_compatibility_projections(&self) -> Result<()> {
        self.write_compatibility_projections_inner(false)
    }

    /// Projection during an explicit, backed-up migration.
    ///
    /// Relaxes the ownership guard: `migrate_legacy_ledgers` has already copied
    /// every ledger into `migration-backups/`, and converting them is the whole
    /// point of the operation. The routine path stays strict because it runs
    /// unattended on every transition.
    fn write_compatibility_projections_migrating(&self) -> Result<()> {
        self.write_compatibility_projections_inner(true)
    }

    fn write_compatibility_projections_inner(&self, migrating: bool) -> Result<()> {
        fs::create_dir_all(self.journal_root())?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?;
        lock.lock_exclusive()?;
        let state = self.replay()?;
        if state.revision == 0 {
            return Err(RuntimeError::NotInitialized);
        }
        let projection_time = self
            .events()?
            .last()
            .map(|event| event.timestamp)
            .ok_or(RuntimeError::NotInitialized)?;
        self.write_compatibility_projections_from_state_inner(&state, projection_time, migrating)
    }

    /// Render revision-stamped compatibility files from already committed
    /// state. Consensus services use this after applying a log entry; the
    /// files are projections only and are never read back as authority.
    pub fn write_compatibility_projections_from_state(
        &self,
        state: &RuntimeState,
        projection_time: DateTime<Utc>,
    ) -> Result<()> {
        self.write_compatibility_projections_from_state_inner(state, projection_time, false)
    }

    fn write_compatibility_projections_from_state_inner(
        &self,
        state: &KbdStateV2,
        projection_time: DateTime<Utc>,
        migrating: bool,
    ) -> Result<()> {
        if state.revision == 0 {
            return Err(RuntimeError::NotInitialized);
        }
        let kbd_root = self.project_root.join(".kbd-orchestrator");
        let active_phase_id = state
            .active_path
            .phase_id
            .clone()
            .or_else(|| state.phases.keys().next().cloned());
        let phase_path_ids = if state.active_path.phase_path.is_empty() {
            active_phase_id.iter().cloned().collect::<Vec<_>>()
        } else {
            state.active_path.phase_path.clone()
        };
        let phase_path = phase_path_ids
            .iter()
            .filter_map(|phase_id| state.phases.get(phase_id))
            .map(|phase| phase.slug.clone())
            .collect::<Vec<_>>();
        let waypoint_phase = phase_path
            .first()
            .cloned()
            .unwrap_or_else(|| state.project_id.clone());
        let child_pointer = if phase_path.len() > 1 {
            phase_path.last().cloned()
        } else {
            None
        };
        let implementation = state
            .completion
            .get(&CompletionDimension::Implementation)
            .cloned()
            .unwrap_or_else(Completion::not_tracked);
        let waypoint = serde_json::json!({
            "schemaVersion": "5",
            "generatedBy": "kbd-runtime",
            "sourceRevision": state.revision,
            "derivedRevision": state.revision,
            "frontier": state.frontier,
            "conflictCount": state.conflicts.len(),
            "projectId": state.project_id,
            "runId": state.run_id,
            "path": phase_path,
            "phaseIds": phase_path_ids,
            "activePhaseId": active_phase_id,
            "phase": waypoint_phase,
            "childPointer": child_pointer,
            "change": state.active_path.change_id,
            "currentTask": state.active_path.task_id,
            "status": lifecycle_name(&state.lifecycle),
            "completionMetric": "implementation",
            "implementationCompleted": implementation.completed,
            "implementationTotal": implementation.total,
            "planRevision": state.plan_revision,
            "revision": state.revision,
            "exactNextCommand": state.exact_next_work,
            "updatedAt": projection_time
        });
        atomic_json(&kbd_root.join("current-waypoint.json"), &waypoint)?;

        // Paths already warned about, PROCESS-WIDE. A per-call set would reset
        // on every transition and reproduce the flood it exists to stop.
        static WARNED: std::sync::OnceLock<std::sync::Mutex<HashSet<PathBuf>>> =
            std::sync::OnceLock::new();
        let warned_lock = WARNED.get_or_init(|| std::sync::Mutex::new(HashSet::new()));

        for phase in state.phases.values() {
            let phase_dir = phase_projection_directory(&kbd_root, state, phase)?;
            let progress_path = phase_dir.join("progress.json");
            // Poisoned lock is not a reason to fail a projection: fall back to
            // warning every time rather than aborting the write path.
            let mut warned = match warned_lock.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };

            // REFUSE to clobber a projection this runtime did not author.
            //
            // `atomic_json` replaces wholesale — no merge, no read-before-write.
            // This loop runs over EVERY phase in runtime state on EVERY
            // transition, so a `progress.json` maintained by anything else (a
            // script, an agent, a hand edit) was silently destroyed and replaced
            // with a runtime-derived value.
            //
            // That is committed data loss with no warning: the file still
            // exists and still parses, so the next reader trusts it. Reported
            // 2026-08-01 after a phase lost its ledger mid-run and the operator
            // had to choose between abandoning canonical tracking and running a
            // manual restore-and-verify cycle after all 24 changes.
            //
            // The marker is `generatedBy: "kbd-runtime"`. A file without it was
            // written by someone else and is not ours to overwrite.
            if !projection_is_writable(&progress_path, migrating) {
                // stderr, not silence: a guard that skips without saying so
                // trades visible data loss for invisible staleness, which is
                // the same class of failure with a longer fuse. This crate has
                // no logging dependency, so eprintln! is the honest channel.
                //
                // ONCE PER PATH, though. This loop runs over every phase on
                // every transition, and an unconditional warn produced
                // thousands of identical lines in the daemon's stderr —
                // burying exactly the diagnostics someone would need. That is
                // the same failure the launchd ThrottleInterval fix addressed:
                // a message repeated without limit is indistinguishable from
                // no message at all.
                if warned.insert(progress_path.clone()) {
                    eprintln!(
                        "kbd-runtime: refusing to overwrite {} — it has no \
                     `generatedBy: \"kbd-runtime\"` marker, so it was written \
                     by something else. Leaving it untouched. Delete the file \
                     if the runtime should own it.",
                        progress_path.display()
                    );
                }
                continue;
            }

            atomic_json(
                &progress_path,
                &phase_progress_projection(state, phase, projection_time),
            )?;
            atomic_text(
                &phase_dir.join("tasks.md"),
                &phase_tasks_projection(state, phase),
            )?;
        }

        let position = serde_json::json!({
            "schemaVersion": "1",
            "generatedBy": "kbd-runtime",
            "sourceRevision": state.revision,
            "derivedRevision": state.revision,
            "frontier": state.frontier,
            "conflictCount": state.conflicts.len(),
            "updatedAt": projection_time,
            "cursor": active_cursor(state),
            "root": {
                "type": "phase",
                "id": state.project_id,
                "status": lifecycle_name(&state.lifecycle),
                "progress": {
                    "done": implementation.completed,
                    "total": implementation.total
                },
                "children": state.phases.values()
                    .filter(|phase| phase.parent_phase_id.is_none())
                    .map(|phase| position_phase_node(state, phase))
                    .collect::<Vec<_>>(),
                "annotations": []
            }
        });
        atomic_json(&kbd_root.join("position.json"), &position)?;
        Ok(())
    }

    /// Compare repository compatibility files with a clean render of the
    /// committed journal. This never imports legacy writes and cannot mutate
    /// canonical state or create writer authority.
    pub fn compatibility_projection_mismatches(&self) -> Result<Vec<PathBuf>> {
        let state = self.replay()?;
        if state.revision == 0 {
            return Err(RuntimeError::NotInitialized);
        }
        let projection_time = self
            .events()?
            .last()
            .map(|event| event.timestamp)
            .ok_or(RuntimeError::NotInitialized)?;
        self.compatibility_projection_mismatches_from_state(&state, projection_time)
    }

    pub fn compatibility_projection_mismatches_from_state(
        &self,
        state: &KbdStateV2,
        projection_time: DateTime<Utc>,
    ) -> Result<Vec<PathBuf>> {
        if state.revision == 0 {
            return Err(RuntimeError::NotInitialized);
        }
        let comparison_root =
            std::env::temp_dir().join(format!("prometheus-kbd-shadow-{}", Uuid::new_v4()));
        let comparison_runtime = Self {
            root: comparison_root.join("runtime"),
            project_root: comparison_root.clone(),
            replica_id: self.replica_id.clone(),
            key_storage: KeyStorage::LegacyRuntimeFile,
            read_only: false,
        };
        comparison_runtime.write_compatibility_projections_from_state(state, projection_time)?;
        let expected_root = comparison_root.join(".kbd-orchestrator");
        let actual_root = self.project_root.join(".kbd-orchestrator");
        let mut expected_paths = Vec::new();
        collect_all_files(&expected_root, &mut expected_paths)?;
        let mut mismatches = Vec::new();
        for expected in expected_paths {
            let relative = expected.strip_prefix(&expected_root).unwrap_or(&expected);
            let actual = actual_root.join(relative);
            if fs::read(&expected).ok() != fs::read(&actual).ok() {
                mismatches.push(relative.to_path_buf());
            }
        }
        let _ = fs::remove_dir_all(&comparison_root);
        mismatches.sort();
        Ok(mismatches)
    }

    pub fn migrate_legacy_ledgers(&self, apply: bool) -> Result<MigrationSummary> {
        let project_root = &self.project_root;
        let manifest = self.project_manifest(apply)?;
        let project_id = manifest
            .as_ref()
            .map(|manifest| manifest.project_id.clone())
            .or_else(|| {
                self.replay()
                    .ok()
                    .filter(|state| state.revision > 0)
                    .map(|state| state.project_id)
            })
            .unwrap_or_else(|| "unassigned-until-apply".into());
        let kbd_root = project_root.join(".kbd-orchestrator");
        let mut paths = Vec::new();
        collect_named_files(&kbd_root.join("phases"), "progress.json", &mut paths)?;
        let backup_directory = if apply {
            let path = self.root.join("migration-backups").join(format!(
                "{}-{}",
                Utc::now().format("%Y%m%dT%H%M%SZ"),
                Uuid::new_v4()
            ));
            fs::create_dir_all(&path)?;
            Some(path)
        } else {
            None
        };
        let mut backup_entries = Vec::new();
        if let Some(backup) = &backup_directory {
            let mut backup_sources = paths.clone();
            for name in [
                "current-waypoint.json",
                "position.json",
                "progress.json",
                "PAUSE",
            ] {
                let path = kbd_root.join(name);
                if path.is_file() {
                    backup_sources.push(path);
                }
            }
            backup_sources.sort();
            backup_sources.dedup();
            for source in backup_sources {
                let relative = source.strip_prefix(&kbd_root).unwrap_or(&source);
                let destination = backup.join("legacy").join(relative);
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent)?;
                }
                let bytes = fs::read(&source)?;
                fs::write(&destination, &bytes)?;
                backup_entries.push(MigrationBackupEntry {
                    source: relative.to_path_buf(),
                    backup: destination
                        .strip_prefix(backup)
                        .unwrap_or(&destination)
                        .to_path_buf(),
                    bytes: bytes.len() as u64,
                    sha256: format!("{:x}", Sha256::digest(&bytes)),
                });
            }
        }
        let mut migrated = 0;
        let mut uncertain = 0;
        let mut invalid = 0;
        let mut alias_conflicts = 0;
        let mut legacy_read_only_phases = 0;
        let mut phases = BTreeMap::new();
        for path in &paths {
            let progress: serde_json::Value = match serde_json::from_reader(File::open(path)?) {
                Ok(progress) => progress,
                Err(_) => {
                    invalid += 1;
                    continue;
                }
            };
            let file_alias_conflict = progress_alias_conflict(&progress);
            alias_conflicts += u64::from(file_alias_conflict);
            let file_uncertain = progress_uncertain_rows(&progress);
            uncertain += file_uncertain;
            if progress["schemaVersion"] != "2" {
                migrated += 1;
            }
            let identity = legacy_phase_identity(&kbd_root, path, &progress);
            let mut phase = legacy_phase(
                &identity.id,
                &progress,
                file_uncertain > 0 || file_alias_conflict,
            );
            phase.slug = identity.slug;
            phase.parent_phase_id = identity.parent_phase_id;
            legacy_read_only_phases += u64::from(phase.legacy_read_only);
            phases.insert(identity.id, phase);
        }
        if apply {
            let waypoint = fs::read(kbd_root.join("current-waypoint.json"))
                .ok()
                .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
                .unwrap_or_else(|| serde_json::json!({}));
            let mut state = self.replay()?;
            if state.revision == 0 {
                let run_id = waypoint
                    .get("runId")
                    .or_else(|| waypoint.get("run_id"))
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("run-{}", Uuid::new_v4()));
                state = self.initialize(
                    project_id.clone(),
                    run_id,
                    Actor::operator("kbd-migration", "prometheus-cli"),
                )?;
            }
            if state.phases.is_empty() && !phases.is_empty() {
                let requested_path = waypoint
                    .get("path")
                    .and_then(serde_json::Value::as_array)
                    .map(|path| {
                        path.iter()
                            .filter_map(serde_json::Value::as_str)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let mut phase_path = resolve_legacy_phase_path(&phases, &requested_path);
                if phase_path.is_empty() {
                    let requested_phase = waypoint.get("phase").and_then(serde_json::Value::as_str);
                    let phase_id = requested_phase
                        .filter(|phase_id| phases.contains_key(*phase_id))
                        .map(str::to_owned)
                        .or_else(|| {
                            let matches = phases
                                .values()
                                .filter(|phase| Some(phase.slug.as_str()) == requested_phase)
                                .map(|phase| phase.id.clone())
                                .collect::<Vec<_>>();
                            (matches.len() == 1).then(|| matches[0].clone())
                        })
                        .or_else(|| {
                            phases
                                .values()
                                .find(|phase| phase.parent_phase_id.is_none())
                                .map(|phase| phase.id.clone())
                        });
                    if let Some(phase_id) = phase_id {
                        phase_path = legacy_phase_chain(&phases, &phase_id);
                    }
                }
                let active_phase = phase_path.last().cloned();
                let active_path = ActivePath {
                    phase_path,
                    phase_id: active_phase,
                    stage_id: waypoint
                        .get("stage")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned),
                    change_id: waypoint
                        .get("change")
                        .or_else(|| waypoint.get("active_change"))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned),
                    task_id: waypoint
                        .get("currentTask")
                        .or_else(|| waypoint.get("current_task"))
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned),
                    commit: None,
                };
                let mut completion = CompletionDimension::all()
                    .into_iter()
                    .map(|dimension| (dimension, Completion::not_tracked()))
                    .collect::<BTreeMap<_, _>>();
                let completed = phases
                    .values()
                    .flat_map(|phase| phase.changes.values())
                    .filter(|change| change.implementation_status.is_complete())
                    .count() as u64;
                let total = phases
                    .values()
                    .map(|phase| phase.changes.len() as u64)
                    .sum();
                completion.insert(
                    CompletionDimension::Implementation,
                    Completion {
                        completed,
                        total,
                        status: if total > 0 && completed == total {
                            WorkStatus::Complete
                        } else if completed > 0 {
                            WorkStatus::InProgress
                        } else {
                            WorkStatus::Pending
                        },
                        summary: Some("Imported from compatibility ledgers".into()),
                        blockers: Vec::new(),
                    },
                );
                self.import_legacy_state(
                    Actor::operator("kbd-migration", "prometheus-cli"),
                    state.revision,
                    format!("legacy-import-v2-{}", state.run_id),
                    phases,
                    active_path,
                    completion,
                    BTreeMap::new(),
                    BTreeMap::new(),
                )?;
            }
            self.write_compatibility_projections_migrating()?;
        }
        let backup_manifest = if let Some(backup) = &backup_directory {
            let path = backup.join("manifest.json");
            let backup_manifest = MigrationBackupManifest {
                schema_version: "1".into(),
                project_id: project_id.clone(),
                created_at: Utc::now(),
                files: backup_entries,
            };
            atomic_json(&path, &serde_json::to_value(backup_manifest)?)?;
            Some(path)
        } else {
            None
        };
        Ok(MigrationSummary {
            project_id,
            progress_files: paths.len() as u64,
            migrated_progress_files: migrated,
            uncertain_rows: uncertain,
            invalid_files: invalid,
            alias_conflicts,
            legacy_read_only_phases,
            stale_projections: projection_mismatch_count(&kbd_root),
            unreplayable_history: self.events_path().exists() && self.replay().is_err(),
            backup_directory,
            backup_manifest,
        })
    }
}

fn lifecycle_name(state: &LifecycleState) -> &'static str {
    match state {
        LifecycleState::Ready => "ready",
        LifecycleState::Running => "running",
        LifecycleState::PauseRequested => "pause_requested",
        LifecycleState::Paused => "paused",
        LifecycleState::Blocked => "blocked",
        LifecycleState::Completed => "completed",
        LifecycleState::Cancelled => "cancelled",
        LifecycleState::Failed => "failed",
    }
}

fn active_cursor(state: &RuntimeState) -> Vec<String> {
    let phase_ids = if state.active_path.phase_path.is_empty() {
        state.active_path.phase_id.iter().cloned().collect()
    } else {
        state.active_path.phase_path.clone()
    };
    let mut cursor = phase_ids
        .iter()
        .filter_map(|phase_id| state.phases.get(phase_id))
        .map(|phase| phase.slug.clone())
        .collect::<Vec<_>>();
    cursor.extend(
        [
            state.active_path.stage_id.as_ref(),
            state.active_path.change_id.as_ref(),
            state.active_path.task_id.as_ref(),
        ]
        .into_iter()
        .flatten()
        .cloned(),
    );
    cursor
}

fn work_status_name(status: &WorkStatus) -> &'static str {
    match status {
        WorkStatus::Pending => "PENDING",
        WorkStatus::InProgress => "IN_PROGRESS",
        WorkStatus::Blocked => "BLOCKED",
        WorkStatus::Complete => "COMPLETE",
        WorkStatus::Cancelled => "SKIPPED",
    }
}

fn legacy_change_status(status: &WorkStatus) -> &'static str {
    if status.is_complete() {
        "DONE"
    } else {
        work_status_name(status)
    }
}

fn count_completion_projection(completion: &Completion) -> serde_json::Value {
    serde_json::json!({
        "completed": completion.completed,
        "total": completion.total,
        "status": work_status_name(&completion.status)
    })
}

fn status_completion_projection(completion: &Completion) -> serde_json::Value {
    serde_json::json!({
        "status": work_status_name(&completion.status),
        "summary": completion.summary,
        "blockers": completion.blockers
    })
}

fn ordered_changes(phase: &Phase) -> Vec<&Change> {
    let mut changes = phase.changes.values().collect::<Vec<_>>();
    changes.sort_by(|left, right| {
        left.sequence
            .cmp(&right.sequence)
            .then_with(|| left.id.cmp(&right.id))
    });
    changes
}

fn ordered_tasks(change: &Change) -> Vec<&Task> {
    let mut tasks = change.tasks.values().collect::<Vec<_>>();
    tasks.sort_by(|left, right| {
        left.sequence
            .cmp(&right.sequence)
            .then_with(|| left.id.cmp(&right.id))
    });
    tasks
}

/// Is this projection file one the runtime wrote, and therefore ours to replace?
///
/// The projection loop replaces `progress.json` wholesale. Before this check it
/// did so for every phase in runtime state unconditionally, which silently
/// destroyed any `progress.json` maintained outside the runtime — a script, an
/// agent, a hand edit. The failure was invisible: the file still existed and
/// still parsed, so the next reader trusted a value that had just been
/// fabricated from partial state.
///
/// May `path` be replaced by a projection? `migrating` relaxes the guard.
///
/// Migration is the one path that legitimately rewrites a ledger it did not
/// author: `migrate_legacy_ledgers` takes a backup first and is an explicit,
/// operator-invoked conversion. The routine projection loop is not — it runs on
/// every transition, unattended.
///
/// Separating the two is the whole fix. The first attempt used one rule for
/// both and had to choose between breaking migration and leaving the bug in
/// place; the repository's own ledgers made that unavoidable, because they
/// carry legacy counters AND a modern `completion` object at the same time.
fn projection_is_writable(path: &Path, migrating: bool) -> bool {
    if !path.exists() {
        return true;
    }
    let Ok(bytes) = fs::read(path) else {
        return false;
    };
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    // Ours: we stamped it.
    if value
        .get("generatedBy")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|who| who == "kbd-runtime")
    {
        return true;
    }

    // A LEGACY ledger is also ours to replace. `migrate_legacy_ledgers` reads
    // these into runtime state and takes a backup first, and the projection
    // loop is what writes them back in the new shape — so refusing here would
    // break migration, not protect anything.
    //
    // Legacy shape is recognised by its snake_case counters, which the current
    // projection does not emit (it uses a nested `completion` object). A file
    // with neither the marker NOR those keys was written by something else
    // entirely, and is the case this guard exists for.
    let has_legacy_counters = value.get("changes_completed").is_some()
        || value.get("changes_total").is_some()
        || value.get("implementation_completed").is_some();

    // A TRUE legacy ledger has the old counters and NOTHING newer. A file that
    // carries both the old counters and a modern `completion` object was
    // written by a current-generation tool that kept the legacy keys for
    // compatibility — that is someone else's live ledger, not a migration
    // candidate.
    //
    // Measured on the file this bug actually destroyed: it had
    // `changes_completed` AND `completion`, so a counters-only check would
    // still have overwritten it. The distinction has to be "old shape only".
    let has_modern_shape = value.get("completion").is_some();

    if migrating {
        // A backup exists, so converting a LEGACY ledger is the point of the
        // operation. But a file with a modern `completion` object and NO legacy
        // counters is not a migration candidate at all — it is already in a
        // current shape and was written by something else. Rewriting it during
        // migration would reintroduce the exact data loss this guard exists to
        // stop, just behind an operator-invoked command instead of an
        // unattended one.
        //
        // Measured: a pure-`completion` ledger was still being rewritten by
        // `kbd migrate --apply` after the first version of this guard.
        return has_legacy_counters;
    }

    // Routine projection replaces NOTHING it did not provably write.
    //
    // The earlier rule here was `has_legacy_counters && !has_modern_shape`,
    // which still returned true for a PURE-LEGACY ledger — old counters, no
    // `completion` key. That is precisely the shape carried by phases which
    // predate the current schema, so the guard protected the modern files and
    // left the oldest ones exposed. Reported as a GitHub issue against this
    // repo; the reporter was right.
    //
    // Measured on this repository when the issue landed: 51 phase ledgers were
    // already `generatedBy: kbd-runtime`, 21 of them empty — overwritten before
    // any guard existed.
    //
    // Legacy CONVERSION is a real need, but it belongs to `migrate_legacy_
    // ledgers`, which takes a backup into `migration-backups/` first and runs
    // only when an operator asks for it. The routine loop runs unattended on
    // every transition and has no backup, so it gets the strict rule: the
    // `generatedBy` marker checked above is the only license to overwrite.
    //
    // `has_legacy_counters` and `has_modern_shape` remain computed because the
    // migrating branch above needs the former; naming the latter keeps the two
    // shapes visible side by side.
    let _ = has_modern_shape;
    false
}

fn phase_progress_projection(
    state: &RuntimeState,
    phase: &Phase,
    updated_at: DateTime<Utc>,
) -> serde_json::Value {
    let completed = phase
        .changes
        .values()
        .filter(|change| change.implementation_status.is_complete())
        .count() as u64;
    let total = phase.changes.len() as u64;
    let implementation_status = if total > 0 && completed == total {
        WorkStatus::Complete
    } else if phase
        .changes
        .values()
        .any(|change| change.implementation_status == WorkStatus::Blocked)
    {
        WorkStatus::Blocked
    } else if completed > 0 {
        WorkStatus::InProgress
    } else {
        WorkStatus::Pending
    };
    let changes = ordered_changes(phase)
        .into_iter()
        .map(|change| {
            let tasks_done = change
                .tasks
                .values()
                .filter(|task| task.status.is_complete())
                .count() as u64;
            let ordered = ordered_tasks(change);
            let last_task_completed = ordered
                .iter()
                .rev()
                .find(|task| task.status.is_complete())
                .map(|task| task.title.clone());
            let next_task_pending = ordered
                .iter()
                .find(|task| !task.status.is_complete())
                .map(|task| task.title.clone());
            serde_json::json!({
                "id": change.id,
                "title": change.title,
                "status": legacy_change_status(&change.status),
                "implementation_status": work_status_name(&change.implementation_status),
                "tasks_total": change.tasks.len(),
                "tasks_done": tasks_done,
                "last_task_completed": last_task_completed,
                "next_task_pending": next_task_pending,
                "blockers": change.tasks.values()
                    .filter(|task| task.status == WorkStatus::Blocked)
                    .map(|task| task.title.clone())
                    .collect::<Vec<_>>()
            })
        })
        .collect::<Vec<_>>();
    let implementation = Completion {
        completed,
        total,
        status: implementation_status,
        summary: None,
        blockers: Vec::new(),
    };
    let dimension = |name: CompletionDimension| {
        state
            .completion
            .get(&name)
            .cloned()
            .unwrap_or_else(Completion::not_tracked)
    };
    let children = state
        .phases
        .values()
        .filter(|candidate| candidate.parent_phase_id.as_deref() == Some(phase.id.as_str()))
        .map(|child| {
            let completed = child
                .changes
                .values()
                .filter(|change| change.implementation_status.is_complete())
                .count() as u64;
            (
                child.slug.clone(),
                serde_json::json!({
                    "status": legacy_change_status(&child.status),
                    "changes_completed": completed,
                    "changes_total": child.changes.len(),
                    "implementation_completed": completed,
                    "implementation_total": child.changes.len(),
                    "certification_status": "NOT_TRACKED",
                    "handoff": serde_json::Value::Null,
                    "completed_at": serde_json::Value::Null
                }),
            )
        })
        .collect::<BTreeMap<_, _>>();
    serde_json::json!({
        "schemaVersion": "2",
        "generatedBy": "kbd-runtime",
        "sourceRevision": state.revision,
        "derivedRevision": state.revision,
        "frontier": state.frontier,
        "conflictCount": state.conflicts.len(),
        "phase": phase.slug,
        "phaseId": phase.id,
        "parentPhase": phase.parent_phase_id,
        "last_updated": updated_at,
        "last_updated_by": "kbd-runtime",
        "changes_total": total,
        "changes_completed": completed,
        "implementation_total": total,
        "implementation_completed": completed,
        "completion": {
            "primaryCounter": "implementation",
            "implementation": count_completion_projection(&implementation),
            "evidence": status_completion_projection(&dimension(CompletionDimension::Evidence)),
            "certification": status_completion_projection(&dimension(CompletionDimension::Certification)),
            "publication": status_completion_projection(&dimension(CompletionDimension::Publication))
        },
        "changes": changes,
        "children": children,
        "migrationStatus": if phase.legacy_read_only {
            serde_json::Value::String("legacy-read-only".into())
        } else {
            serde_json::Value::Null
        }
    })
}

fn phase_projection_directory(
    kbd_root: &Path,
    state: &RuntimeState,
    phase: &Phase,
) -> Result<PathBuf> {
    let mut chain = vec![phase.slug.clone()];
    let mut parent = phase.parent_phase_id.as_ref();
    let mut visited = HashSet::new();
    visited.insert(phase.id.clone());
    while let Some(parent_id) = parent {
        if !visited.insert(parent_id.clone()) {
            return Err(RuntimeError::InvalidState(
                "cycle in phase parent hierarchy".into(),
            ));
        }
        let parent_phase =
            state
                .phases
                .get(parent_id)
                .ok_or_else(|| RuntimeError::WorkItemNotFound {
                    kind: "phase",
                    id: parent_id.clone(),
                })?;
        chain.push(parent_phase.slug.clone());
        parent = parent_phase.parent_phase_id.as_ref();
    }
    chain.reverse();
    let mut directory = kbd_root.join("phases").join(&chain[0]);
    for child in chain.iter().skip(1) {
        directory = directory.join("children").join(child);
    }
    Ok(directory)
}

fn phase_tasks_projection(state: &RuntimeState, phase: &Phase) -> String {
    let mut output = format!(
        "<!-- generated by kbd-runtime; source revision {} -->\n# {} tasks\n\n",
        state.revision, phase.title
    );
    for change in ordered_changes(phase) {
        output.push_str(&format!("## {} — {}\n\n", change.id, change.title));
        if change.tasks.is_empty() {
            output.push_str("- [ ] No tasks registered\n\n");
            continue;
        }
        for task in ordered_tasks(change) {
            let marker = if task.status.is_complete() { "x" } else { " " };
            output.push_str(&format!(
                "- [{marker}] {} — {} ({})\n",
                task.id,
                task.title,
                work_status_name(&task.status)
            ));
        }
        output.push('\n');
    }
    output
}

fn position_phase_node(state: &RuntimeState, phase: &Phase) -> serde_json::Value {
    let completed = phase
        .changes
        .values()
        .filter(|change| change.implementation_status.is_complete())
        .count() as u64;
    let mut children = state
        .phases
        .values()
        .filter(|candidate| candidate.parent_phase_id.as_deref() == Some(phase.id.as_str()))
        .map(|candidate| position_phase_node(state, candidate))
        .collect::<Vec<_>>();
    children.extend(ordered_changes(phase).into_iter().map(|change| {
        let done = change
            .tasks
            .values()
            .filter(|task| task.status.is_complete())
            .count() as u64;
        serde_json::json!({
            "type": "change",
            "id": change.id,
            "status": work_status_name(&change.status),
            "progress": {"done": done, "total": change.tasks.len()}
        })
    }));
    serde_json::json!({
        "type": "phase",
        "id": phase.slug,
        "canonicalId": phase.id,
        "status": work_status_name(&phase.status),
        "progress": {"done": completed, "total": phase.changes.len()},
        "children": children,
        "annotations": []
    })
}

struct LegacyPhaseIdentity {
    id: String,
    slug: String,
    parent_phase_id: Option<String>,
}

fn legacy_phase_identity(
    kbd_root: &Path,
    progress_path: &Path,
    progress: &serde_json::Value,
) -> LegacyPhaseIdentity {
    let mut slugs = Vec::new();
    if let Ok(relative) = progress_path.strip_prefix(kbd_root.join("phases")) {
        let components = relative
            .components()
            .filter_map(|component| match component {
                std::path::Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
                _ => None,
            })
            .collect::<Vec<_>>();
        if components.last().map(String::as_str) == Some("progress.json") {
            let mut index = 0;
            while index + 1 < components.len() {
                slugs.push(components[index].clone());
                index += 1;
                if index + 1 < components.len() {
                    if components[index] != "children" {
                        slugs.clear();
                        break;
                    }
                    index += 1;
                }
            }
        }
    }
    if slugs.is_empty() {
        slugs.push(
            progress["phase"]
                .as_str()
                .map(str::to_owned)
                .or_else(|| {
                    progress_path
                        .parent()
                        .and_then(Path::file_name)
                        .map(|name| name.to_string_lossy().into_owned())
                })
                .unwrap_or_else(|| "legacy-phase".into()),
        );
    }
    let id = slugs.join("::");
    let parent_phase_id = (slugs.len() > 1).then(|| slugs[..slugs.len() - 1].join("::"));
    LegacyPhaseIdentity {
        id,
        slug: slugs
            .last()
            .cloned()
            .unwrap_or_else(|| "legacy-phase".into()),
        parent_phase_id,
    }
}

fn resolve_legacy_phase_path(phases: &BTreeMap<String, Phase>, slugs: &[&str]) -> Vec<String> {
    let mut resolved = Vec::new();
    let mut parent: Option<String> = None;
    for slug in slugs {
        let Some(phase) = phases.values().find(|phase| {
            phase.parent_phase_id.as_deref() == parent.as_deref()
                && (phase.slug == *slug || phase.id == *slug)
        }) else {
            return Vec::new();
        };
        resolved.push(phase.id.clone());
        parent = Some(phase.id.clone());
    }
    resolved
}

fn legacy_phase_chain(phases: &BTreeMap<String, Phase>, phase_id: &str) -> Vec<String> {
    let mut chain = Vec::new();
    let mut current = Some(phase_id);
    while let Some(id) = current {
        let Some(phase) = phases.get(id) else {
            return Vec::new();
        };
        chain.push(phase.id.clone());
        current = phase.parent_phase_id.as_deref();
    }
    chain.reverse();
    chain
}

fn parse_work_status(value: Option<&str>) -> WorkStatus {
    let normalized = value.unwrap_or_default().trim().to_ascii_uppercase();
    let status = normalized
        .split(|character: char| character.is_whitespace() || matches!(character, '(' | '['))
        .next()
        .unwrap_or_default();
    match status {
        "IN_PROGRESS" | "RUNNING" | "EXECUTING" => WorkStatus::InProgress,
        "BLOCKED" | "PAUSED" => WorkStatus::Blocked,
        "COMPLETE" | "COMPLETED" | "DONE" => WorkStatus::Complete,
        "SKIPPED" | "CANCELLED" | "CANCELED" => WorkStatus::Cancelled,
        _ => WorkStatus::Pending,
    }
}

fn legacy_tasks(row: &serde_json::Value) -> BTreeMap<String, Task> {
    let Some(tasks) = row.get("tasks").and_then(serde_json::Value::as_array) else {
        return BTreeMap::new();
    };
    tasks
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let id = value
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .or_else(|| value.as_str().map(str::to_owned))
                .unwrap_or_else(|| format!("legacy-task-{index:03}"));
            let title = value
                .get("title")
                .or_else(|| value.get("summary"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or(&id)
                .to_string();
            let status = parse_work_status(value.get("status").and_then(serde_json::Value::as_str));
            (
                id.clone(),
                Task {
                    id,
                    title,
                    sequence: index as u64,
                    status,
                    summary: value
                        .get("summary")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned),
                },
            )
        })
        .collect()
}

fn legacy_phase(phase_id: &str, progress: &serde_json::Value, mut legacy_read_only: bool) -> Phase {
    let rows = match legacy_changes(progress) {
        Some(serde_json::Value::Array(rows)) => rows
            .iter()
            .enumerate()
            .map(|(index, row)| (index, row.clone()))
            .collect::<Vec<_>>(),
        Some(serde_json::Value::Object(rows)) => rows
            .iter()
            .enumerate()
            .map(|(index, (id, row))| {
                let mut row = row.as_object().cloned().unwrap_or_default();
                row.entry("id").or_insert_with(|| serde_json::json!(id));
                (index, serde_json::Value::Object(row))
            })
            .collect::<Vec<_>>(),
        _ => {
            legacy_read_only = true;
            Vec::new()
        }
    };
    let changes = rows
        .into_iter()
        .map(|(index, row)| {
            let id = row
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
                .or_else(|| row.as_str().map(str::to_owned))
                .unwrap_or_else(|| format!("legacy-change-{index:03}"));
            let implementation_status = parse_work_status(
                row.get("implementation_status")
                    .or_else(|| row.get("implementationStatus"))
                    .or_else(|| row.get("status"))
                    .and_then(serde_json::Value::as_str),
            );
            let tasks = legacy_tasks(&row);
            (
                id.clone(),
                Change {
                    id: id.clone(),
                    title: row
                        .get("title")
                        .or_else(|| row.get("summary"))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or(&id)
                        .to_string(),
                    sequence: index as u64,
                    status: implementation_status.clone(),
                    implementation_status,
                    tasks,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let status = if !changes.is_empty()
        && changes
            .values()
            .all(|change| change.implementation_status.is_complete())
    {
        WorkStatus::Complete
    } else if changes
        .values()
        .any(|change| change.implementation_status == WorkStatus::Blocked)
    {
        WorkStatus::Blocked
    } else if changes
        .values()
        .any(|change| change.implementation_status == WorkStatus::InProgress)
    {
        WorkStatus::InProgress
    } else {
        WorkStatus::Pending
    };
    Phase {
        id: phase_id.to_string(),
        slug: phase_id.to_string(),
        title: progress
            .get("title")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(phase_id)
            .to_string(),
        parent_phase_id: None,
        status,
        stages: BTreeMap::new(),
        changes,
        legacy_read_only,
    }
}

fn progress_uncertain_rows(progress: &serde_json::Value) -> u64 {
    let Some(changes) = legacy_changes(progress) else {
        return 0;
    };
    match changes {
        serde_json::Value::Array(rows) => rows
            .iter()
            .filter(|row| !row.is_object() || row.get("id").is_none())
            .count() as u64,
        serde_json::Value::Object(_) => 0,
        serde_json::Value::Null => 0,
        _ => 1,
    }
}

fn legacy_changes(progress: &serde_json::Value) -> Option<&serde_json::Value> {
    let changes = progress.get("changes");
    let changes_have_rows = match changes {
        Some(serde_json::Value::Array(rows)) => !rows.is_empty(),
        Some(serde_json::Value::Object(rows)) => !rows.is_empty(),
        _ => false,
    };
    if changes_have_rows {
        changes
    } else {
        progress.get("ordered_changes").or(changes)
    }
}

fn progress_alias_conflict(progress: &serde_json::Value) -> bool {
    [
        ("implementation_completed", "changes_completed"),
        ("implementation_total", "changes_total"),
    ]
    .iter()
    .any(|(canonical, alias)| {
        progress.get(*canonical).is_some()
            && progress.get(*alias).is_some()
            && progress.get(*canonical) != progress.get(*alias)
    })
}

fn projection_mismatch_count(kbd_root: &Path) -> u64 {
    let waypoint = fs::read(kbd_root.join("current-waypoint.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    let position = fs::read(kbd_root.join("position.json"))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok());
    match (waypoint, position) {
        (Some(waypoint), Some(position))
            if waypoint["revision"].as_u64() == position["sourceRevision"].as_u64() =>
        {
            0
        }
        (Some(_), Some(_)) => 1,
        _ => 0,
    }
}

fn collect_named_files(dir: &Path, name: &str, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_named_files(&path, name, out)?;
        } else if entry.file_name() == name {
            out.push(path);
        }
    }
    out.sort();
    Ok(())
}

fn collect_all_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect_all_files(&path, out)?;
        } else if entry.file_type()?.is_file() {
            out.push(path);
        }
    }
    out.sort();
    Ok(())
}

#[allow(dead_code)]
fn normalize_progress(
    progress: &mut serde_json::Value,
    phase: &str,
    completed: u64,
    total: u64,
    updated_at: DateTime<Utc>,
) -> Result<()> {
    let object = progress
        .as_object_mut()
        .ok_or_else(|| RuntimeError::InvalidState("progress must be an object".into()))?;
    let changes = object
        .remove("changes")
        .unwrap_or_else(|| serde_json::json!([]));
    let normalized = match changes {
        serde_json::Value::Array(rows) => rows
            .into_iter()
            .enumerate()
            .map(|(index, row)| match row {
                serde_json::Value::String(id) => serde_json::json!({
                    "id": id,
                    "status": "PENDING",
                    "implementation_status": "PENDING"
                }),
                serde_json::Value::Object(mut row) => {
                    if !row.contains_key("id") {
                        row.insert("id".into(), serde_json::json!(format!("legacy-{index:03}")));
                    }
                    if !row.contains_key("status") {
                        row.insert("status".into(), serde_json::json!("PENDING"));
                    }
                    if !row.contains_key("implementation_status") {
                        let status = row
                            .get("status")
                            .and_then(|value| value.as_str())
                            .unwrap_or("PENDING");
                        row.insert(
                            "implementation_status".into(),
                            serde_json::json!(if status == "DONE" { "COMPLETE" } else { status }),
                        );
                    }
                    serde_json::Value::Object(row)
                }
                _ => serde_json::json!({
                    "id": format!("legacy-{index:03}"),
                    "status": "PENDING",
                    "implementation_status": "PENDING"
                }),
            })
            .collect::<Vec<_>>(),
        serde_json::Value::Object(rows) => rows
            .into_iter()
            .map(|(id, value)| {
                let mut row = value.as_object().cloned().unwrap_or_default();
                row.insert("id".into(), serde_json::json!(id));
                if !row.contains_key("status") {
                    row.insert("status".into(), serde_json::json!("PENDING"));
                }
                if !row.contains_key("implementation_status") {
                    let status = row
                        .get("status")
                        .and_then(|entry| entry.as_str())
                        .unwrap_or("PENDING");
                    row.insert(
                        "implementation_status".into(),
                        serde_json::json!(if status == "DONE" { "COMPLETE" } else { status }),
                    );
                }
                serde_json::Value::Object(row)
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    object.insert("schemaVersion".into(), serde_json::json!("2"));
    object.insert("phase".into(), serde_json::json!(phase));
    object.insert("last_updated".into(), serde_json::json!(updated_at));
    object.insert("last_updated_by".into(), serde_json::json!("kbd-runtime"));
    object.insert("changes".into(), serde_json::Value::Array(normalized));
    object.insert("changes_completed".into(), serde_json::json!(completed));
    object.insert("changes_total".into(), serde_json::json!(total));
    object.insert(
        "implementation_completed".into(),
        serde_json::json!(completed),
    );
    object.insert("implementation_total".into(), serde_json::json!(total));
    object.insert(
        "completion".into(),
        serde_json::json!({
            "primaryCounter": "implementation",
            "implementation": {
                "completed": completed,
                "total": total,
                "status": if completed >= total { "COMPLETE" } else { "IN_PROGRESS" }
            },
            "evidence": {"status":"NOT_TRACKED","summary":null,"blockers":[]},
            "certification": {"status":"NOT_TRACKED","summary":null,"blockers":[]},
            "publication": {"status":"NOT_TRACKED","summary":null,"blockers":[]}
        }),
    );
    Ok(())
}

fn atomic_json(path: &Path, value: &serde_json::Value) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    atomic_bytes(path, &bytes)
}

fn atomic_text(path: &Path, value: &str) -> Result<()> {
    atomic_bytes(path, value.as_bytes())
}

fn atomic_bytes(path: &Path, value: &[u8]) -> Result<()> {
    let parent = path.parent().ok_or(RuntimeError::NotInitialized)?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("projection"),
        Uuid::new_v4()
    ));
    {
        let mut file = File::create(&temp)?;
        file.write_all(value)?;
        file.sync_all()?;
    }
    fs::rename(&temp, path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

fn validate_command_frontier(state: &RuntimeState, envelope: &CommandEnvelope) -> Result<()> {
    if envelope.schema_version == "1" {
        if state.revision != envelope.expected_revision {
            return Err(RuntimeError::RevisionConflict {
                expected: envelope.expected_revision,
                actual: state.revision,
            });
        }
        return Ok(());
    }
    let supplied = envelope.frontier.clone().ok_or_else(|| {
        RuntimeError::InvalidState("command schemaVersion 2 requires frontier".into())
    })?;
    if supplied != state.frontier {
        return Err(RuntimeError::FrontierConflict {
            supplied,
            current: state.frontier.clone(),
        });
    }
    Ok(())
}

/// Prepare an event for a filesystem-free replica. The returned bytes are the
/// exact Ed25519 payload a secure host must sign; private key material never
/// needs to cross the embedding boundary.
pub fn prepare_host_signed_event(
    state: &KbdStateV2,
    replica_id: &str,
    envelope: CommandEnvelope,
    signer_key_id: &str,
    signer_public_key: &str,
) -> Result<(Event, Vec<u8>)> {
    if replica_id.trim().is_empty() {
        return Err(RuntimeError::InvalidState(
            "replicaId must not be empty".into(),
        ));
    }
    if envelope.schema_version != EVENT_SCHEMA_VERSION {
        return Err(RuntimeError::InvalidState(format!(
            "embedded replicas require command schemaVersion {EVENT_SCHEMA_VERSION}"
        )));
    }
    if envelope.command_id.trim().is_empty() {
        return Err(RuntimeError::InvalidState(
            "commandId must not be empty".into(),
        ));
    }
    if envelope.project_id != state.project_id {
        return Err(RuntimeError::ProjectMismatch {
            supplied: envelope.project_id,
            current: state.project_id.clone(),
        });
    }
    if envelope.run_id != state.run_id {
        return Err(RuntimeError::RunMismatch {
            supplied: envelope.run_id,
            current: state.run_id.clone(),
        });
    }
    if state.command_revisions.contains_key(&envelope.command_id) {
        return Err(RuntimeError::DuplicateCommand(envelope.command_id));
    }
    validate_command_frontier(state, &envelope)?;
    validate_claim_write(state, &envelope.actor, &envelope.command)?;
    validate_replica_write_state(state, &envelope.actor, &envelope.command)?;
    let kind = prepare_command_event(state, &envelope.actor, &envelope.command)?;
    let replica_head = state.replica_heads.get(replica_id);
    let mut event = Event {
        schema_version: EVENT_SCHEMA_VERSION.into(),
        project_id: state.project_id.clone(),
        replica_id: replica_id.into(),
        run_id: state.run_id.clone(),
        event_id: Uuid::new_v4().to_string(),
        command_id: Some(envelope.command_id),
        revision: state.frontier.derived_revision().saturating_add(1),
        expected_revision: state.revision,
        lamport: state.frontier.next_lamport(replica_id),
        frontier: state.frontier.clone(),
        causal_parent: replica_head.map(|head| head.event_id.clone()),
        actor_id: envelope.actor.id.clone(),
        actor: envelope.actor,
        timestamp: Utc::now(),
        kind,
        previous_hash: replica_head.map(|head| head.integrity_hash.clone()),
        migration_provenance: None,
        integrity_hash: String::new(),
        signer_key_id: None,
        signer_public_key: None,
        signature: None,
    };
    let bytes = event.prepare_host_signature(signer_key_id, signer_public_key)?;
    Ok((event, bytes))
}

fn validate_replica_write_state(
    state: &RuntimeState,
    actor: &Actor,
    command: &CommandKind,
) -> Result<()> {
    let Some(scope) = command_scope(command) else {
        return Ok(());
    };
    let now = Utc::now();
    if let Some(claim) = state.claims.values().find(|claim| {
        claim.mode == ClaimMode::Exclusive
            && claim.holder_id != actor.id
            && claim.active_at(now)
            && scopes_intersect(&claim.scope, &scope)
    }) {
        return Err(RuntimeError::ReplicaRebaseRequired {
            scope: claim.scope.clone(),
            winner_event_id: claim.acquired_event_id.clone(),
            frontier: state.frontier.clone(),
        });
    }
    if let Some(conflict) = state.conflicts.values().find(|conflict| {
        conflict.resolved_by_event_id.is_none()
            && matches!(
                conflict.kind,
                ConflictKind::Lifecycle | ConflictKind::ActivePath | ConflictKind::Completion
            )
            && scopes_intersect(&conflict.slot, &scope)
    }) {
        return Err(RuntimeError::ReplicaRebaseRequired {
            scope,
            winner_event_id: conflict.winner_event_id.clone(),
            frontier: state.frontier.clone(),
        });
    }
    Ok(())
}

struct TemporaryGitIndex(PathBuf);

impl Drop for TemporaryGitIndex {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
        let mut lock = self.0.as_os_str().to_owned();
        lock.push(".lock");
        let _ = fs::remove_file(PathBuf::from(lock));
    }
}

fn git_plumbing(
    root: &Path,
    arguments: &[&str],
    stdin: Option<&[u8]>,
    index_path: Option<&Path>,
) -> Result<String> {
    let mut command = Command::new("git");
    command.arg("-C").arg(root).args(arguments);
    command
        .env("GIT_AUTHOR_NAME", "Prometheus KBD Audit")
        .env("GIT_AUTHOR_EMAIL", "kbd-audit@localhost")
        .env("GIT_COMMITTER_NAME", "Prometheus KBD Audit")
        .env("GIT_COMMITTER_EMAIL", "kbd-audit@localhost");
    if let Some(index_path) = index_path {
        command.env("GIT_INDEX_FILE", index_path);
    }
    if stdin.is_some() {
        command.stdin(Stdio::piped());
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|error| {
        RuntimeError::InvalidState(format!("cannot run git {}: {error}", arguments.join(" ")))
    })?;
    if let Some(bytes) = stdin {
        child
            .stdin
            .take()
            .ok_or_else(|| RuntimeError::InvalidState("git stdin was unavailable".into()))?
            .write_all(bytes)?;
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        return Err(RuntimeError::InvalidState(format!(
            "git {} failed: {}",
            arguments.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| {
            RuntimeError::InvalidState(format!("git returned non-UTF-8 output: {error}"))
        })
}

fn git_plumbing_optional(
    root: &Path,
    arguments: &[&str],
    index_path: Option<&Path>,
) -> Result<Option<String>> {
    let mut command = Command::new("git");
    command.arg("-C").arg(root).args(arguments);
    if let Some(index_path) = index_path {
        command.env("GIT_INDEX_FILE", index_path);
    }
    let output = command.output().map_err(|error| {
        RuntimeError::InvalidState(format!("cannot run git {}: {error}", arguments.join(" ")))
    })?;
    if !output.status.success() {
        return Ok(None);
    }
    String::from_utf8(output.stdout)
        .map(|value| Some(value.trim().to_owned()))
        .map_err(|error| {
            RuntimeError::InvalidState(format!("git returned non-UTF-8 output: {error}"))
        })
}

fn git_stdout(root: &Path, arguments: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn git_success(root: &Path, arguments: &[&str]) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(arguments)
        .output()
        .is_ok_and(|output| output.status.success())
}

fn validate_claim_write(state: &RuntimeState, actor: &Actor, command: &CommandKind) -> Result<()> {
    let Some(command_scope) = command_scope(command) else {
        return Ok(());
    };
    for conflict in state
        .conflicts
        .values()
        .filter(|conflict| conflict.kind == ConflictKind::Claim)
    {
        let winner = conflict
            .candidates
            .iter()
            .find(|candidate| candidate.event_id == conflict.winner_event_id);
        let winner_holder = winner
            .and_then(|candidate| candidate.value.get("payload"))
            .and_then(|payload| payload.get("holderId"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        for candidate in &conflict.candidates {
            let payload = candidate.value.get("payload");
            let holder = payload
                .and_then(|payload| payload.get("holderId"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or(&candidate.actor_id);
            let scope = payload
                .and_then(|payload| payload.get("scope"))
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            if candidate.event_id != conflict.winner_event_id
                && holder == actor.id
                && scopes_intersect(scope, &command_scope)
            {
                return Err(RuntimeError::ClaimBlocked {
                    scope: scope.to_string(),
                    holder_id: winner_holder.to_string(),
                    frontier: state.frontier.clone(),
                });
            }
        }
    }
    Ok(())
}

fn scopes_intersect(left: &str, right: &str) -> bool {
    left == right
        || right
            .strip_prefix(left)
            .is_some_and(|suffix| suffix.starts_with('/'))
        || left
            .strip_prefix(right)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn command_scope(command: &CommandKind) -> Option<String> {
    Some(match command {
        CommandKind::ClaimAcquire { .. }
        | CommandKind::ClaimRenew { .. }
        | CommandKind::ClaimRelease { .. }
        | CommandKind::ConflictResolve { .. }
        | CommandKind::DeviceEnroll { .. }
        | CommandKind::DeviceRevoke { .. }
        | CommandKind::DeviceRotate { .. } => return None,
        CommandKind::Pause { .. }
        | CommandKind::Cancel { .. }
        | CommandKind::LifecycleTransition { .. }
        | CommandKind::Resume { .. }
        | CommandKind::PlanRevise { .. } => "singleton:lifecycle".into(),
        CommandKind::ActivePathSet { .. } => "singleton:active_path".into(),
        CommandKind::PhaseDefine { phase } => format!("phase:{}", phase.id),
        CommandKind::PhaseTransition { phase_id, .. } => format!("phase:{phase_id}"),
        CommandKind::StageEnter { phase_id, stage } => {
            format!("phase:{phase_id}/stage:{}", stage.id)
        }
        CommandKind::StageTransition {
            phase_id, stage_id, ..
        } => format!("phase:{phase_id}/stage:{stage_id}"),
        CommandKind::ChangeRegister { phase_id, change } => {
            format!("phase:{phase_id}/change:{}", change.id)
        }
        CommandKind::ChangeTransition {
            phase_id,
            change_id,
            ..
        } => format!("phase:{phase_id}/change:{change_id}"),
        CommandKind::TaskRegister {
            phase_id,
            change_id,
            task,
        } => format!("phase:{phase_id}/change:{change_id}/task:{}", task.id),
        CommandKind::TaskTransition {
            phase_id,
            change_id,
            task_id,
            ..
        } => format!("phase:{phase_id}/change:{change_id}/task:{task_id}"),
        CommandKind::CompletionSet { dimension, .. } => format!("completion:{dimension:?}"),
        CommandKind::DecisionRecord { decision } => format!("decision:{}", decision.id),
        CommandKind::BlockerRecord { blocker } => format!("blocker:{}", blocker.id),
        CommandKind::BlockerClear { blocker_id, .. } => format!("blocker:{blocker_id}"),
        CommandKind::SubmodulePinSet { pin } => format!("submodule:{}", pin.path),
    })
}

fn prepare_command_event(
    state: &RuntimeState,
    actor: &Actor,
    command: &CommandKind,
) -> Result<EventKind> {
    let require_reason = |reason: &str| {
        if reason.trim().is_empty() {
            Err(RuntimeError::ReasonRequired)
        } else {
            Ok(())
        }
    };
    Ok(match command {
        CommandKind::Pause { checkpoint } => {
            require_reason(&checkpoint.reason)?;
            let mut checkpoint = checkpoint.clone();
            checkpoint.previous_state = state.lifecycle.clone();
            checkpoint.plan_revision = state.plan_revision;
            EventKind::PauseCheckpointed { checkpoint }
        }
        CommandKind::Cancel { reason } => {
            require_reason(reason)?;
            EventKind::LifecycleTransition {
                from: state.lifecycle.clone(),
                to: LifecycleState::Cancelled,
                reason: reason.clone(),
            }
        }
        CommandKind::LifecycleTransition { to, reason } => {
            require_reason(reason)?;
            EventKind::LifecycleTransition {
                from: state.lifecycle.clone(),
                to: to.clone(),
                reason: reason.clone(),
            }
        }
        CommandKind::Resume { plan_revision } => {
            if *plan_revision != state.plan_revision {
                return Err(RuntimeError::PlanRevision {
                    supplied: *plan_revision,
                    current: state.plan_revision,
                });
            }
            EventKind::LifecycleTransition {
                from: state.lifecycle.clone(),
                to: LifecycleState::Running,
                reason: format!("resume plan revision {plan_revision}"),
            }
        }
        CommandKind::PlanRevise {
            reason,
            exact_next_work,
        } => {
            require_reason(reason)?;
            EventKind::PlanRevised {
                from_revision: state.plan_revision,
                to_revision: state.plan_revision + 1,
                reason: reason.clone(),
                superseded_next_work: state.exact_next_work.clone(),
                exact_next_work: exact_next_work.clone(),
            }
        }
        CommandKind::PhaseDefine { phase } => EventKind::PhaseDefined {
            phase: phase.clone(),
        },
        CommandKind::PhaseTransition { phase_id, to } => {
            let phase =
                state
                    .phases
                    .get(phase_id)
                    .ok_or_else(|| RuntimeError::WorkItemNotFound {
                        kind: "phase",
                        id: phase_id.clone(),
                    })?;
            EventKind::PhaseTransitioned {
                phase_id: phase_id.clone(),
                from: phase.status.clone(),
                to: to.clone(),
            }
        }
        CommandKind::StageEnter { phase_id, stage } => EventKind::StageEntered {
            phase_id: phase_id.clone(),
            stage: stage.clone(),
        },
        CommandKind::StageTransition {
            phase_id,
            stage_id,
            to,
        } => {
            let stage = state
                .phases
                .get(phase_id)
                .and_then(|phase| phase.stages.get(stage_id))
                .ok_or_else(|| RuntimeError::WorkItemNotFound {
                    kind: "stage",
                    id: stage_id.clone(),
                })?;
            EventKind::StageTransitioned {
                phase_id: phase_id.clone(),
                stage_id: stage_id.clone(),
                from: stage.status.clone(),
                to: to.clone(),
            }
        }
        CommandKind::ChangeRegister { phase_id, change } => EventKind::ChangeRegistered {
            phase_id: phase_id.clone(),
            change: change.clone(),
        },
        CommandKind::ChangeTransition {
            phase_id,
            change_id,
            to,
        } => {
            let change = state
                .phases
                .get(phase_id)
                .and_then(|phase| phase.changes.get(change_id))
                .ok_or_else(|| RuntimeError::WorkItemNotFound {
                    kind: "change",
                    id: change_id.clone(),
                })?;
            EventKind::ChangeTransitioned {
                phase_id: phase_id.clone(),
                change_id: change_id.clone(),
                from: change.status.clone(),
                to: to.clone(),
            }
        }
        CommandKind::TaskRegister {
            phase_id,
            change_id,
            task,
        } => EventKind::TaskRegistered {
            phase_id: phase_id.clone(),
            change_id: change_id.clone(),
            task: task.clone(),
        },
        CommandKind::TaskTransition {
            phase_id,
            change_id,
            task_id,
            to,
            summary,
        } => {
            let from = state
                .phases
                .get(phase_id)
                .and_then(|phase| phase.changes.get(change_id))
                .and_then(|change| change.tasks.get(task_id))
                .map(|task| task.status.clone())
                .ok_or_else(|| RuntimeError::WorkItemNotFound {
                    kind: "task",
                    id: task_id.clone(),
                })?;
            EventKind::TaskTransitioned {
                phase_id: phase_id.clone(),
                change_id: change_id.clone(),
                task_id: task_id.clone(),
                from,
                to: to.clone(),
                summary: summary.clone(),
            }
        }
        CommandKind::ActivePathSet {
            active_path,
            exact_next_work,
        } => EventKind::ActivePathChanged {
            active_path: active_path.clone(),
            exact_next_work: exact_next_work.clone(),
        },
        CommandKind::CompletionSet {
            dimension,
            completion,
        } => EventKind::CompletionUpdated {
            dimension: *dimension,
            completion: completion.clone(),
        },
        CommandKind::DecisionRecord { decision } => {
            let mut decision = decision.clone();
            decision.plan_revision = state.plan_revision;
            EventKind::DecisionRecorded { decision }
        }
        CommandKind::BlockerRecord { blocker } => EventKind::BlockerRecorded {
            blocker: blocker.clone(),
        },
        CommandKind::BlockerClear {
            blocker_id,
            resolution,
        } => EventKind::BlockerCleared {
            blocker_id: blocker_id.clone(),
            resolution: resolution.clone(),
        },
        CommandKind::DeviceEnroll { device } => {
            if actor.kind != ActorKind::Operator {
                return Err(RuntimeError::InvalidState(
                    "only an operator may enroll a device".into(),
                ));
            }
            EventKind::DeviceEnrolled {
                device: device.clone(),
            }
        }
        CommandKind::DeviceRevoke { key_id, reason } => {
            if actor.kind != ActorKind::Operator {
                return Err(RuntimeError::InvalidState(
                    "only an operator may revoke a device".into(),
                ));
            }
            require_reason(reason)?;
            EventKind::DeviceRevoked {
                key_id: key_id.clone(),
                reason: reason.clone(),
            }
        }
        CommandKind::DeviceRotate {
            previous_key_id,
            replacement,
        } => {
            if actor.kind != ActorKind::Operator {
                return Err(RuntimeError::InvalidState(
                    "only an operator may rotate a device key".into(),
                ));
            }
            EventKind::DeviceKeyRotated {
                previous_key_id: previous_key_id.clone(),
                replacement: replacement.clone(),
            }
        }
        CommandKind::ConflictResolve {
            conflict_id,
            winner_event_id,
            reason,
        } => {
            if actor.kind != ActorKind::Operator {
                return Err(RuntimeError::InvalidState(
                    "only an operator may resolve a conflict".into(),
                ));
            }
            require_reason(reason)?;
            let conflict =
                state
                    .conflicts
                    .get(conflict_id)
                    .ok_or_else(|| RuntimeError::WorkItemNotFound {
                        kind: "conflict",
                        id: conflict_id.clone(),
                    })?;
            if !conflict
                .candidates
                .iter()
                .any(|candidate| candidate.event_id == *winner_event_id)
            {
                return Err(RuntimeError::InvalidState(format!(
                    "event {winner_event_id} is not a candidate for conflict {conflict_id}"
                )));
            }
            EventKind::ConflictResolved {
                conflict_id: conflict_id.clone(),
                winner_event_id: winner_event_id.clone(),
                reason: reason.clone(),
            }
        }
        CommandKind::ClaimAcquire {
            scope,
            mode,
            ttl_seconds,
            holder_id,
        } => {
            validate_claim_ttl(*ttl_seconds)?;
            if scope.trim().is_empty() || holder_id.trim().is_empty() {
                return Err(RuntimeError::InvalidState(
                    "claim scope and holderId must not be empty".into(),
                ));
            }
            if holder_id != &actor.id {
                return Err(RuntimeError::InvalidState(
                    "claim holderId must match actor.id".into(),
                ));
            }
            let now = Utc::now();
            if let Some(existing) = state.claims.values().find(|claim| {
                claim.scope == *scope
                    && claim.holder_id != *holder_id
                    && claim.active_at(now)
                    && (*mode == ClaimMode::Exclusive || claim.mode == ClaimMode::Exclusive)
            }) {
                return Err(RuntimeError::ClaimBlocked {
                    scope: scope.clone(),
                    holder_id: existing.holder_id.clone(),
                    frontier: state.frontier.clone(),
                });
            }
            let monotonic_token = state
                .claims
                .values()
                .filter(|claim| claim.scope == *scope)
                .map(|claim| claim.monotonic_token)
                .max()
                .unwrap_or(0)
                .saturating_add(1);
            EventKind::ClaimAcquired {
                claim_id: Uuid::new_v4().to_string(),
                scope: scope.clone(),
                holder_id: holder_id.clone(),
                mode: *mode,
                expires_at: now + chrono::Duration::seconds(*ttl_seconds as i64),
                monotonic_token,
            }
        }
        CommandKind::ClaimRenew {
            claim_id,
            ttl_seconds,
        } => {
            validate_claim_ttl(*ttl_seconds)?;
            let claim =
                state
                    .claims
                    .get(claim_id)
                    .ok_or_else(|| RuntimeError::WorkItemNotFound {
                        kind: "claim",
                        id: claim_id.clone(),
                    })?;
            if claim.released {
                return Err(RuntimeError::InvalidState(format!(
                    "claim {claim_id} is already released"
                )));
            }
            if claim.holder_id != actor.id && actor.kind != ActorKind::Operator {
                return Err(RuntimeError::InvalidState(
                    "only the claim holder or an operator may renew a claim".into(),
                ));
            }
            EventKind::ClaimRenewed {
                claim_id: claim_id.clone(),
                expires_at: Utc::now() + chrono::Duration::seconds(*ttl_seconds as i64),
                monotonic_token: claim.monotonic_token.saturating_add(1),
            }
        }
        CommandKind::ClaimRelease { claim_id } => {
            let claim =
                state
                    .claims
                    .get(claim_id)
                    .ok_or_else(|| RuntimeError::WorkItemNotFound {
                        kind: "claim",
                        id: claim_id.clone(),
                    })?;
            if claim.holder_id != actor.id && actor.kind != ActorKind::Operator {
                return Err(RuntimeError::InvalidState(
                    "only the claim holder or an operator may release a claim".into(),
                ));
            }
            EventKind::ClaimReleased {
                claim_id: claim_id.clone(),
                monotonic_token: claim.monotonic_token.saturating_add(1),
            }
        }
        CommandKind::SubmodulePinSet { pin } => {
            if actor.kind != ActorKind::Operator {
                return Err(RuntimeError::InvalidState(
                    "submodule pins are parent-owned and require an operator actor".into(),
                ));
            }
            EventKind::SubmodulePinRecorded { pin: pin.clone() }
        }
    })
}

fn validate_claim_ttl(ttl_seconds: u64) -> Result<()> {
    if !(1..=86_400).contains(&ttl_seconds) {
        return Err(RuntimeError::InvalidState(
            "claim TTL must be between 1 and 86400 seconds".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn actor(kind: ActorKind, harness: &str) -> Actor {
        Actor {
            kind,
            id: harness.into(),
            device: format!("device-{harness}"),
            harness: harness.into(),
            session: format!("session-{harness}"),
        }
    }

    fn authority_state(mut state: RuntimeState) -> RuntimeState {
        state.replica_view = None;
        state
    }

    #[test]
    fn replay_is_deterministic_and_integrity_checked() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let state = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let running = runtime
            .transition(
                actor(ActorKind::Harness, "codex"),
                state.revision,
                LifecycleState::Running,
                "begin",
            )
            .unwrap();
        assert_eq!(running, runtime.replay().unwrap());
    }

    #[test]
    fn workflow_events_reconstruct_exact_position_and_commands_are_idempotent() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let initialized = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let context = |state: &RuntimeState, command_id: &str| MutationContext {
            expected_revision: state.revision,
            command_id: command_id.into(),
        };
        let phase = runtime
            .define_phase(
                actor(ActorKind::Harness, "codex"),
                context(&initialized, "define-phase"),
                Phase {
                    id: "phase-1".into(),
                    slug: "phase-1".into(),
                    title: "Production convergence".into(),
                    parent_phase_id: None,
                    status: WorkStatus::InProgress,
                    stages: BTreeMap::new(),
                    changes: BTreeMap::new(),
                    legacy_read_only: false,
                },
            )
            .unwrap();
        let change = runtime
            .register_change(
                actor(ActorKind::Harness, "codex"),
                context(&phase, "register-change"),
                "phase-1",
                Change {
                    id: "change-1".into(),
                    title: "Canonical authority".into(),
                    sequence: 0,
                    status: WorkStatus::Pending,
                    implementation_status: WorkStatus::Pending,
                    tasks: BTreeMap::new(),
                },
            )
            .unwrap();
        let registered = runtime
            .register_task(
                actor(ActorKind::Harness, "codex"),
                context(&change, "register-task"),
                "phase-1",
                "change-1",
                Task {
                    id: "task-1".into(),
                    title: "Persist task state".into(),
                    sequence: 0,
                    status: WorkStatus::Pending,
                    summary: None,
                },
            )
            .unwrap();
        let started = runtime
            .transition_task(
                actor(ActorKind::Harness, "codex"),
                context(&registered, "start-task"),
                "phase-1",
                "change-1",
                "task-1",
                WorkStatus::InProgress,
                None,
            )
            .unwrap();
        let completed = runtime
            .transition_task(
                actor(ActorKind::Harness, "codex"),
                context(&started, "complete-task"),
                "phase-1",
                "change-1",
                "task-1",
                WorkStatus::Complete,
                Some("stored in the canonical journal".into()),
            )
            .unwrap();
        let positioned = runtime
            .set_active_path(
                actor(ActorKind::Harness, "codex"),
                context(&completed, "position"),
                ActivePath {
                    phase_path: vec!["phase-1".into()],
                    phase_id: Some("phase-1".into()),
                    stage_id: None,
                    change_id: Some("change-1".into()),
                    task_id: Some("task-1".into()),
                    commit: None,
                },
                Some("start durable storage".into()),
            )
            .unwrap();
        let before_retry = runtime.events().unwrap().len();
        let original = runtime
            .append_command(
                actor(ActorKind::Harness, "codex"),
                completed.revision,
                "position",
                EventKind::ActivePathChanged {
                    active_path: ActivePath::default(),
                    exact_next_work: None,
                },
            )
            .unwrap();
        assert_eq!(original, positioned);
        assert_eq!(runtime.events().unwrap().len(), before_retry);
        assert_eq!(
            positioned
                .phases
                .get("phase-1")
                .unwrap()
                .changes
                .get("change-1")
                .unwrap()
                .implementation_status,
            WorkStatus::Complete
        );
        assert_eq!(
            positioned
                .completion
                .get(&CompletionDimension::Implementation)
                .unwrap()
                .completed,
            1
        );
    }

    #[test]
    fn schema_v2_events_are_canonical_signed_and_reject_unknown_signers() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let initialized = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let running = runtime
            .transition(
                actor(ActorKind::Harness, "codex"),
                initialized.revision,
                LifecycleState::Running,
                "begin",
            )
            .unwrap();
        let events = runtime.events().unwrap();
        assert_eq!(events[0].schema_version, "2");
        assert_eq!(events[0].replica_id, runtime.replica_id());
        assert_eq!(events[0].lamport, 1);
        assert!(events[0].frontier.is_empty());
        assert_eq!(events[0].actor_id, events[0].actor.id);
        assert_eq!(events[1].lamport, 2);
        assert_eq!(events[1].frontier.lamport(runtime.replica_id()), 1);
        assert_eq!(running.frontier.lamport(runtime.replica_id()), 2);
        assert_eq!(running.revision, running.frontier.derived_revision());
        assert!(events[0].signature.is_some());
        assert_eq!(initialized.devices.len(), 1);

        let mut forged = events[1].clone();
        forged.event_id = Uuid::new_v4().to_string();
        forged.command_id = Some(Uuid::new_v4().to_string());
        let unknown = DeviceSigner::generate();
        forged.seal(&unknown).unwrap();
        // Any not-yet-enrolled signer is now trusted (auto-enrolled) rather
        // than rejected — enrollment is no longer genesis-only. Revocation
        // (tested elsewhere) remains the real, enforced trust boundary.
        let forged_state = replay_events(&[events[0].clone(), forged])
            .expect("a new signer on a fresh event is auto-enrolled, not rejected");
        assert_eq!(forged_state.devices.len(), 2);

        let mut tampered = events;
        tampered[1].actor.harness = "claude".into();
        assert!(matches!(
            replay_events(&tampered),
            Err(RuntimeError::Signature { .. }) | Err(RuntimeError::Integrity { .. })
        ));
        assert_eq!(running.devices.len(), 1);

        let mut audit = Vec::new();
        assert_eq!(runtime.export_signed_audit(&mut audit).unwrap(), 2);
        let lines = std::str::from_utf8(&audit)
            .unwrap()
            .lines()
            .collect::<Vec<_>>();
        assert_eq!(lines.len(), 2);
        let exported = lines
            .iter()
            .map(|line| serde_json::from_str::<Event>(line).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(replay_events(&exported).unwrap(), authority_state(running));

        #[cfg(unix)]
        assert_eq!(
            fs::metadata(runtime.device_key_path())
                .unwrap()
                .permissions()
                .mode()
                & 0o077,
            0
        );
    }

    #[test]
    fn git_audit_export_preserves_worktree_and_detects_chain_tampering() {
        let project = tempdir().unwrap();
        assert!(Command::new("git")
            .arg("-C")
            .arg(project.path())
            .args(["init", "--quiet"])
            .status()
            .unwrap()
            .success());
        fs::write(
            project.path().join("tracked.txt"),
            b"worktree remains untouched\n",
        )
        .unwrap();
        assert!(Command::new("git")
            .arg("-C")
            .arg(project.path())
            .args(["add", "tracked.txt"])
            .status()
            .unwrap()
            .success());

        let runtime = Runtime::open(project.path());
        let initialized = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        runtime
            .transition(
                actor(ActorKind::Harness, "codex"),
                initialized.revision,
                LifecycleState::Running,
                "begin",
            )
            .unwrap();
        let status_before = git_stdout(project.path(), &["status", "--porcelain=v1"]).unwrap();
        let index_before = git_plumbing(project.path(), &["write-tree"], None, None).unwrap();

        let exported = runtime.export_audit_to_git().unwrap();
        assert_eq!(exported.ref_name, AUDIT_GIT_REF);
        assert!(!exported.unchanged);
        let repeated = runtime.export_audit_to_git().unwrap();
        assert!(repeated.unchanged);
        assert_eq!(repeated.commit_id, exported.commit_id);
        assert_eq!(
            git_stdout(project.path(), &["status", "--porcelain=v1"]).unwrap(),
            status_before
        );
        assert_eq!(
            git_plumbing(project.path(), &["write-tree"], None, None).unwrap(),
            index_before
        );
        let committed = git_plumbing(
            project.path(),
            &["show", &format!("{}:{}", AUDIT_GIT_REF, exported.tree_path)],
            None,
            None,
        )
        .unwrap();
        let (audit, count) = runtime.signed_audit_jsonl().unwrap();
        assert_eq!(count, 2);
        assert_eq!(committed.as_bytes(), audit.strip_suffix(b"\n").unwrap());

        let events = runtime.events().unwrap();
        assert!(replay_events(&events[1..]).is_err());
        let mut mutated = events.clone();
        mutated[1].command_id = Some("mutated-command".into());
        assert!(replay_events(&mutated).is_err());
        let mut reordered = events;
        reordered.reverse();
        assert!(replay_events(&reordered).is_err());
    }

    #[test]
    fn remote_command_signature_covers_the_full_schema_v2_envelope() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let state = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "operator"))
            .unwrap();
        let signer = runtime.device_signer().unwrap();
        let command = CommandEnvelope {
            schema_version: "2".into(),
            project_id: state.project_id.clone(),
            run_id: state.run_id.clone(),
            command_id: "signed-command".into(),
            frontier: Some(state.frontier.clone()),
            expected_revision: 0,
            actor: actor(ActorKind::Harness, "remote"),
            command: CommandKind::PlanRevise {
                reason: "signed remote mutation".into(),
                exact_next_work: Some("continue".into()),
            },
        };
        let signed = SignedCommandEnvelope::sign(command, &signer).unwrap();
        signed.verify(&state).unwrap();

        let mut tampered = signed;
        if let CommandKind::PlanRevise { reason, .. } = &mut tampered.command.command {
            *reason = "tampered".into();
        }
        assert!(matches!(
            tampered.verify(&state),
            Err(RuntimeError::Signature { .. })
        ));
    }

    #[test]
    fn missing_active_path_commit_renders_ahead_of_me_without_a_conflict() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let mut state = RuntimeState::default();
        state.active_path.commit = Some("0".repeat(40));
        runtime.decorate_replica_view(&mut state);
        assert_eq!(
            state.replica_view.unwrap().active_path_status,
            ReplicaCommitStatus::AheadOfMe
        );
        assert!(state.conflicts.is_empty());
    }

    #[test]
    fn explicit_headless_device_key_is_validated_before_use() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("device-key.json");
        let original = DeviceSigner::generate();
        let mut options = OpenOptions::new();
        options.create_new(true).write(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = options.open(&path).unwrap();
        serde_json::to_writer(&mut file, &stored_device_key(&original)).unwrap();
        file.sync_all().unwrap();

        let loaded = load_device_key(&path).unwrap();
        assert_eq!(loaded.key_id(), original.key_id());

        let mut altered = stored_device_key(&original);
        altered.key_id = DeviceSigner::generate().key_id().to_string();
        serde_json::to_writer(File::create(&path).unwrap(), &altered).unwrap();
        assert!(matches!(
            load_device_key(&path),
            Err(RuntimeError::InvalidState(_))
        ));
    }

    #[test]
    fn headless_device_key_initialization_is_idempotent_and_private() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("device-key.json");
        let created = ensure_device_key_file(&path).unwrap();
        let reopened = ensure_device_key_file(&path).unwrap();
        assert_eq!(created.key_id(), reopened.key_id());
        #[cfg(unix)]
        assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o077, 0);
    }

    #[cfg(unix)]
    #[test]
    fn explicit_headless_device_key_rejects_broad_permissions_and_symlinks() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("device-key.json");
        let signer = DeviceSigner::generate();
        let mut options = OpenOptions::new();
        options.create_new(true).write(true).mode(0o644);
        serde_json::to_writer(options.open(&path).unwrap(), &stored_device_key(&signer)).unwrap();
        assert!(matches!(
            load_device_key(&path),
            Err(RuntimeError::InvalidState(_))
        ));

        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let link = dir.path().join("linked-key.json");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(matches!(
            load_device_key(&link),
            Err(RuntimeError::InvalidState(_))
        ));
    }

    #[test]
    fn command_envelopes_dedupe_by_command_id_without_ownership_fields() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let initialized = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let phase_command = CommandEnvelope {
            schema_version: "2".into(),
            project_id: "project".into(),
            run_id: "run".into(),
            command_id: "phase-1".into(),
            frontier: Some(initialized.frontier.clone()),
            expected_revision: 0,
            actor: actor(ActorKind::Harness, "codex"),
            command: CommandKind::PhaseDefine {
                phase: Phase {
                    id: "phase-1".into(),
                    slug: "phase-1".into(),
                    title: "Phase".into(),
                    parent_phase_id: None,
                    status: WorkStatus::Pending,
                    stages: BTreeMap::new(),
                    changes: BTreeMap::new(),
                    legacy_read_only: false,
                },
            },
        };
        let applied = runtime.execute_command(phase_command.clone()).unwrap();
        assert!(!applied.duplicate);
        assert_eq!(applied.committed_revision, initialized.revision + 1);
        // Resubmitting the exact same command_id is still idempotent.
        let duplicate = runtime.execute_command(phase_command).unwrap();
        assert!(duplicate.duplicate);
        assert_eq!(duplicate.state, applied.state);
    }

    #[test]
    fn concurrent_commands_cannot_both_commit_from_the_same_revision() {
        use std::sync::{Arc, Barrier};

        let dir = tempdir().unwrap();
        let runtime = Arc::new(Runtime::open(dir.path()));
        let initialized = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let barrier = Arc::new(Barrier::new(3));

        let handles = ["phase-a", "phase-b"].map(|phase_id| {
            let runtime = Arc::clone(&runtime);
            let barrier = Arc::clone(&barrier);
            let phase_id = phase_id.to_string();
            let frontier = initialized.frontier.clone();
            std::thread::spawn(move || {
                let command = CommandEnvelope {
                    schema_version: "2".into(),
                    project_id: "project".into(),
                    run_id: "run".into(),
                    command_id: format!("define-{phase_id}"),
                    frontier: Some(frontier),
                    expected_revision: 0,
                    actor: actor(ActorKind::Harness, &phase_id),
                    command: CommandKind::PhaseDefine {
                        phase: Phase {
                            id: phase_id.clone(),
                            slug: phase_id.clone(),
                            title: phase_id,
                            parent_phase_id: None,
                            status: WorkStatus::Pending,
                            stages: BTreeMap::new(),
                            changes: BTreeMap::new(),
                            legacy_read_only: false,
                        },
                    },
                };
                barrier.wait();
                runtime.execute_command(command)
            })
        });

        barrier.wait();
        let results = handles.map(|handle| handle.join().unwrap());
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| matches!(result, Err(RuntimeError::FrontierConflict { .. })))
                .count(),
            1
        );
        assert_eq!(runtime.events().unwrap().len(), 2);
        assert_eq!(runtime.replay().unwrap().revision, initialized.revision + 1);
    }

    #[test]
    fn losing_claim_holder_is_blocked_with_winner_frontier_and_rebase_instruction() {
        let mut state = RuntimeState::default();
        state.frontier.advance("winner-replica", 4);
        state.conflicts.insert(
            "claim-conflict".into(),
            ConflictRecord {
                id: "claim-conflict".into(),
                slot: "claim:phase:recovery".into(),
                kind: ConflictKind::Claim,
                candidates: vec![
                    ConflictCandidate {
                        event_id: "loser-event".into(),
                        replica_id: "loser-replica".into(),
                        lamport: 2,
                        actor_id: "holder-a".into(),
                        value: serde_json::json!({
                            "type":"claim_acquired",
                            "payload":{"scope":"phase:recovery","holderId":"holder-a"}
                        }),
                    },
                    ConflictCandidate {
                        event_id: "winner-event".into(),
                        replica_id: "winner-replica".into(),
                        lamport: 4,
                        actor_id: "holder-b".into(),
                        value: serde_json::json!({
                            "type":"claim_acquired",
                            "payload":{"scope":"phase:recovery","holderId":"holder-b"}
                        }),
                    },
                ],
                winner_event_id: "winner-event".into(),
                resolved_by_event_id: None,
                resolution_reason: None,
            },
        );
        let losing_actor = Actor {
            kind: ActorKind::Harness,
            id: "holder-a".into(),
            device: "device-a".into(),
            harness: "test".into(),
            session: "session-a".into(),
        };
        let error = validate_claim_write(
            &state,
            &losing_actor,
            &CommandKind::PhaseTransition {
                phase_id: "recovery".into(),
                to: WorkStatus::InProgress,
            },
        )
        .unwrap_err();
        assert!(matches!(
            &error,
            RuntimeError::ClaimBlocked {
                holder_id,
                frontier,
                ..
            } if holder_id == "holder-b" && frontier == &state.frontier
        ));
        assert!(error.to_string().contains("rebase"));
    }

    #[test]
    fn torn_journal_tail_is_archived_with_checksum_before_recovery() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let initialized = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let torn = br#"{"schemaVersion":"2","eventId":"interrupted"#;
        let mut journal = OpenOptions::new()
            .append(true)
            .open(runtime.events_path())
            .unwrap();
        journal.write_all(torn).unwrap();
        journal.sync_data().unwrap();
        assert!(runtime.events().is_err());

        let archive = runtime
            .recover_journal_tail()
            .unwrap()
            .expect("invalid tail must be preserved");
        assert_eq!(fs::read(&archive).unwrap(), torn);
        let checksum = fs::read_to_string(archive.with_extension("archive.sha256")).unwrap();
        assert!(checksum.contains(&format!("{:x}", Sha256::digest(torn))));
        assert_eq!(runtime.events().unwrap().len(), 1);

        let result = runtime
            .execute_command(CommandEnvelope {
                schema_version: "2".into(),
                project_id: "project".into(),
                run_id: "run".into(),
                command_id: "after-recovery".into(),
                frontier: Some(initialized.frontier.clone()),
                expected_revision: 0,
                actor: actor(ActorKind::Harness, "codex"),
                command: CommandKind::LifecycleTransition {
                    to: LifecycleState::Running,
                    reason: "continue after recovery".into(),
                },
            })
            .unwrap();
        assert_eq!(result.committed_revision, initialized.revision + 1);
        assert_eq!(runtime.events().unwrap().len(), 2);
    }

    #[test]
    fn operator_can_cancel_without_an_ownership_side_channel() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let initialized = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let cancelled = runtime
            .append(
                actor(ActorKind::Operator, "claude"),
                initialized.revision,
                EventKind::LifecycleTransition {
                    from: LifecycleState::Ready,
                    to: LifecycleState::Cancelled,
                    reason: "architectural issue".into(),
                },
            )
            .unwrap();
        assert_eq!(cancelled.lifecycle, LifecycleState::Cancelled);
    }

    #[test]
    fn checkpoint_and_plan_revision_preserve_causality() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let initialized = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let running = runtime
            .transition(
                actor(ActorKind::Harness, "codex"),
                initialized.revision,
                LifecycleState::Running,
                "begin",
            )
            .unwrap();
        let revised = runtime
            .revise_plan(
                actor(ActorKind::Harness, "codex"),
                running.revision,
                "architectural correction",
                Some("implement boundary first".into()),
            )
            .unwrap();
        assert_eq!(revised.plan_revision, 2);
        let paused = runtime
            .pause(
                actor(ActorKind::Operator, "claude"),
                revised.revision,
                Checkpoint {
                    reason: "review boundary".into(),
                    previous_state: LifecycleState::Ready,
                    last_completed: Some("audit".into()),
                    exact_next_work: Some("implement boundary first".into()),
                    decisions: vec!["single writer".into()],
                    blockers: vec![],
                    dirty_work_summary: None,
                    plan_revision: 0,
                },
            )
            .unwrap();
        assert_eq!(paused.lifecycle, LifecycleState::Paused);
        assert_eq!(paused.checkpoint.unwrap().plan_revision, 2);
        assert_eq!(paused.revision, runtime.events().unwrap().len() as u64);
    }

    #[test]
    fn signed_frontier_cache_and_hash_linked_compaction_preserve_replay() {
        let project = tempdir().unwrap();
        let runtime = Runtime::open(project.path());
        let initialized = runtime
            .initialize(
                "project-cache",
                "run-cache",
                Actor::operator("operator-cache", "test"),
            )
            .unwrap();
        let running = runtime
            .transition(
                Actor::operator("operator-cache", "test"),
                initialized.revision,
                LifecycleState::Running,
                "exercise checkpoint cache",
            )
            .unwrap();
        let checkpoints = fs::read_dir(runtime.checkpoint_dir())
            .unwrap()
            .filter_map(std::result::Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("checkpoint-")
            })
            .count();
        assert!(checkpoints >= 2);
        assert_eq!(
            runtime.replay_authority().unwrap(),
            authority_state(running.clone())
        );

        let compacted = runtime.compact_journal(1).unwrap().unwrap();
        assert_eq!(compacted.archived_events, 1);
        assert_eq!(compacted.retained_events, 1);
        assert!(compacted.segment.is_file());
        assert!(compacted.manifest.is_file());
        assert!(compacted.rollback_metadata.is_file());
        assert_eq!(
            compacted.payload_sha256,
            format!(
                "sha256:{:x}",
                Sha256::digest(fs::read(&compacted.segment).unwrap())
            )
        );
        assert_eq!(
            runtime.replay_authority().unwrap(),
            authority_state(running)
        );
    }

    #[test]
    fn tampered_folded_checkpoint_is_rejected() {
        let project = tempdir().unwrap();
        let runtime = Runtime::open(project.path());
        runtime
            .initialize(
                "project-tamper",
                "run-tamper",
                Actor::operator("operator-tamper", "test"),
            )
            .unwrap();
        let pointer: CheckpointPointer = serde_json::from_reader(
            File::open(runtime.checkpoint_dir().join("current.json")).unwrap(),
        )
        .unwrap();
        let path = runtime.checkpoint_dir().join(pointer.checkpoint);
        let mut checkpoint: serde_json::Value =
            serde_json::from_reader(File::open(&path).unwrap()).unwrap();
        checkpoint["signature"] = serde_json::Value::String("tampered".into());
        atomic_json(&path, &checkpoint).unwrap();

        assert!(matches!(
            runtime.replay_authority(),
            Err(RuntimeError::InvalidState(message)) if message.contains("checkpoint")
        ));
    }

    #[test]
    fn annotated_legacy_statuses_are_parsed_without_prefix_matches() {
        assert_eq!(
            parse_work_status(Some("DONE (merged #108)")),
            WorkStatus::Complete
        );
        assert_eq!(
            parse_work_status(Some("IN_PROGRESS [owner: codex]")),
            WorkStatus::InProgress
        );
        assert_eq!(parse_work_status(Some("DONEISH")), WorkStatus::Pending);
        assert_eq!(
            parse_work_status(Some("BLOCKED_BY_DESIGN")),
            WorkStatus::Pending
        );
    }

    #[test]
    fn migration_preserves_nested_phase_identity_and_scoped_duplicate_slugs() {
        let dir = tempdir().unwrap();
        let kbd = dir.path().join(".kbd-orchestrator");
        for parent in ["phase-a", "phase-b"] {
            let parent_dir = kbd.join("phases").join(parent);
            let child_dir = parent_dir.join("children/spike");
            fs::create_dir_all(&child_dir).unwrap();
            fs::write(
                parent_dir.join("progress.json"),
                format!(
                    r#"{{"phase":"{parent}","changes":{{"parent-change":{{"status":"DONE (merged)"}}}}}}"#
                ),
            )
            .unwrap();
            fs::write(
                child_dir.join("progress.json"),
                r#"{"phase":"spike","changes":{"child-change":{"status":"IN_PROGRESS"}}}"#,
            )
            .unwrap();
        }
        fs::write(
            kbd.join("current-waypoint.json"),
            r#"{"phase":"spike","path":["phase-b","spike"]}"#,
        )
        .unwrap();

        let runtime = Runtime::open(dir.path());
        runtime.migrate_legacy_ledgers(true).unwrap();
        let state = runtime.replay().unwrap();

        assert_eq!(
            state.phases["phase-a::spike"].parent_phase_id.as_deref(),
            Some("phase-a")
        );
        assert_eq!(
            state.phases["phase-b::spike"].parent_phase_id.as_deref(),
            Some("phase-b")
        );
        assert_eq!(state.phases["phase-a::spike"].slug, "spike");
        assert_eq!(state.phases["phase-b::spike"].slug, "spike");
        assert!(!state.phases["phase-a"].legacy_read_only);
        assert_eq!(
            state.active_path.phase_path,
            vec!["phase-b".to_string(), "phase-b::spike".to_string()]
        );
        assert_eq!(
            state.active_path.phase_id.as_deref(),
            Some("phase-b::spike")
        );
        assert_eq!(
            state.phases["phase-a"].changes["parent-change"].implementation_status,
            WorkStatus::Complete
        );
    }

    #[test]
    fn migration_imports_ordered_changes_without_marking_phase_read_only() {
        let dir = tempdir().unwrap();
        let phase = dir
            .path()
            .join(".kbd-orchestrator/phases/legacy-planned-phase");
        fs::create_dir_all(&phase).unwrap();
        fs::write(
            phase.join("progress.json"),
            r#"{
                "phase":"legacy-planned-phase",
                "changes":[],
                "ordered_changes":[
                    {"id":"C-001","title":"First planned change"},
                    {"id":"C-002","title":"Second planned change"}
                ]
            }"#,
        )
        .unwrap();

        let runtime = Runtime::open(dir.path());
        let check = runtime.migrate_legacy_ledgers(false).unwrap();
        assert_eq!(check.legacy_read_only_phases, 0);
        runtime.migrate_legacy_ledgers(true).unwrap();
        let state = runtime.replay().unwrap();
        let migrated = &state.phases["legacy-planned-phase"];

        assert!(!migrated.legacy_read_only);
        assert_eq!(migrated.changes.len(), 2);
        assert_eq!(migrated.changes["C-001"].title, "First planned change");
    }

    #[test]
    fn projections_are_atomic_canonical_and_replayable() {
        let dir = tempdir().unwrap();
        let kbd = dir.path().join(".kbd-orchestrator");
        fs::create_dir_all(kbd.join("phases/phase-x")).unwrap();
        fs::write(
            kbd.join("current-waypoint.json"),
            r#"{"phase":"phase-x","status":"execute_ready","changes_completed":1,"changes_total":2,"changesCompleted":0}"#,
        )
        .unwrap();
        fs::write(
            kbd.join("phases/phase-x/progress.json"),
            r#"{"phase":"phase-x","changes_completed":1,"changes_total":2,"changes":{"a":{"status":"DONE"},"b":{"status":"PENDING"}}}"#,
        )
        .unwrap();
        let runtime = Runtime::open(dir.path());
        runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        runtime.migrate_legacy_ledgers(true).unwrap();

        let waypoint: serde_json::Value =
            serde_json::from_reader(File::open(kbd.join("current-waypoint.json")).unwrap())
                .unwrap();
        assert_eq!(waypoint["schemaVersion"], "5");
        assert_eq!(waypoint["generatedBy"], "kbd-runtime");
        assert_eq!(waypoint["sourceRevision"], 2);
        assert_eq!(waypoint["derivedRevision"], 2);
        assert_eq!(waypoint["frontier"][runtime.replica_id()], 2);
        assert_eq!(waypoint["implementationCompleted"], 1);
        assert!(waypoint.get("changes_completed").is_none());
        assert!(waypoint.get("changesCompleted").is_none());

        let progress: serde_json::Value =
            serde_json::from_reader(File::open(kbd.join("phases/phase-x/progress.json")).unwrap())
                .unwrap();
        assert_eq!(progress["schemaVersion"], "2");
        assert_eq!(progress["generatedBy"], "kbd-runtime");
        assert_eq!(progress["sourceRevision"], 2);
        assert_eq!(progress["frontier"][runtime.replica_id()], 2);
        assert!(progress["changes"].is_array());
        assert_eq!(progress["changes"][0]["id"], "a");

        let position: serde_json::Value =
            serde_json::from_reader(File::open(kbd.join("position.json")).unwrap()).unwrap();
        assert_eq!(position["cursor"][0], "phase-x");
        assert_eq!(position["sourceRevision"], 2);
        assert_eq!(position["frontier"][runtime.replica_id()], 2);
        assert_eq!(runtime.replay().unwrap().revision, 2);
        assert!(fs::read_to_string(kbd.join("phases/phase-x/tasks.md"))
            .unwrap()
            .contains("source revision 2"));

        let first_waypoint = fs::read(kbd.join("current-waypoint.json")).unwrap();
        let first_progress = fs::read(kbd.join("phases/phase-x/progress.json")).unwrap();
        let first_position = fs::read(kbd.join("position.json")).unwrap();
        runtime.write_compatibility_projections().unwrap();
        assert_eq!(
            first_waypoint,
            fs::read(kbd.join("current-waypoint.json")).unwrap()
        );
        assert_eq!(
            first_progress,
            fs::read(kbd.join("phases/phase-x/progress.json")).unwrap()
        );
        assert_eq!(first_position, fs::read(kbd.join("position.json")).unwrap());
    }

    #[test]
    fn migration_checks_then_applies_all_ledgers_with_backups() {
        let dir = tempdir().unwrap();
        let phases = dir.path().join(".kbd-orchestrator/phases");
        for phase in ["phase-a", "phase-b"] {
            let path = phases.join(phase);
            fs::create_dir_all(&path).unwrap();
            fs::write(
                path.join("progress.json"),
                format!(
                    r#"{{"phase":"{phase}","changes_total":1,"changes_completed":0,"changes":["change-1"]}}"#
                ),
            )
            .unwrap();
        }
        let runtime = Runtime::open(dir.path());
        let check = runtime.migrate_legacy_ledgers(false).unwrap();
        assert_eq!(check.progress_files, 2);
        assert_eq!(check.migrated_progress_files, 2);
        assert!(check.backup_directory.is_none());

        let applied = runtime.migrate_legacy_ledgers(true).unwrap();
        assert_eq!(applied.migrated_progress_files, 2);
        let backup_directory = applied.backup_directory.unwrap();
        assert!(backup_directory.exists());
        let manifest_path = applied.backup_manifest.unwrap();
        let manifest: MigrationBackupManifest =
            serde_json::from_reader(File::open(manifest_path).unwrap()).unwrap();
        assert_eq!(manifest.files.len(), 2);
        for entry in manifest.files {
            let backup = backup_directory.join(entry.backup);
            let bytes = fs::read(backup).unwrap();
            assert_eq!(entry.bytes, bytes.len() as u64);
            assert_eq!(entry.sha256, format!("{:x}", Sha256::digest(bytes)));
        }
        let project: ProjectManifest = serde_json::from_reader(
            File::open(dir.path().join(".prometheus/project.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(runtime.replay().unwrap().project_id, project.project_id);
        let after = runtime.migrate_legacy_ledgers(false).unwrap();
        assert_eq!(after.migrated_progress_files, 0);
    }

    #[test]
    fn shadow_comparison_is_read_only_and_detects_projection_drift() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        runtime.write_compatibility_projections().unwrap();
        assert!(runtime
            .compatibility_projection_mismatches()
            .unwrap()
            .is_empty());

        let position = dir.path().join(".kbd-orchestrator/position.json");
        fs::write(&position, b"{\"sourceRevision\":0}\n").unwrap();
        assert_eq!(
            runtime.compatibility_projection_mismatches().unwrap(),
            vec![PathBuf::from("position.json")]
        );
        assert_eq!(fs::read(position).unwrap(), b"{\"sourceRevision\":0}\n");
    }

    #[test]
    fn replicated_history_must_strictly_extend_the_local_chain() {
        let source_dir = tempdir().unwrap();
        let source = Runtime::open(source_dir.path());
        let initialized = source
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();

        let target_dir = tempdir().unwrap();
        let target = Runtime::open(target_dir.path());
        let imported = target.import_events(&source.events().unwrap()).unwrap();
        assert_eq!(imported, initialized);

        let divergent_dir = tempdir().unwrap();
        let divergent = Runtime::open(divergent_dir.path());
        divergent
            .initialize("project", "other-run", actor(ActorKind::Operator, "claude"))
            .unwrap();
        assert!(matches!(
            divergent.import_events(&source.events().unwrap()),
            Err(RuntimeError::InvalidState(_))
        ));
    }

    #[test]
    fn canonical_runtime_is_outside_repository_and_uses_tracked_identity() {
        let project = tempdir().unwrap();
        let data = tempdir().unwrap();
        let runtime = Runtime::open_canonical_at(project.path(), data.path()).unwrap();
        let manifest: ProjectManifest = serde_json::from_reader(
            File::open(project.path().join(".prometheus/project.json")).unwrap(),
        )
        .unwrap();
        assert!(runtime.runtime_root().starts_with(data.path()));
        assert!(!runtime.runtime_root().starts_with(project.path()));
        assert!(runtime.runtime_root().ends_with(&manifest.project_id));

        let reopened = Runtime::open_canonical_at(project.path(), data.path()).unwrap();
        assert_eq!(runtime.runtime_root(), reopened.runtime_root());
        assert_eq!(
            runtime.project_manifest(false).unwrap().unwrap(),
            reopened.project_manifest(false).unwrap().unwrap()
        );
        let registry = registry::ProjectRegistry::open_at(data.path());
        let registration = registry.lookup_path(project.path()).unwrap().unwrap();
        assert_eq!(registration.project_id, manifest.project_id);
        assert_eq!(
            registry.lookup_project(&manifest.project_id).unwrap().len(),
            1
        );
    }

    #[test]
    fn registered_open_recovers_torn_tail_before_loro_reconciliation() {
        let project = tempdir().unwrap();
        let data = tempdir().unwrap();
        let runtime = Runtime::open_canonical_at(project.path(), data.path()).unwrap();
        let initialized = runtime
            .initialize(
                runtime.project_manifest(false).unwrap().unwrap().project_id,
                "run-a",
                actor(ActorKind::Operator, "codex"),
            )
            .unwrap();
        let mut journal = OpenOptions::new()
            .append(true)
            .open(runtime.events_path())
            .unwrap();
        journal.write_all(b"{interrupted").unwrap();
        journal.sync_all().unwrap();

        let reopened =
            Runtime::open_registered_at(project.path(), data.path(), &initialized.project_id)
                .unwrap();

        assert_eq!(reopened.replay_authority().unwrap().revision, 1);
        assert!(fs::read(reopened.events_path()).unwrap().ends_with(b"\n"));
        assert_eq!(
            fs::read_dir(reopened.journal_root())
                .unwrap()
                .flatten()
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("events.jsonl.torn-")
                        && entry.path().extension().is_some_and(|ext| ext == "archive")
                })
                .count(),
            1
        );
    }

    #[test]
    fn startup_reconciles_a_fsynced_replica_journal_missing_from_loro() {
        let fixture = tempdir().unwrap();
        let project_id = Uuid::new_v4().to_string();
        let source = fixture.path().join("source");
        fs::create_dir_all(&source).unwrap();
        let source_runtime = Runtime::open(&source);
        source_runtime
            .initialize(&project_id, "run-a", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let source_events = source_runtime.events().unwrap();

        let project_root = fixture.path().join("checkout");
        fs::create_dir_all(&project_root).unwrap();
        atomic_json(
            &project_root.join(".prometheus/project.json"),
            &serde_json::json!({
                "schemaVersion": "1",
                "projectId": project_id.clone(),
                "repositoryFingerprint": "sha256:test"
            }),
        )
        .unwrap();
        let runtime = Runtime {
            root: fixture.path().join("data/projects").join(&project_id),
            project_root,
            replica_id: "replica-a".into(),
            key_storage: KeyStorage::PlatformCredentialStore,
            read_only: false,
        };
        fs::create_dir_all(runtime.journal_root()).unwrap();
        let mut journal = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(runtime.events_path())
            .unwrap();
        for event in &source_events {
            serde_json::to_writer(&mut journal, event).unwrap();
            journal.write_all(b"\n").unwrap();
        }
        journal.sync_all().unwrap();

        let document = runtime.project_document().unwrap();
        assert!(document.events().unwrap().is_empty());
        assert_eq!(runtime.reconcile_project_document().unwrap(), 1);
        assert_eq!(runtime.reconcile_project_document().unwrap(), 0);
        assert_eq!(document.events().unwrap(), source_events);
        assert_eq!(runtime.replay().unwrap().revision, 1);
    }

    #[test]
    fn same_machine_replicas_converge_through_the_shared_project_document() {
        let fixture = tempdir().unwrap();
        let first_root = fixture.path().join("first");
        let second_root = fixture.path().join("second");
        let data_root = fixture.path().join("data");
        fs::create_dir_all(&first_root).unwrap();
        fs::create_dir_all(second_root.join(".prometheus")).unwrap();

        let first = Runtime::open_canonical_at(&first_root, &data_root).unwrap();
        let project_id = first.project_manifest(false).unwrap().unwrap().project_id;
        let initialized = first
            .initialize(project_id, "run-a", actor(ActorKind::Operator, "operator"))
            .unwrap();
        fs::copy(
            first_root.join(".prometheus/project.json"),
            second_root.join(".prometheus/project.json"),
        )
        .unwrap();
        let second = Runtime::open_canonical_at(&second_root, &data_root).unwrap();
        assert_ne!(first.replica_id(), second.replica_id());
        assert_eq!(
            authority_state(second.replay().unwrap()),
            authority_state(initialized.clone())
        );

        let actor = actor(ActorKind::Harness, "replica-two");
        let claimed = second
            .execute_command(CommandEnvelope {
                schema_version: "2".into(),
                project_id: initialized.project_id.clone(),
                run_id: initialized.run_id.clone(),
                command_id: "claim-on-second".into(),
                frontier: Some(initialized.frontier.clone()),
                expected_revision: 0,
                actor: actor.clone(),
                command: CommandKind::ClaimAcquire {
                    scope: "phase:sync".into(),
                    mode: ClaimMode::Shared,
                    ttl_seconds: 300,
                    holder_id: actor.id,
                },
            })
            .unwrap()
            .state;
        assert_eq!(claimed.claims.len(), 1);
        let converged = first.replay().unwrap();
        assert_eq!(converged.frontier, claimed.frontier);
        assert_eq!(converged.claims, claimed.claims);
    }

    #[test]
    fn v1_journal_migration_archives_resigns_and_is_idempotent() {
        let fixture = tempdir().unwrap();
        let project_id = Uuid::new_v4().to_string();
        let source_runtime = Runtime::open(fixture.path().join("source"));
        let source_state = source_runtime
            .initialize(&project_id, "run-a", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let source_events = source_runtime.events().unwrap();

        let project_root = fixture.path().join("checkout");
        fs::create_dir_all(&project_root).unwrap();
        atomic_json(
            &project_root.join(".prometheus/project.json"),
            &serde_json::json!({
                "schemaVersion": "1",
                "projectId": project_id.clone(),
                "repositoryFingerprint": "sha256:test"
            }),
        )
        .unwrap();
        let runtime = Runtime {
            root: fixture.path().join("data/projects").join(&project_id),
            project_root,
            replica_id: "initial-replica".into(),
            key_storage: KeyStorage::PlatformCredentialStore,
            read_only: false,
        };
        fs::create_dir_all(runtime.runtime_root()).unwrap();
        let legacy_path = runtime.runtime_root().join("events.jsonl");
        write_event_file_atomic(&legacy_path, &source_events).unwrap();
        let legacy_bytes = fs::read(&legacy_path).unwrap();

        let signer = DeviceSigner::generate();
        let migrated = runtime
            .migrate_v1_journal_inner(Some(&signer))
            .unwrap()
            .unwrap();
        assert!(!migrated.already_migrated);
        assert!(!legacy_path.exists());
        assert_eq!(fs::read(&migrated.archive_journal).unwrap(), legacy_bytes);
        assert!(migrated.rollback_instructions.is_file());
        let active = read_event_file(&migrated.active_journal).unwrap();
        assert_eq!(active.len(), source_events.len());
        for (event, source) in active.iter().zip(&source_events) {
            assert_eq!(event.replica_id, "initial-replica");
            let provenance = event.migration_provenance.as_ref().unwrap();
            assert_eq!(provenance.source_event_id, source.event_id);
            assert_eq!(provenance.source_integrity_hash, source.integrity_hash);
        }
        let folded = runtime.project_document().unwrap().fold().unwrap();
        assert_eq!(folded.project_id, source_state.project_id);
        assert_eq!(folded.run_id, source_state.run_id);
        assert_eq!(folded.lifecycle, source_state.lifecycle);

        let repeated = runtime
            .migrate_v1_journal_inner(Some(&signer))
            .unwrap()
            .unwrap();
        assert!(repeated.already_migrated);
        assert_eq!(repeated.archive_sha256, migrated.archive_sha256);
    }

    #[test]
    fn every_repository_ledger_shape_migrates_in_a_recoverable_copy() {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let source_root = repository.join(".kbd-orchestrator");
        if !source_root.exists() {
            return;
        }
        let mut source_ledgers = Vec::new();
        collect_named_files(
            &source_root.join("phases"),
            "progress.json",
            &mut source_ledgers,
        )
        .unwrap();
        let dir = tempdir().unwrap();
        for source in &source_ledgers {
            let relative = source.strip_prefix(&source_root).unwrap();
            let destination = dir.path().join(".kbd-orchestrator").join(relative);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(source, destination).unwrap();
        }
        let runtime = Runtime::open(dir.path());
        let check = runtime.migrate_legacy_ledgers(false).unwrap();
        assert_eq!(check.progress_files, source_ledgers.len() as u64);
        assert_eq!(check.invalid_files, 0);
        let applied = runtime.migrate_legacy_ledgers(true).unwrap();
        assert_eq!(
            applied.migrated_progress_files,
            check.migrated_progress_files
        );
        assert!(applied.backup_directory.unwrap().exists());
        let mut migrated_ledgers = Vec::new();
        collect_named_files(
            &dir.path().join(".kbd-orchestrator/phases"),
            "progress.json",
            &mut migrated_ledgers,
        )
        .unwrap();
        for ledger in migrated_ledgers {
            let value: serde_json::Value =
                serde_json::from_reader(File::open(ledger).unwrap()).unwrap();
            assert_eq!(value["schemaVersion"], "2");
            let rows = value["changes"].as_array().unwrap();
            let ids = rows
                .iter()
                .map(|row| row["id"].as_str().unwrap())
                .collect::<std::collections::HashSet<_>>();
            assert_eq!(ids.len(), rows.len());
        }
        assert_eq!(
            runtime
                .migrate_legacy_ledgers(false)
                .unwrap()
                .migrated_progress_files,
            0
        );
    }
}

#[cfg(test)]
mod projection_ownership_tests {
    use super::projection_is_writable;
    use std::io::Write;

    fn temp_file(name: &str, body: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("kbd-proj-own-{}-{}", std::process::id(), name));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("progress.json");
        let mut f = std::fs::File::create(&path).unwrap();
        f.write_all(body.as_bytes()).unwrap();
        path
    }

    /// THE REGRESSION. A hand-maintained ledger must survive a transition.
    ///
    /// Before the guard, the projection loop replaced every phase's
    /// `progress.json` wholesale, destroying externally-maintained content with
    /// no warning — the file still existed and still parsed, so the next reader
    /// trusted a fabricated value.
    #[test]
    fn a_file_without_the_marker_is_not_ours_to_overwrite() {
        let path = temp_file(
            "foreign",
            r#"{"completion":{"implementation":{"completed":16,"total":16}}}"#,
        );
        assert!(
            !projection_is_writable(&path, false),
            "a progress.json with no `generatedBy` marker was written by \
             something else; overwriting it is committed data loss"
        );
    }

    #[test]
    fn a_file_the_runtime_wrote_is_ours() {
        let path = temp_file("ours", r#"{"generatedBy":"kbd-runtime","phase":"x"}"#);
        assert!(projection_is_writable(&path, false));
    }

    /// Another writer's marker must not be mistaken for ours.
    #[test]
    fn a_different_generator_is_not_ours() {
        let path = temp_file("other", r#"{"generatedBy":"some-other-tool"}"#);
        assert!(!projection_is_writable(&path, false));
    }

    /// Bootstrapping still works — the guard blocks overwrites, not first writes.
    #[test]
    fn an_absent_file_is_ours_so_the_first_write_still_happens() {
        let missing = std::env::temp_dir()
            .join(format!("kbd-proj-absent-{}", std::process::id()))
            .join("progress.json");
        let _ = std::fs::remove_file(&missing);
        assert!(
            projection_is_writable(&missing, false),
            "an absent path must be writable or the runtime can never \
             initialise a phase"
        );
    }

    /// A LEGACY ledger is ours — migration depends on it.
    ///
    /// `migrate_legacy_ledgers` reads these into runtime state (taking a backup
    /// first), and the projection loop writes them back in the new shape.
    ///
    /// CORRECTED when the GitHub issue landed. This test previously asserted the
    /// property against `projection_is_writable(_, false)` — the ROUTINE path — and
    /// so encoded the very defect that was reported: that an unattended
    /// projection may overwrite a legacy ledger. Migration is the path that may
    /// convert these files, because it backs them up first; routine projection
    /// is not, because it does not.
    #[test]
    fn a_legacy_ledger_is_ours_to_migrate() {
        let path = temp_file(
            "legacy",
            r#"{"phase":"phase-x","changes_completed":1,"changes_total":2}"#,
        );
        assert!(
            projection_is_writable(&path, true),
            "migrate_legacy_ledgers must be able to convert legacy counters; \
             refusing here would break migration while protecting nothing"
        );
        assert!(
            !projection_is_writable(&path, false),
            "...but the UNATTENDED loop must not touch it — that is the reported \
             issue, and it has no backup to fall back on"
        );
    }

    /// The discriminator must be the SHAPE, not merely the absence of a marker.
    ///
    /// A foreign file with neither the marker nor legacy counters is the case
    /// the guard exists for.
    #[test]
    fn a_modern_foreign_ledger_is_still_not_ours() {
        let path = temp_file(
            "modern-foreign",
            r#"{"completion":{"implementation":{"completed":16,"total":16}}}"#,
        );
        assert!(
            !projection_is_writable(&path, false),
            "a nested `completion` object with no marker is someone else's \
             modern ledger — the exact file that was silently destroyed"
        );
    }

    /// Unparseable bytes are NOT ours. Replacing a file we cannot understand is
    /// exactly the destructive act the guard exists to prevent.
    #[test]
    fn an_unparseable_file_is_not_ours() {
        let path = temp_file("corrupt", "{ this is not json");
        assert!(
            !projection_is_writable(&path, false),
            "a corrupt file is more likely someone's in-progress work than a \
             runtime artifact; skip it rather than destroy it"
        );
    }
}

#[cfg(test)]
mod projection_guard_migration_tests {
    use super::projection_is_writable;
    use std::io::Write;

    fn f(name: &str, body: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("kbd-guard-mig-{}-{}", std::process::id(), name));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("progress.json");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(body.as_bytes())
            .unwrap();
        path
    }

    /// THE FILE THIS BUG ACTUALLY DESTROYED.
    ///
    /// A ledger carrying legacy counters AND a modern `completion` object — the
    /// real shape in this repository. A counters-only check would still have
    /// overwritten it, which is why the discriminator is "old shape and nothing
    /// newer" rather than "has old keys".
    #[test]
    fn a_live_ledger_with_both_shapes_is_protected_from_routine_projection() {
        let path = f(
            "both-shapes",
            r#"{"changes_completed":16,"changes_total":16,
                "completion":{"implementation":{"completed":16,"total":16}}}"#,
        );
        assert!(
            !projection_is_writable(&path, false),
            "routine projection must NOT overwrite a live ledger that carries a \
             modern `completion` object, even though it also keeps the legacy \
             counters for compatibility"
        );
    }

    /// THE FILED ISSUE. A pure-LEGACY ledger must survive routine projection.
    ///
    /// Old counters, no `completion` key — the shape carried by phases that
    /// predate the current schema. The earlier rule
    /// (`has_legacy_counters && !has_modern_shape`) returned TRUE here, so the
    /// guard protected modern ledgers and left the oldest ones exposed to
    /// unattended overwrite. Reported against this repo; the reporter was right.
    ///
    /// Legacy CONVERSION still works — it belongs to `migrate_legacy_ledgers`,
    /// which backs the file up first. See the pair below.
    #[test]
    fn a_pure_legacy_ledger_survives_routine_projection() {
        let path = f(
            "pure-legacy-routine",
            r#"{"phase":"old","changes_completed":3,"changes_total":7}"#,
        );
        assert!(
            !projection_is_writable(&path, false),
            "routine projection overwrote a pure-legacy ledger. The unattended \
             loop has no backup, so this is committed data loss for exactly the \
             phases whose ledgers are oldest."
        );
    }

    /// ...and migration still converts that same file, because it backs up first.
    #[test]
    fn migration_still_converts_a_pure_legacy_ledger() {
        let path = f(
            "pure-legacy-migrating",
            r#"{"phase":"old","changes_completed":3,"changes_total":7}"#,
        );
        assert!(
            projection_is_writable(&path, true),
            "migrate_legacy_ledgers must still be able to convert legacy files; \
             refusing here would break migration while protecting nothing"
        );
    }

    /// A PURE-modern foreign ledger is protected even from `migrate --apply`.
    ///
    /// Found by end-to-end probe, not by reasoning: after the first version of
    /// this guard, `kbd migrate --apply` still rewrote a ledger holding only a
    /// `completion` object. A backup does not make that acceptable — the file
    /// is already in a current shape, so it is not a migration candidate at
    /// all, and rewriting it reintroduces the same data loss behind an
    /// operator-invoked command instead of an unattended one.
    #[test]
    fn a_pure_modern_foreign_ledger_survives_even_migration() {
        let path = f(
            "pure-modern",
            r#"{"completion":{"implementation":{"completed":16,"total":16}}}"#,
        );
        assert!(
            !projection_is_writable(&path, true),
            "a ledger with a modern `completion` object and NO legacy counters \
             is not a migration candidate; converting it destroys someone \
             else's current-shape data"
        );
    }

    /// ...but migration MUST still convert it, because a backup was taken first.
    ///
    /// This is the pair that forced the routine/migration split. One rule for
    /// both paths had to either break migration or leave the corruption in
    /// place — the repository's own ledgers made that unavoidable.
    #[test]
    fn migration_may_convert_the_same_file() {
        let path = f(
            "both-shapes-mig",
            r#"{"changes_completed":16,"changes_total":16,
                "completion":{"implementation":{"completed":16,"total":16}}}"#,
        );
        assert!(
            projection_is_writable(&path, true),
            "migrate_legacy_ledgers backs every ledger up before converting; \
             refusing here would break migration while protecting nothing"
        );
    }
}
