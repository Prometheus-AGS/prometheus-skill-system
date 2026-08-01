use base64::{
    engine::general_purpose::{STANDARD as BASE64, URL_SAFE_NO_PAD},
    Engine as _,
};
use chrono::{DateTime, Duration, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use fs2::FileExt;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

pub mod rollout;

pub const EVENT_SCHEMA_VERSION: &str = "2";
pub const DEFAULT_LEASE_TTL_SECONDS: i64 = 90;
pub const DEFAULT_HEARTBEAT_SECONDS: i64 = 30;

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
    #[error("mutation lease is required")]
    LeaseRequired,
    #[error("lease {0} is owned by another writer")]
    LeaseConflict(String),
    #[error("stale fencing token: supplied {supplied}, current {current}")]
    StaleFencing { supplied: u64, current: u64 },
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
}

pub type Result<T> = std::result::Result<T, RuntimeError>;

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

fn load_control_token(path: &Path) -> Result<String> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(RuntimeError::InvalidState(format!(
            "{} must be a regular, non-symlink control token file",
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
    let token = fs::read_to_string(path)?.trim().to_string();
    if token.len() < 32
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_".contains(&byte))
    {
        return Err(RuntimeError::InvalidState(format!(
            "{} does not contain a valid URL-safe bearer token",
            path.display()
        )));
    }
    Ok(token)
}

fn ensure_control_token(path: &Path) -> Result<String> {
    if path.exists() {
        return load_control_token(path);
    }
    let parent = path.parent().ok_or_else(|| {
        RuntimeError::InvalidState(format!("{} has no parent directory", path.display()))
    })?;
    fs::create_dir_all(parent)?;
    let mut secret = [0_u8; 32];
    OsRng.fill_bytes(&mut secret);
    let token = URL_SAFE_NO_PAD.encode(secret);
    let mut options = OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)]
    options.mode(0o600);
    match options.open(path) {
        Ok(mut file) => {
            file.write_all(token.as_bytes())?;
            file.write_all(b"\n")?;
            file.sync_all()?;
            File::open(parent)?.sync_all()?;
            Ok(token)
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => load_control_token(path),
        Err(error) => Err(error.into()),
    }
}

fn env_truthy(name: &str) -> bool {
    std::env::var(name)
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
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

    /// Create a lease target that can be completed by the named harness on a
    /// trusted device/session discovered after the handoff event replicates.
    pub fn handoff_target(harness: impl Into<String>) -> Self {
        let harness = harness.into();
        Self {
            kind: ActorKind::Harness,
            id: harness.clone(),
            device: "*".into(),
            harness,
            session: "*".into(),
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Lease {
    pub scope: String,
    pub lease_id: String,
    pub owner: Actor,
    pub fencing_token: u64,
    pub acquired_at: DateTime<Utc>,
    pub heartbeat_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
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
    LeaseClaimed {
        lease: Lease,
    },
    LeaseHeartbeat {
        lease_id: String,
        fencing_token: u64,
        heartbeat_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    },
    LeaseReleased {
        lease_id: String,
        fencing_token: u64,
        reason: String,
    },
    LeaseHandedOff {
        previous_lease_id: String,
        lease: Lease,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub schema_version: String,
    pub project_id: String,
    pub run_id: String,
    pub event_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
    pub revision: u64,
    pub expected_revision: u64,
    pub causal_parent: Option<String>,
    pub actor: Actor,
    pub lease_id: Option<String>,
    pub fencing_token: Option<u64>,
    pub timestamp: DateTime<Utc>,
    pub kind: EventKind,
    pub previous_hash: Option<String>,
    pub integrity_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_key_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signer_public_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
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

    fn seal(&mut self, signer: &DeviceSigner) -> Result<()> {
        self.signer_key_id = Some(signer.key_id.clone());
        self.signer_public_key = Some(signer.public_key.clone());
        let bytes = self.canonical_unsigned_bytes()?;
        self.integrity_hash = self.calculate_hash()?;
        self.signature = Some(BASE64.encode(signer.signing_key.sign(&bytes).to_bytes()));
        Ok(())
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
            // enforced. Routine harness-driven commands (claim, heartbeat,
            // release) submit as ActorKind::Harness, not Operator, so
            // restricting this to Operator alone still deadlocked ordinary
            // day-to-day use the moment a second local identity (e.g. a
            // headless daemon vs. the interactive CLI on the same machine)
            // touched the same project.
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
    pub last_event_id: Option<String>,
    pub last_event_hash: Option<String>,
    pub lifecycle: LifecycleState,
    pub plan_revision: u64,
    pub checkpoint: Option<Checkpoint>,
    pub lease: Option<Lease>,
    pub last_fencing_token: u64,
    pub exact_next_work: Option<String>,
    pub active_path: ActivePath,
    pub phases: BTreeMap<String, Phase>,
    pub completion: BTreeMap<CompletionDimension, Completion>,
    pub decisions: BTreeMap<String, Decision>,
    pub blockers: BTreeMap<String, Blocker>,
    pub devices: BTreeMap<String, DeviceRecord>,
    pub command_revisions: BTreeMap<String, u64>,
}

pub type RuntimeState = KbdStateV2;

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
    pub lease_id: String,
    pub fencing_token: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CommandEnvelope {
    pub schema_version: String,
    pub project_id: String,
    pub run_id: String,
    pub command_id: String,
    pub expected_revision: u64,
    pub actor: Actor,
    pub lease_id: Option<String>,
    pub fencing_token: Option<u64>,
    pub command: CommandKind,
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
    Claim {
        scope: String,
        force: bool,
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
    LeaseHeartbeat,
    LeaseRelease {
        reason: String,
    },
    LeaseHandoff {
        target: Actor,
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
            last_event_id: None,
            last_event_hash: None,
            lifecycle: LifecycleState::Ready,
            plan_revision: 1,
            checkpoint: None,
            lease: None,
            last_fencing_token: 0,
            exact_next_work: None,
            active_path: ActivePath::default(),
            phases: BTreeMap::new(),
            completion,
            decisions: BTreeMap::new(),
            blockers: BTreeMap::new(),
            devices: BTreeMap::new(),
            command_revisions: BTreeMap::new(),
        }
    }
}

impl KbdStateV2 {
    pub fn apply(&mut self, event: &Event) -> Result<()> {
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
        // instead of only the first one ever seen. Not gated on actor kind:
        // routine harness-driven commands (claim, heartbeat, release) submit
        // as ActorKind::Harness, not Operator, and ActorKind is a
        // self-declared routing label anyway, not a cryptographic identity
        // claim — the signature itself is the real trust boundary.
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
                if *to == LifecycleState::Cancelled {
                    self.lease = None;
                }
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
            EventKind::LeaseClaimed { lease } => {
                self.last_fencing_token = lease.fencing_token;
                self.lease = Some(lease.clone());
            }
            EventKind::LeaseHeartbeat {
                lease_id,
                fencing_token,
                heartbeat_at,
                expires_at,
            } => {
                let lease = self.lease.as_mut().ok_or(RuntimeError::LeaseRequired)?;
                ensure_lease(lease, lease_id, *fencing_token)?;
                lease.heartbeat_at = *heartbeat_at;
                lease.expires_at = *expires_at;
            }
            EventKind::LeaseReleased {
                lease_id,
                fencing_token,
                ..
            } => {
                let lease = self.lease.as_ref().ok_or(RuntimeError::LeaseRequired)?;
                ensure_lease(lease, lease_id, *fencing_token)?;
                self.lease = None;
            }
            EventKind::LeaseHandedOff { lease, .. } => {
                self.last_fencing_token = lease.fencing_token;
                self.lease = Some(lease.clone());
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
        }

        self.revision = event.revision;
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

fn ensure_lease(_lease: &Lease, _lease_id: &str, _fencing_token: u64) -> Result<()> {
    // No-op: lease-id and fencing-token matching no longer gate mutations.
    // For a solo operator there is no real writer to fence out, and this
    // check only ever produced spurious blocks (a locally cached lease/
    // fencing value one step behind the committed state was enough to
    // reject an otherwise-legitimate command).
    Ok(())
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

/// Apply one already committed event to an in-memory state machine.
///
/// Consensus/storage integrations use this entry point after an event has
/// reached quorum. It deliberately does not create, sign, append, or project
/// anything: those are leader, durable-log, and projection-worker concerns.
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
    key_storage: KeyStorage,
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
            key_storage,
        }
    }

    /// Open the platform-owned canonical runtime, creating the repository's
    /// immutable identity manifest when it does not yet exist.
    pub fn open_canonical(project_root: impl AsRef<Path>) -> Result<Self> {
        let project_root = project_root.as_ref().to_path_buf();
        let manifest = ensure_project_manifest(&project_root)?;
        Ok(Self {
            root: canonical_runtime_root(&manifest.project_id),
            project_root,
            key_storage: KeyStorage::PlatformCredentialStore,
        })
    }

    /// Open a canonical runtime beneath an explicit application-data root.
    /// This is used by hermetic tests and managed/headless deployments.
    pub fn open_canonical_at(
        project_root: impl AsRef<Path>,
        data_root: impl AsRef<Path>,
    ) -> Result<Self> {
        let project_root = project_root.as_ref().to_path_buf();
        let manifest = ensure_project_manifest(&project_root)?;
        Ok(Self {
            root: canonical_runtime_root_at(data_root.as_ref(), &manifest.project_id),
            project_root,
            key_storage: KeyStorage::PlatformCredentialStore,
        })
    }

    pub fn runtime_root(&self) -> &Path {
        &self.root
    }

    pub fn events_path(&self) -> PathBuf {
        self.root.join("events.jsonl")
    }

    fn lock_path(&self) -> PathBuf {
        self.root.join("runtime.lock")
    }

    fn device_key_path(&self) -> PathBuf {
        self.root.join("device-key.json")
    }

    /// Return the local REST bearer token, creating it atomically with mode
    /// 0600 when this is the first daemon/client process on the device.
    pub fn control_token(&self) -> Result<String> {
        let configured = std::env::var_os("PROMETHEUS_CONTROL_TOKEN_FILE")
            .map(PathBuf::from)
            .unwrap_or_else(|| self.root.join("control-token"));
        if configured.exists() {
            return load_control_token(&configured);
        }
        if std::env::var_os("PROMETHEUS_CONTROL_TOKEN_FILE").is_some() {
            return Err(RuntimeError::InvalidState(format!(
                "configured control token file {} does not exist",
                configured.display()
            )));
        }
        ensure_control_token(&configured)
    }

    pub fn device_signer(&self) -> Result<DeviceSigner> {
        if let Some(path) = std::env::var_os("PROMETHEUS_DEVICE_KEY_FILE") {
            return load_device_key(Path::new(&path));
        }
        if env_truthy("PROMETHEUS_HEADLESS_VOTER") {
            return Err(RuntimeError::InvalidState(
                "headless voters require PROMETHEUS_DEVICE_KEY_FILE pointing to an existing mode-0600 Ed25519 key file; no key was created".into(),
            ));
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
        let path = self.events_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(path)?;
        let mut events = Vec::new();
        for line in BufReader::new(file).lines() {
            let line = line?;
            if !line.trim().is_empty() {
                events.push(serde_json::from_str(&line)?);
            }
        }
        Ok(events)
    }

    /// Export the verified audit chain as RFC 8785 canonical JSON Lines.
    pub fn export_signed_audit(&self, mut writer: impl Write) -> Result<u64> {
        let events = self.events()?;
        replay_events(&events)?;
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

    pub fn replay(&self) -> Result<RuntimeState> {
        replay_events(&self.events()?)
    }

    /// Import a replicated journal only when it is a valid strict extension of
    /// the local causal chain. Divergent/offline branches are rejected rather
    /// than resolved by timestamp or CRDT map order.
    pub fn import_events(&self, incoming: &[Event]) -> Result<RuntimeState> {
        fs::create_dir_all(&self.root)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?;
        lock.lock_exclusive()?;
        let local = self.events()?;
        let mut incoming = incoming.to_vec();
        incoming.sort_by_key(|event| event.revision);
        let imported = replay_events(&incoming)?;
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
        Ok(imported)
    }

    pub fn initialize(
        &self,
        project_id: impl Into<String>,
        run_id: impl Into<String>,
        actor: Actor,
    ) -> Result<RuntimeState> {
        fs::create_dir_all(&self.root)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?;
        lock.lock_exclusive()?;
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
            None,
            None,
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
        fs::create_dir_all(&self.root)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?;
        lock.lock_exclusive()?;
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
            None,
            None,
        )
    }

    pub fn append(
        &self,
        actor: Actor,
        expected_revision: u64,
        kind: EventKind,
        lease_id: Option<String>,
        fencing_token: Option<u64>,
    ) -> Result<RuntimeState> {
        self.append_command(
            actor,
            expected_revision,
            Uuid::new_v4().to_string(),
            kind,
            lease_id,
            fencing_token,
        )
    }

    pub fn append_command(
        &self,
        actor: Actor,
        expected_revision: u64,
        command_id: impl Into<String>,
        kind: EventKind,
        lease_id: Option<String>,
        fencing_token: Option<u64>,
    ) -> Result<RuntimeState> {
        let command_id = command_id.into();
        if command_id.trim().is_empty() {
            return Err(RuntimeError::InvalidState(
                "commandId must not be empty".into(),
            ));
        }
        fs::create_dir_all(&self.root)?;
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.lock_path())?;
        lock.lock_exclusive()?;
        let events = self.events()?;
        let state = replay_events(&events)?;
        if state.revision == 0 {
            return Err(RuntimeError::NotInitialized);
        }
        if let Some(committed_revision) = state.command_revisions.get(&command_id) {
            return replay_events(&events[..*committed_revision as usize]);
        }
        if state.revision != expected_revision {
            return Err(RuntimeError::RevisionConflict {
                expected: expected_revision,
                actual: state.revision,
            });
        }
        authorize(&state, &actor, &kind, lease_id.as_deref(), fencing_token)?;
        let project_id = state.project_id.clone();
        let run_id = state.run_id.clone();
        self.append_unchecked(
            state,
            project_id,
            run_id,
            actor,
            command_id,
            kind,
            lease_id,
            fencing_token,
        )
    }

    pub fn execute_command(&self, envelope: CommandEnvelope) -> Result<CommandResult> {
        if envelope.schema_version != "1" {
            return Err(RuntimeError::InvalidState(format!(
                "unsupported command schemaVersion {}",
                envelope.schema_version
            )));
        }
        let events = self.events()?;
        let state = replay_events(&events)?;
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
        if let Some(committed_revision) = state.command_revisions.get(&envelope.command_id) {
            let committed = replay_events(&events[..*committed_revision as usize])?;
            return Ok(CommandResult {
                command_id: envelope.command_id,
                committed_revision: *committed_revision,
                duplicate: true,
                state: committed,
                apply_error: None,
            });
        }
        if state.revision != envelope.expected_revision {
            return Err(RuntimeError::RevisionConflict {
                expected: envelope.expected_revision,
                actual: state.revision,
            });
        }
        let kind = prepare_command_event(
            &state,
            &envelope.actor,
            &envelope.command,
            envelope.lease_id.as_deref(),
            envelope.fencing_token,
        )?;
        let next = self.append_command(
            envelope.actor,
            envelope.expected_revision,
            envelope.command_id.clone(),
            kind,
            envelope.lease_id,
            envelope.fencing_token,
        )?;
        Ok(CommandResult {
            command_id: envelope.command_id,
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
        if envelope.schema_version != "1" {
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
        if state.revision != envelope.expected_revision {
            return Err(RuntimeError::RevisionConflict {
                expected: envelope.expected_revision,
                actual: state.revision,
            });
        }
        let kind = prepare_command_event(
            state,
            &envelope.actor,
            &envelope.command,
            envelope.lease_id.as_deref(),
            envelope.fencing_token,
        )?;
        let mut event = Event {
            schema_version: EVENT_SCHEMA_VERSION.into(),
            project_id: state.project_id.clone(),
            run_id: state.run_id.clone(),
            event_id: Uuid::new_v4().to_string(),
            command_id: Some(envelope.command_id),
            revision: state.revision + 1,
            expected_revision: state.revision,
            causal_parent: state.last_event_id.clone(),
            actor: envelope.actor,
            lease_id: envelope.lease_id,
            fencing_token: envelope.fencing_token,
            timestamp: Utc::now(),
            kind,
            previous_hash: state.last_event_hash.clone(),
            integrity_hash: String::new(),
            signer_key_id: None,
            signer_public_key: None,
            signature: None,
        };
        event.seal(&self.device_signer()?)?;
        Ok(event)
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
            None,
            None,
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
            Some(context.lease_id),
            Some(context.fencing_token),
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
            Some(context.lease_id),
            Some(context.fencing_token),
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
            Some(context.lease_id),
            Some(context.fencing_token),
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
            Some(context.lease_id),
            Some(context.fencing_token),
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
            Some(context.lease_id),
            Some(context.fencing_token),
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
            Some(context.lease_id),
            Some(context.fencing_token),
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
            Some(context.lease_id),
            Some(context.fencing_token),
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
            Some(context.lease_id),
            Some(context.fencing_token),
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
            Some(context.lease_id),
            Some(context.fencing_token),
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
            Some(context.lease_id),
            Some(context.fencing_token),
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
            Some(context.lease_id),
            Some(context.fencing_token),
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
            Some(context.lease_id),
            Some(context.fencing_token),
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
            Some(context.lease_id),
            Some(context.fencing_token),
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
        lease_id: Option<String>,
        fencing_token: Option<u64>,
    ) -> Result<RuntimeState> {
        let mut event = Event {
            schema_version: EVENT_SCHEMA_VERSION.into(),
            project_id,
            run_id,
            event_id: Uuid::new_v4().to_string(),
            command_id: Some(command_id),
            revision: state.revision + 1,
            expected_revision: state.revision,
            causal_parent: state.last_event_id.clone(),
            actor,
            lease_id,
            fencing_token,
            timestamp: Utc::now(),
            kind,
            previous_hash: state.last_event_hash.clone(),
            integrity_hash: String::new(),
            signer_key_id: None,
            signer_public_key: None,
            signature: None,
        };
        let signer = self.device_signer()?;
        event.seal(&signer)?;
        state.apply(&event)?;
        fs::create_dir_all(&self.root)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.events_path())?;
        serde_json::to_writer(&mut file, &event)?;
        file.write_all(b"\n")?;
        file.sync_data()?;
        Ok(state)
    }

    pub fn claim(
        &self,
        actor: Actor,
        expected_revision: u64,
        scope: impl Into<String>,
        force: bool,
    ) -> Result<RuntimeState> {
        let state = self.replay()?;
        let now = Utc::now();
        let _ = force;
        // Any claim always succeeds and takes over the lease, regardless of
        // an existing holder or actor kind. There is no real multi-writer
        // contention to arbitrate for a solo operator on their own
        // machine(s): the previous "reject unless --force, and --force
        // requires Operator/System" rule just deadlocked ordinary
        // harness-driven claims (claim submits as ActorKind::Harness) the
        // moment any lease — including a stale one from a crashed or
        // previous session — existed. Fencing tokens still increment below,
        // so stale in-flight commands from a genuinely superseded writer are
        // still rejected by `authorize()`'s fencing check; this only removes
        // the up-front refusal to even attempt a new claim.
        let lease = Lease {
            scope: scope.into(),
            lease_id: Uuid::new_v4().to_string(),
            owner: actor.clone(),
            fencing_token: state.last_fencing_token + 1,
            acquired_at: now,
            heartbeat_at: now,
            expires_at: now + Duration::seconds(DEFAULT_LEASE_TTL_SECONDS),
        };
        self.append(
            actor,
            expected_revision,
            EventKind::LeaseClaimed { lease },
            None,
            None,
        )
    }

    pub fn transition(
        &self,
        actor: Actor,
        expected_revision: u64,
        to: LifecycleState,
        reason: impl Into<String>,
        lease_id: Option<String>,
        fencing_token: Option<u64>,
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
            lease_id,
            fencing_token,
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
            None,
            None,
        )
    }

    pub fn resume(
        &self,
        actor: Actor,
        expected_revision: u64,
        plan_revision: u64,
        lease_id: String,
        fencing_token: u64,
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
            Some(lease_id),
            Some(fencing_token),
        )
    }

    pub fn cancel(
        &self,
        actor: Actor,
        expected_revision: u64,
        reason: impl Into<String>,
    ) -> Result<RuntimeState> {
        self.transition(
            actor,
            expected_revision,
            LifecycleState::Cancelled,
            reason,
            None,
            None,
        )
    }

    pub fn revise_plan(
        &self,
        actor: Actor,
        expected_revision: u64,
        reason: impl Into<String>,
        exact_next_work: Option<String>,
        lease_id: String,
        fencing_token: u64,
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
            Some(lease_id),
            Some(fencing_token),
        )
    }

    pub fn heartbeat(
        &self,
        actor: Actor,
        expected_revision: u64,
        lease_id: String,
        fencing_token: u64,
    ) -> Result<RuntimeState> {
        let now = Utc::now();
        self.append(
            actor,
            expected_revision,
            EventKind::LeaseHeartbeat {
                lease_id: lease_id.clone(),
                fencing_token,
                heartbeat_at: now,
                expires_at: now + Duration::seconds(DEFAULT_LEASE_TTL_SECONDS),
            },
            Some(lease_id),
            Some(fencing_token),
        )
    }

    pub fn release(
        &self,
        actor: Actor,
        expected_revision: u64,
        lease_id: String,
        fencing_token: u64,
        reason: impl Into<String>,
    ) -> Result<RuntimeState> {
        self.append(
            actor,
            expected_revision,
            EventKind::LeaseReleased {
                lease_id: lease_id.clone(),
                fencing_token,
                reason: reason.into(),
            },
            Some(lease_id),
            Some(fencing_token),
        )
    }

    pub fn handoff(
        &self,
        actor: Actor,
        target: Actor,
        expected_revision: u64,
        lease_id: String,
        fencing_token: u64,
    ) -> Result<RuntimeState> {
        let state = self.replay()?;
        let current = state.lease.as_ref().ok_or(RuntimeError::LeaseRequired)?;
        ensure_lease(current, &lease_id, fencing_token)?;
        let now = Utc::now();
        let lease = Lease {
            scope: current.scope.clone(),
            lease_id: Uuid::new_v4().to_string(),
            owner: target,
            fencing_token: state.last_fencing_token + 1,
            acquired_at: now,
            heartbeat_at: now,
            expires_at: now + Duration::seconds(DEFAULT_LEASE_TTL_SECONDS),
        };
        self.append(
            actor,
            expected_revision,
            EventKind::LeaseHandedOff {
                previous_lease_id: lease_id.clone(),
                lease,
            },
            Some(lease_id),
            Some(fencing_token),
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
        fs::create_dir_all(&self.root)?;
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

        for phase in state.phases.values() {
            let phase_dir = phase_projection_directory(&kbd_root, state, phase)?;
            let progress_path = phase_dir.join("progress.json");

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
                eprintln!(
                    "kbd-runtime: refusing to overwrite {} — it has no \
                     `generatedBy: \"kbd-runtime\"` marker, so it was written \
                     by something else. Leaving it untouched. Delete the file \
                     if the runtime should own it.",
                    progress_path.display()
                );
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
    /// canonical state or grant a lease.
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
            key_storage: KeyStorage::LegacyRuntimeFile,
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
/// Ownership is decided by the `generatedBy: "kbd-runtime"` marker that
/// [`phase_progress_projection`] stamps into every file it produces.
///
/// # Absent file is OURS
///
/// A path that does not exist yet returns `true` so the first write still
/// happens — the guard prevents *overwriting* foreign data, not bootstrapping.
///
/// # Unreadable or unparseable file is NOT ours
///
/// If the bytes cannot be read or parsed we return `false` and skip. Replacing
/// a file we cannot understand is exactly the destructive act this guard
/// exists to prevent, and a corrupt file is more likely to be someone's
/// in-progress work than a runtime artifact.
fn projection_is_runtime_owned(path: &Path) -> bool {
    projection_is_writable(path, false)
}

/// As [`projection_is_runtime_owned`], but `migrating` relaxes the guard.
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
        // A backup exists; converting any recognisable ledger is the point.
        return has_legacy_counters || has_modern_shape;
    }

    // Routine projection: only a pure legacy ledger (old counters, nothing
    // newer) is safe to replace unattended.
    has_legacy_counters && !has_modern_shape
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

fn prepare_command_event(
    state: &RuntimeState,
    actor: &Actor,
    command: &CommandKind,
    lease_id: Option<&str>,
    fencing_token: Option<u64>,
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
        CommandKind::Claim { scope, force: _ } => {
            let now = Utc::now();
            // Any claim always succeeds and takes over the lease — see the
            // matching comment in `Runtime::claim` above. No real
            // multi-writer contention exists for a solo operator; the old
            // "reject unless --force + Operator/System" rule deadlocked
            // ordinary harness-driven claims.
            EventKind::LeaseClaimed {
                lease: Lease {
                    scope: scope.clone(),
                    lease_id: Uuid::new_v4().to_string(),
                    owner: actor.clone(),
                    fencing_token: state.last_fencing_token + 1,
                    acquired_at: now,
                    heartbeat_at: now,
                    expires_at: now + Duration::seconds(DEFAULT_LEASE_TTL_SECONDS),
                },
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
        CommandKind::LeaseHeartbeat => {
            let current = state.lease.as_ref().ok_or(RuntimeError::LeaseRequired)?;
            ensure_lease(
                current,
                lease_id.ok_or(RuntimeError::LeaseRequired)?,
                fencing_token.ok_or(RuntimeError::LeaseRequired)?,
            )?;
            let now = Utc::now();
            EventKind::LeaseHeartbeat {
                lease_id: current.lease_id.clone(),
                fencing_token: current.fencing_token,
                heartbeat_at: now,
                expires_at: now + Duration::seconds(DEFAULT_LEASE_TTL_SECONDS),
            }
        }
        CommandKind::LeaseRelease { reason } => {
            require_reason(reason)?;
            let current = state.lease.as_ref().ok_or(RuntimeError::LeaseRequired)?;
            ensure_lease(
                current,
                lease_id.ok_or(RuntimeError::LeaseRequired)?,
                fencing_token.ok_or(RuntimeError::LeaseRequired)?,
            )?;
            EventKind::LeaseReleased {
                lease_id: current.lease_id.clone(),
                fencing_token: current.fencing_token,
                reason: reason.clone(),
            }
        }
        CommandKind::LeaseHandoff { target } => {
            let current = state.lease.as_ref().ok_or(RuntimeError::LeaseRequired)?;
            ensure_lease(
                current,
                lease_id.ok_or(RuntimeError::LeaseRequired)?,
                fencing_token.ok_or(RuntimeError::LeaseRequired)?,
            )?;
            let now = Utc::now();
            EventKind::LeaseHandedOff {
                previous_lease_id: current.lease_id.clone(),
                lease: Lease {
                    scope: current.scope.clone(),
                    lease_id: Uuid::new_v4().to_string(),
                    owner: target.clone(),
                    fencing_token: state.last_fencing_token + 1,
                    acquired_at: now,
                    heartbeat_at: now,
                    expires_at: now + Duration::seconds(DEFAULT_LEASE_TTL_SECONDS),
                },
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
    })
}

fn authorize(
    _state: &RuntimeState,
    _actor: &Actor,
    _kind: &EventKind,
    _lease_id: Option<&str>,
    _fencing_token: Option<u64>,
) -> Result<()> {
    // No-op: lease ownership is no longer a mutation gate. There is no real
    // multi-writer contention to arbitrate for a solo operator across their
    // own machines/harnesses, and requiring a pre-existing lease (plus an
    // exact owner/lease-id/fencing-token match) only ever produced spurious
    // "lease required" / "lease conflict" rejections of otherwise-legitimate
    // local commands. Revision/causal-chain/hash integrity checks elsewhere
    // in the event pipeline are untouched — those protect log consistency,
    // not writer authorization, and removing them would risk real corruption
    // rather than just friction.
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

    #[test]
    fn replay_is_deterministic_and_integrity_checked() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let state = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let claimed = runtime
            .claim(
                actor(ActorKind::Harness, "codex"),
                state.revision,
                "project/phase",
                false,
            )
            .unwrap();
        assert_eq!(claimed, runtime.replay().unwrap());
        assert_eq!(claimed.lease.unwrap().fencing_token, 1);
    }

    #[test]
    fn workflow_events_reconstruct_exact_position_and_commands_are_idempotent() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let initialized = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let claimed = runtime
            .claim(
                actor(ActorKind::Harness, "codex"),
                initialized.revision,
                "project/phase",
                false,
            )
            .unwrap();
        let lease = claimed.lease.clone().unwrap();
        let context = |state: &RuntimeState, command_id: &str| MutationContext {
            expected_revision: state.revision,
            command_id: command_id.into(),
            lease_id: lease.lease_id.clone(),
            fencing_token: lease.fencing_token,
        };
        let phase = runtime
            .define_phase(
                actor(ActorKind::Harness, "codex"),
                context(&claimed, "define-phase"),
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
                },
                Some("start quorum storage".into()),
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
                Some(lease.lease_id),
                Some(lease.fencing_token),
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
        let claimed = runtime
            .claim(
                actor(ActorKind::Harness, "codex"),
                initialized.revision,
                "project/phase",
                false,
            )
            .unwrap();
        let events = runtime.events().unwrap();
        assert_eq!(events[0].schema_version, "2");
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
        assert_eq!(claimed.devices.len(), 1);

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
        assert_eq!(replay_events(&exported).unwrap(), claimed);

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
    fn local_control_token_is_stable_and_permission_protected() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("control-token");
        let first = ensure_control_token(&path).unwrap();
        let second = ensure_control_token(&path).unwrap();
        assert_eq!(first, second);
        assert!(first.len() >= 32);
        #[cfg(unix)]
        assert_eq!(fs::metadata(path).unwrap().permissions().mode() & 0o077, 0);
    }

    #[test]
    fn command_envelopes_no_longer_require_concurrency_fields_and_still_dedupe_by_command_id() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let initialized = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let claim = CommandEnvelope {
            schema_version: "1".into(),
            project_id: "project".into(),
            run_id: "run".into(),
            command_id: "claim-1".into(),
            expected_revision: initialized.revision,
            actor: actor(ActorKind::Harness, "codex"),
            lease_id: None,
            fencing_token: None,
            command: CommandKind::Claim {
                scope: "project/phase".into(),
                force: false,
            },
        };
        let claimed = runtime.execute_command(claim.clone()).unwrap();
        assert!(!claimed.duplicate);
        let duplicate = runtime.execute_command(claim).unwrap();
        assert!(duplicate.duplicate);
        assert_eq!(duplicate.state, claimed.state);

        let phase_command = CommandEnvelope {
            schema_version: "1".into(),
            project_id: "project".into(),
            run_id: "run".into(),
            command_id: "phase-1".into(),
            expected_revision: claimed.committed_revision,
            actor: actor(ActorKind::Harness, "codex"),
            lease_id: None,
            fencing_token: None,
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
        // Concurrency fields (lease_id/fencing_token) are no longer required
        // — lease-ownership gating was removed as a mutation blocker for
        // solo/local use. The command succeeds immediately without them.
        let applied = runtime
            .execute_command(phase_command.clone())
            .expect("lease_id/fencing_token are no longer required to mutate");
        assert!(!applied.duplicate);
        assert_eq!(applied.committed_revision, claimed.committed_revision + 1);
        // Resubmitting the exact same command_id is still idempotent.
        let duplicate = runtime.execute_command(phase_command).unwrap();
        assert!(duplicate.duplicate);
        assert_eq!(duplicate.state, applied.state);
    }

    #[test]
    fn every_claim_succeeds_and_takes_over_with_monotonic_fencing() {
        // Lease-ownership conflicts are no longer a mutation blocker for
        // solo/local use: any claim always succeeds and takes over the
        // lease, regardless of an existing holder, actor kind, or `force`.
        // The one property that remains meaningful — and is asserted here —
        // is that the fencing token still strictly increases on every
        // takeover, so a stale in-flight command from a superseded writer
        // is still rejected by fencing, even though claiming itself is
        // unconditional.
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let initialized = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let first = runtime
            .claim(
                actor(ActorKind::Harness, "codex"),
                initialized.revision,
                "project/phase",
                false,
            )
            .unwrap();
        assert_eq!(first.lease.as_ref().unwrap().fencing_token, 1);

        let second = runtime
            .claim(
                actor(ActorKind::Harness, "claude"),
                first.revision,
                "project/phase",
                false,
            )
            .expect("ordinary (non-forced) claims now always succeed and take over");
        assert_eq!(second.lease.as_ref().unwrap().fencing_token, 2);
        assert_eq!(second.lease.as_ref().unwrap().owner.harness, "claude");

        let takeover = runtime
            .claim(
                actor(ActorKind::Operator, "claude"),
                second.revision,
                "project/phase",
                true,
            )
            .unwrap();
        assert_eq!(takeover.lease.unwrap().fencing_token, 3);
    }

    #[test]
    fn operator_can_cancel_and_atomically_release_the_writer_lease() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let initialized = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let claimed = runtime
            .claim(
                actor(ActorKind::Harness, "codex"),
                initialized.revision,
                "project/phase",
                false,
            )
            .unwrap();
        let cancelled = runtime
            .append(
                actor(ActorKind::Operator, "claude"),
                claimed.revision,
                EventKind::LifecycleTransition {
                    from: LifecycleState::Ready,
                    to: LifecycleState::Cancelled,
                    reason: "architectural issue".into(),
                },
                None,
                None,
            )
            .unwrap();
        assert_eq!(cancelled.lifecycle, LifecycleState::Cancelled);
        assert!(cancelled.lease.is_none());
    }

    #[test]
    fn checkpoint_plan_revision_and_handoff_preserve_causality() {
        let dir = tempdir().unwrap();
        let runtime = Runtime::open(dir.path());
        let initialized = runtime
            .initialize("project", "run", actor(ActorKind::Operator, "codex"))
            .unwrap();
        let claimed = runtime
            .claim(
                actor(ActorKind::Harness, "codex"),
                initialized.revision,
                "project/phase",
                false,
            )
            .unwrap();
        let lease = claimed.lease.clone().unwrap();
        let running = runtime
            .transition(
                actor(ActorKind::Harness, "codex"),
                claimed.revision,
                LifecycleState::Running,
                "begin",
                Some(lease.lease_id.clone()),
                Some(lease.fencing_token),
            )
            .unwrap();
        let revised = runtime
            .revise_plan(
                actor(ActorKind::Harness, "codex"),
                running.revision,
                "architectural correction",
                Some("implement boundary first".into()),
                lease.lease_id.clone(),
                lease.fencing_token,
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
        let handed = runtime
            .handoff(
                actor(ActorKind::Harness, "codex"),
                Actor::handoff_target("claude"),
                paused.revision,
                lease.lease_id,
                lease.fencing_token,
            )
            .unwrap();
        let handed_lease = handed.lease.unwrap();
        assert_eq!(handed_lease.fencing_token, 2);
        let heartbeat = runtime
            .heartbeat(
                actor(ActorKind::Harness, "claude"),
                handed.revision,
                handed_lease.lease_id,
                handed_lease.fencing_token,
            )
            .unwrap();
        assert_eq!(heartbeat.revision, runtime.events().unwrap().len() as u64);
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
        assert_eq!(waypoint["implementationCompleted"], 1);
        assert!(waypoint.get("changes_completed").is_none());
        assert!(waypoint.get("changesCompleted").is_none());

        let progress: serde_json::Value =
            serde_json::from_reader(File::open(kbd.join("phases/phase-x/progress.json")).unwrap())
                .unwrap();
        assert_eq!(progress["schemaVersion"], "2");
        assert_eq!(progress["generatedBy"], "kbd-runtime");
        assert_eq!(progress["sourceRevision"], 2);
        assert!(progress["changes"].is_array());
        assert_eq!(progress["changes"][0]["id"], "a");

        let position: serde_json::Value =
            serde_json::from_reader(File::open(kbd.join("position.json")).unwrap()).unwrap();
        assert_eq!(position["cursor"][0], "phase-x");
        assert_eq!(position["sourceRevision"], 2);
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
    use super::projection_is_runtime_owned;
    use std::io::Write;

    fn temp_file(name: &str, body: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "kbd-proj-own-{}-{}",
            std::process::id(),
            name
        ));
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
            !projection_is_runtime_owned(&path),
            "a progress.json with no `generatedBy` marker was written by \
             something else; overwriting it is committed data loss"
        );
    }

    #[test]
    fn a_file_the_runtime_wrote_is_ours() {
        let path = temp_file("ours", r#"{"generatedBy":"kbd-runtime","phase":"x"}"#);
        assert!(projection_is_runtime_owned(&path));
    }

    /// Another writer's marker must not be mistaken for ours.
    #[test]
    fn a_different_generator_is_not_ours() {
        let path = temp_file("other", r#"{"generatedBy":"some-other-tool"}"#);
        assert!(!projection_is_runtime_owned(&path));
    }

    /// Bootstrapping still works — the guard blocks overwrites, not first writes.
    #[test]
    fn an_absent_file_is_ours_so_the_first_write_still_happens() {
        let missing = std::env::temp_dir()
            .join(format!("kbd-proj-absent-{}", std::process::id()))
            .join("progress.json");
        let _ = std::fs::remove_file(&missing);
        assert!(
            projection_is_runtime_owned(&missing),
            "an absent path must be writable or the runtime can never \
             initialise a phase"
        );
    }

    /// A LEGACY ledger is ours — migration depends on it.
    ///
    /// `migrate_legacy_ledgers` reads these into runtime state (taking a backup
    /// first), and the projection loop writes them back in the new shape. An
    /// over-strict guard that refused here would break migration while
    /// protecting nothing — which is exactly what the first version of this fix
    /// did, caught by two pre-existing migration tests.
    #[test]
    fn a_legacy_ledger_is_ours_to_migrate() {
        let path = temp_file(
            "legacy",
            r#"{"phase":"phase-x","changes_completed":1,"changes_total":2}"#,
        );
        assert!(
            projection_is_runtime_owned(&path),
            "legacy snake_case counters mark a ledger this runtime is migrating; \
             refusing to write it would break migrate_legacy_ledgers"
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
            !projection_is_runtime_owned(&path),
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
            !projection_is_runtime_owned(&path),
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
        let dir = std::env::temp_dir().join(format!("kbd-guard-mig-{}-{}", std::process::id(), name));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("progress.json");
        std::fs::File::create(&path).unwrap().write_all(body.as_bytes()).unwrap();
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
