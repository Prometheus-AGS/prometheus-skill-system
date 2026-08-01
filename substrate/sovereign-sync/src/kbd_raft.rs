//! Durable OpenRaft storage for authoritative KBD control-plane events.
//!
//! Raft decides the single committed order. redb supplies atomic, crash-safe
//! persistence for the log, vote/commit metadata, state machine, membership,
//! snapshots, idempotent command results, and projection revision.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Debug},
    io::{self, Cursor},
    ops::RangeBounds,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration as MonotonicDuration, Instant, SystemTime, UNIX_EPOCH},
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use kbd_runtime::{
    apply_committed_event, CommandKind, CommandResult, DeviceRecord, DeviceStatus, Event,
    KbdStateV2,
};
use openraft::{
    storage::{Adaptor, LogState, RaftLogReader, RaftSnapshotBuilder, Snapshot},
    CommittedLeaderId, Entry, EntryPayload, LogId, OptionalSend, RaftLogId, RaftStorage,
    RaftTypeConfig, SnapshotMeta, StorageError, StorageIOError, StoredMembership, Vote,
};
use rand_core::{OsRng, RngCore};
use redb::{Database, ReadableTable, TableDefinition};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub type KbdNodeId = u64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuorumStatus {
    pub configured_voters: usize,
    pub available_voters: usize,
    pub quorum_size: usize,
    pub writable: bool,
    pub standalone_non_ha: bool,
    pub automatic_takeover: bool,
    pub reason: String,
}

/// Safety policy applied before any KBD mutation is submitted to Raft.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuorumPolicy {
    node_id: KbdNodeId,
    voters: BTreeSet<KbdNodeId>,
}

/// Per-leader monotonic lease timer. It intentionally cannot recover an
/// elapsed duration across process restart: a new process/leader starts a full
/// TTL grace period, which is the conservative safety behavior.
#[derive(Debug, Clone)]
pub struct LeaseReclaimGate {
    term: u64,
    ttl: MonotonicDuration,
    leader_since: Instant,
    last_committed_renewal: Option<Instant>,
}

impl LeaseReclaimGate {
    pub fn new(term: u64, leader_since: Instant) -> Self {
        Self {
            term,
            ttl: MonotonicDuration::from_secs(kbd_runtime::DEFAULT_LEASE_TTL_SECONDS as u64),
            leader_since,
            last_committed_renewal: None,
        }
    }

    pub fn term(&self) -> u64 {
        self.term
    }

    pub fn observe_leadership(&mut self, term: u64, now: Instant) {
        if term != self.term {
            self.term = term;
            self.leader_since = now;
            self.last_committed_renewal = None;
        }
    }

    pub fn observe_committed_renewal(&mut self, term: u64, now: Instant) {
        self.observe_leadership(term, now);
        self.last_committed_renewal = Some(now);
    }

    pub fn may_reclaim_at(&self, now: Instant) -> bool {
        let safety_origin = self.last_committed_renewal.unwrap_or(self.leader_since);
        now.saturating_duration_since(safety_origin) >= self.ttl
    }

    pub fn require_reclaim_at(&self, now: Instant) -> io::Result<()> {
        if self.may_reclaim_at(now) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "uncertain lease cannot be reclaimed until a full monotonic TTL has elapsed",
            ))
        }
    }
}

impl QuorumPolicy {
    pub fn new(
        node_id: KbdNodeId,
        voters: impl IntoIterator<Item = KbdNodeId>,
    ) -> io::Result<Self> {
        let voters = voters.into_iter().collect::<BTreeSet<_>>();
        if voters.is_empty() || voters.contains(&0) || !voters.contains(&node_id) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "KBD voters must be nonzero and include the local node",
            ));
        }
        Ok(Self { node_id, voters })
    }

    pub fn node_id(&self) -> KbdNodeId {
        self.node_id
    }

    pub fn voters(&self) -> &BTreeSet<KbdNodeId> {
        &self.voters
    }

    pub fn status(&self, available: impl IntoIterator<Item = KbdNodeId>) -> QuorumStatus {
        let available = available
            .into_iter()
            .filter(|node| self.voters.contains(node))
            .collect::<BTreeSet<_>>();
        let configured = self.voters.len();
        let quorum_size = configured / 2 + 1;
        let writable = available.len() >= quorum_size;
        let standalone_non_ha = configured == 1;
        let automatic_takeover = configured >= 3 && writable;
        let reason = if standalone_non_ha {
            "standalone writer; no automatic failover".to_string()
        } else if !writable {
            format!(
                "read-only: {} of {} voters available; {} required",
                available.len(),
                configured,
                quorum_size
            )
        } else if configured == 2 {
            "writable only while both voters are available; automatic takeover disabled".to_string()
        } else {
            "quorum available".to_string()
        };
        QuorumStatus {
            configured_voters: configured,
            available_voters: available.len(),
            quorum_size,
            writable,
            standalone_non_ha,
            automatic_takeover,
            reason,
        }
    }

    pub fn require_write_quorum(
        &self,
        available: impl IntoIterator<Item = KbdNodeId>,
    ) -> io::Result<QuorumStatus> {
        let status = self.status(available);
        if !status.writable {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                status.reason.clone(),
            ));
        }
        Ok(status)
    }

    pub fn require_automatic_takeover(
        &self,
        available: impl IntoIterator<Item = KbdNodeId>,
    ) -> io::Result<QuorumStatus> {
        let status = self.require_write_quorum(available)?;
        if !status.automatic_takeover {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "automatic cross-device takeover requires at least three configured voters",
            ));
        }
        Ok(status)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KbdRaftNode {
    pub endpoint: String,
    pub signer_key_id: String,
    pub witness: bool,
}

openraft::declare_raft_types!(
    pub KbdRaftConfig:
        D = Event,
        R = CommandResult,
        Node = KbdRaftNode,
);

pub type KbdRaftLogStore = Adaptor<KbdRaftConfig, Arc<RedbRaftStore>>;
pub type KbdRaftStateMachine = Adaptor<KbdRaftConfig, Arc<RedbRaftStore>>;

const LOG_TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("kbd_raft_log");
const META_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("kbd_raft_meta");
const STATE_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("kbd_raft_state");
const PROJECTION_TABLE: TableDefinition<&str, u64> = TableDefinition::new("kbd_raft_projection");
const PAIRING_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("kbd_pairing_offers");

const LAST_PURGED_KEY: &str = "last_purged";
const COMMITTED_KEY: &str = "committed";
const VOTE_KEY: &str = "vote";
const STATE_MACHINE_KEY: &str = "state_machine";
const SNAPSHOT_KEY: &str = "snapshot";
const PROJECTION_REVISION_KEY: &str = "revision";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PairingOffer {
    pub token: String,
    pub expires_unix_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PairingRecord {
    expires_unix_seconds: u64,
    consumed: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DurableStateMachine {
    last_applied_log: Option<LogId<KbdNodeId>>,
    last_membership: StoredMembership<KbdNodeId, KbdRaftNode>,
    runtime: KbdStateV2,
    command_results: BTreeMap<String, CommandResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DurableSnapshot {
    meta: SnapshotMeta<KbdNodeId, KbdRaftNode>,
    data: Vec<u8>,
}

pub struct RedbRaftStore {
    db: Database,
    snapshot_sequence: AtomicU64,
}

impl Debug for RedbRaftStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RedbRaftStore")
            .field(
                "snapshot_sequence",
                &self.snapshot_sequence.load(Ordering::Relaxed),
            )
            .finish_non_exhaustive()
    }
}


/// Best-effort: which process holds `path` open?
///
/// Returns a printable suffix, or an empty string when nothing can be
/// determined. Deliberately non-fatal and non-blocking — this runs on an error
/// path, and a diagnostic that can itself hang would make the failure worse.
///
/// `lsof` is the only portable-enough option on macOS/Linux without adding a
/// dependency. Its absence is not an error; the message simply omits the hint.
fn lock_holder_hint(path: &Path) -> String {
    use std::process::{Command, Stdio};

    let output = Command::new("lsof")
        .arg("-t")
        .arg(path)
        .stderr(Stdio::null())
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let pids: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .split_whitespace()
                .map(str::to_string)
                .collect();
            if pids.is_empty() {
                // No holder, yet the open still failed — a stale lock or a
                // permissions problem, and worth saying so rather than
                // implying contention that is not there.
                " (no process appears to hold this file; the lock may be stale \
                  or the path may not be writable)"
                    .to_string()
            } else {
                format!(
                    " (held by pid {}; stop it before starting another daemon)",
                    pids.join(", ")
                )
            }
        }
        _ => String::new(),
    }
}

impl RedbRaftStore {
    pub fn open(path: &Path) -> io::Result<Arc<Self>> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        // A lock failure must say WHO holds it, not just that it failed.
        //
        // `Database::create` returns `Database already open. Cannot acquire
        // lock.` when another process holds the file. Under launchd's
        // `KeepAlive=true` with no `ThrottleInterval`, that error restarts the
        // daemon immediately into the same failure — a tight loop that wrote
        // the identical line thousands of times and told an operator nothing
        // actionable. Observed on this machine 2026-07-28.
        //
        // redb's lock is advisory and released when the holding process exits,
        // so there is nothing safe to "clear": forcibly removing it while a
        // live daemon holds the store risks two writers on one file, which is
        // far worse than a failed start. The fix is therefore DIAGNOSIS, not
        // deletion — name the holder so the operator can act in one step
        // instead of grepping a log full of identical lines.
        let mut db = match Database::create(path) {
            Ok(db) => db,
            Err(e) => {
                let held_by = lock_holder_hint(path);
                return Err(io::Error::other(format!(
                    "cannot open the raft store at {}: {e}{held_by}",
                    path.display()
                )));
            }
        };

        // Reclaim pages freed by `purge_logs_upto`.
        //
        // openraft snapshots and then calls purge, which DELETES log rows — but
        // redb never returns freed pages to the filesystem on its own, so the
        // file only ever grows. Observed on a live daemon: raft.redb reached 44
        // MiB, then 65 MiB within a single working session, while the logical
        // log stayed small. A wedged daemon on that store answered /health in
        // 7.2 s; a fresh process on the same data answered in 1.3 ms.
        //
        // Compaction runs HERE, at open, because it needs `&mut Database` and no
        // live transactions — both true only before the store is shared. It is
        // also the point where a restart already costs startup latency, so the
        // work is invisible.
        //
        // Never fatal. A failure to shrink a file is not a reason to refuse to
        // start: the daemon is still correct, just larger.
        match db.compact() {
            Ok(true) => tracing::info!(path = %path.display(), "raft store compacted"),
            Ok(false) => tracing::debug!("raft store had nothing to compact"),
            Err(error) => tracing::warn!(%error, "raft store compaction skipped"),
        }

        {
            let transaction = db.begin_write().map_err(io_other)?;
            transaction.open_table(LOG_TABLE).map_err(io_other)?;
            transaction.open_table(META_TABLE).map_err(io_other)?;
            transaction.open_table(STATE_TABLE).map_err(io_other)?;
            transaction.open_table(PROJECTION_TABLE).map_err(io_other)?;
            transaction.open_table(PAIRING_TABLE).map_err(io_other)?;
            transaction.commit().map_err(io_other)?;
        }
        let store = Arc::new(Self {
            db,
            snapshot_sequence: AtomicU64::new(0),
        });
        if let Some(snapshot) = store.read_json::<DurableSnapshot>(STATE_TABLE, SNAPSHOT_KEY)? {
            let suffix = snapshot
                .meta
                .snapshot_id
                .rsplit('-')
                .next()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(0);
            store.snapshot_sequence.store(suffix, Ordering::Relaxed);
        }
        Ok(store)
    }

    pub fn into_openraft_stores(self: &Arc<Self>) -> (KbdRaftLogStore, KbdRaftStateMachine) {
        Adaptor::new(self.clone())
    }

    pub fn runtime_state(&self) -> io::Result<KbdStateV2> {
        Ok(self.state_machine()?.runtime)
    }

    pub fn command_result(&self, command_id: &str) -> io::Result<Option<CommandResult>> {
        Ok(self
            .state_machine()?
            .command_results
            .get(command_id)
            .cloned())
    }

    pub fn projection_revision(&self) -> io::Result<u64> {
        let transaction = self.db.begin_read().map_err(io_other)?;
        let table = transaction.open_table(PROJECTION_TABLE).map_err(io_other)?;
        Ok(table
            .get(PROJECTION_REVISION_KEY)
            .map_err(io_other)?
            .map(|value| value.value())
            .unwrap_or(0))
    }

    /// Commit one event atomically in explicitly configured one-voter mode.
    ///
    /// Multi-voter deployments must use `Raft::client_write`; this method
    /// exists so standalone mode has the same redb log/state/idempotency path
    /// without pretending it provides high availability.
    pub fn commit_standalone(
        &self,
        policy: &QuorumPolicy,
        event: Event,
    ) -> io::Result<CommandResult> {
        if policy.voters().len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "local commit is restricted to explicit one-voter mode",
            ));
        }
        policy.require_write_quorum([policy.node_id()])?;
        let command_id = event
            .command_id
            .clone()
            .unwrap_or_else(|| event.event_id.clone());
        let transaction = self.db.begin_write().map_err(io_other)?;
        let mut state = {
            let table = transaction.open_table(STATE_TABLE).map_err(io_other)?;
            let decoded = match table.get(STATE_MACHINE_KEY).map_err(io_other)? {
                Some(value) => serde_json::from_slice::<DurableStateMachine>(value.value())
                    .map_err(io_other)?,
                None => DurableStateMachine::default(),
            };
            decoded
        };
        if let Some(original) = state.command_results.get(&command_id) {
            let mut duplicate = original.clone();
            duplicate.duplicate = true;
            return Ok(duplicate);
        }
        let index = state
            .last_applied_log
            .map(|log_id| log_id.index.saturating_add(1))
            .unwrap_or(1);
        let log_id = LogId::new(CommittedLeaderId::new(1, policy.node_id()), index);
        let entry: Entry<KbdRaftConfig> = Entry {
            log_id,
            payload: EntryPayload::Normal(event.clone()),
        };
        apply_committed_event(&mut state.runtime, &event).map_err(io_other)?;
        state.last_applied_log = Some(log_id);
        let response = CommandResult {
            command_id: command_id.clone(),
            committed_revision: state.runtime.revision,
            duplicate: false,
            state: state.runtime.clone(),
            apply_error: None,
        };
        state.command_results.insert(command_id, response.clone());

        let entry_bytes = serde_json::to_vec(&entry).map_err(io_other)?;
        let log_id_bytes = serde_json::to_vec(&log_id).map_err(io_other)?;
        let state_bytes = serde_json::to_vec(&state).map_err(io_other)?;
        {
            let mut log = transaction.open_table(LOG_TABLE).map_err(io_other)?;
            log.insert(index, entry_bytes.as_slice())
                .map_err(io_other)?;
            let mut meta = transaction.open_table(META_TABLE).map_err(io_other)?;
            meta.insert(COMMITTED_KEY, log_id_bytes.as_slice())
                .map_err(io_other)?;
            let mut state_table = transaction.open_table(STATE_TABLE).map_err(io_other)?;
            state_table
                .insert(STATE_MACHINE_KEY, state_bytes.as_slice())
                .map_err(io_other)?;
            let mut projection = transaction.open_table(PROJECTION_TABLE).map_err(io_other)?;
            projection
                .insert(PROJECTION_REVISION_KEY, state.runtime.revision)
                .map_err(io_other)?;
        }
        transaction.commit().map_err(io_other)?;
        Ok(response)
    }

    pub fn committed_events(&self, since_revision: u64) -> io::Result<Vec<Event>> {
        let state = self.state_machine()?;
        let Some(last_applied) = state.last_applied_log else {
            return Ok(Vec::new());
        };
        let transaction = self.db.begin_read().map_err(io_other)?;
        let table = transaction.open_table(LOG_TABLE).map_err(io_other)?;
        let mut events = Vec::new();
        for row in table.range(0..=last_applied.index).map_err(io_other)? {
            let (_, value) = row.map_err(io_other)?;
            let entry: Entry<KbdRaftConfig> =
                serde_json::from_slice(value.value()).map_err(io_other)?;
            if let EntryPayload::Normal(event) = entry.payload {
                if event.revision >= since_revision {
                    events.push(event);
                }
            }
        }
        Ok(events)
    }

    /// Create a local, one-use pairing secret. Consuming it only produces the
    /// canonical enrollment command; trust is not granted until that command is
    /// committed by Raft.
    pub fn create_pairing_offer(&self, ttl: MonotonicDuration) -> io::Result<PairingOffer> {
        if ttl.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "pairing TTL must be greater than zero",
            ));
        }
        let mut secret = [0_u8; 32];
        OsRng.fill_bytes(&mut secret);
        let token = URL_SAFE_NO_PAD.encode(secret);
        let token_hash = blake3::hash(token.as_bytes()).to_hex().to_string();
        let expires_unix_seconds = unix_seconds()?.saturating_add(ttl.as_secs());
        let bytes = serde_json::to_vec(&PairingRecord {
            expires_unix_seconds,
            consumed: false,
        })
        .map_err(io_other)?;
        let transaction = self.db.begin_write().map_err(io_other)?;
        {
            let mut table = transaction.open_table(PAIRING_TABLE).map_err(io_other)?;
            table
                .insert(token_hash.as_str(), bytes.as_slice())
                .map_err(io_other)?;
        }
        transaction.commit().map_err(io_other)?;
        Ok(PairingOffer {
            token,
            expires_unix_seconds,
        })
    }

    pub fn consume_pairing_offer(
        &self,
        token: &str,
        device: DeviceRecord,
    ) -> io::Result<CommandKind> {
        let token_hash = blake3::hash(token.as_bytes()).to_hex().to_string();
        let transaction = self.db.begin_write().map_err(io_other)?;
        {
            let mut table = transaction.open_table(PAIRING_TABLE).map_err(io_other)?;
            let value = table
                .get(token_hash.as_str())
                .map_err(io_other)?
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::PermissionDenied, "unknown pairing token")
                })?;
            let mut record: PairingRecord =
                serde_json::from_slice(value.value()).map_err(io_other)?;
            drop(value);
            if record.consumed {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "pairing token was already consumed",
                ));
            }
            if unix_seconds()? > record.expires_unix_seconds {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "pairing token expired",
                ));
            }
            record.consumed = true;
            let bytes = serde_json::to_vec(&record).map_err(io_other)?;
            table
                .insert(token_hash.as_str(), bytes.as_slice())
                .map_err(io_other)?;
        }
        transaction.commit().map_err(io_other)?;
        Ok(CommandKind::DeviceEnroll { device })
    }

    /// Both Raft membership and the committed device registry must authorize a
    /// peer. A topic name, operator string, or endpoint alone never grants
    /// observation access.
    pub fn is_peer_authorized(&self, endpoint: &str, signer_key_id: &str) -> io::Result<bool> {
        let state = self.state_machine()?;
        let device_is_active = state
            .runtime
            .devices
            .get(signer_key_id)
            .is_some_and(|device| device.status == DeviceStatus::Active);
        if !device_is_active {
            return Ok(false);
        }
        let membership_authorizes = state
            .last_membership
            .nodes()
            .any(|(_, node)| node.endpoint == endpoint && node.signer_key_id == signer_key_id);
        Ok(membership_authorizes)
    }

    fn state_machine(&self) -> io::Result<DurableStateMachine> {
        Ok(self
            .read_json(STATE_TABLE, STATE_MACHINE_KEY)?
            .unwrap_or_default())
    }

    fn read_json<T: DeserializeOwned>(
        &self,
        table_definition: TableDefinition<&str, &[u8]>,
        key: &str,
    ) -> io::Result<Option<T>> {
        let transaction = self.db.begin_read().map_err(io_other)?;
        let table = transaction.open_table(table_definition).map_err(io_other)?;
        let Some(value) = table.get(key).map_err(io_other)? else {
            return Ok(None);
        };
        serde_json::from_slice(value.value())
            .map(Some)
            .map_err(io_other)
    }

    fn write_meta<T: Serialize>(&self, key: &str, value: &T) -> io::Result<()> {
        let bytes = serde_json::to_vec(value).map_err(io_other)?;
        let transaction = self.db.begin_write().map_err(io_other)?;
        {
            let mut table = transaction.open_table(META_TABLE).map_err(io_other)?;
            table.insert(key, bytes.as_slice()).map_err(io_other)?;
        }
        transaction.commit().map_err(io_other)
    }

    fn read_meta<T: DeserializeOwned>(&self, key: &str) -> io::Result<Option<T>> {
        self.read_json(META_TABLE, key)
    }

    fn persist_state_machine(&self, state: &DurableStateMachine) -> io::Result<()> {
        let bytes = serde_json::to_vec(state).map_err(io_other)?;
        let transaction = self.db.begin_write().map_err(io_other)?;
        {
            let mut state_table = transaction.open_table(STATE_TABLE).map_err(io_other)?;
            state_table
                .insert(STATE_MACHINE_KEY, bytes.as_slice())
                .map_err(io_other)?;
            let mut projection_table =
                transaction.open_table(PROJECTION_TABLE).map_err(io_other)?;
            projection_table
                .insert(PROJECTION_REVISION_KEY, state.runtime.revision)
                .map_err(io_other)?;
        }
        transaction.commit().map_err(io_other)
    }
}

impl RaftLogReader<KbdRaftConfig> for Arc<RedbRaftStore> {
    #[allow(clippy::result_large_err)]
    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + OptionalSend>(
        &mut self,
        range: RB,
    ) -> Result<Vec<Entry<KbdRaftConfig>>, StorageError<KbdNodeId>> {
        let transaction = self.db.begin_read().map_err(storage_read)?;
        let table = transaction.open_table(LOG_TABLE).map_err(storage_read)?;
        let entries = table
            .range(range)
            .map_err(storage_read)?
            .map(|entry| {
                let (_, value) = entry.map_err(storage_read)?;
                serde_json::from_slice(value.value()).map_err(storage_read)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(entries)
    }
}

impl RaftSnapshotBuilder<KbdRaftConfig> for Arc<RedbRaftStore> {
    async fn build_snapshot(&mut self) -> Result<Snapshot<KbdRaftConfig>, StorageError<KbdNodeId>> {
        let state = self.state_machine().map_err(storage_read_state_machine)?;
        let data = serde_json::to_vec(&state).map_err(storage_read_state_machine)?;
        let sequence = self.snapshot_sequence.fetch_add(1, Ordering::SeqCst) + 1;
        let snapshot_id = state
            .last_applied_log
            .map(|log| format!("{}-{}-{sequence}", log.leader_id, log.index))
            .unwrap_or_else(|| format!("empty-{sequence}"));
        let meta = SnapshotMeta {
            last_log_id: state.last_applied_log,
            last_membership: state.last_membership,
            snapshot_id,
        };
        let durable = DurableSnapshot {
            meta: meta.clone(),
            data: data.clone(),
        };
        let snapshot_bytes = serde_json::to_vec(&durable).map_err(storage_write_snapshot)?;
        let transaction = self.db.begin_write().map_err(storage_write_snapshot)?;
        {
            let mut table = transaction
                .open_table(STATE_TABLE)
                .map_err(storage_write_snapshot)?;
            table
                .insert(SNAPSHOT_KEY, snapshot_bytes.as_slice())
                .map_err(storage_write_snapshot)?;
        }
        transaction.commit().map_err(storage_write_snapshot)?;
        Ok(Snapshot {
            meta,
            snapshot: Box::new(Cursor::new(data)),
        })
    }
}

impl RaftStorage<KbdRaftConfig> for Arc<RedbRaftStore> {
    type LogReader = Self;
    type SnapshotBuilder = Self;

    async fn get_log_state(&mut self) -> Result<LogState<KbdRaftConfig>, StorageError<KbdNodeId>> {
        let transaction = self.db.begin_read().map_err(storage_read)?;
        let table = transaction.open_table(LOG_TABLE).map_err(storage_read)?;
        let last_log_id = match table.last().map_err(storage_read)? {
            Some((_, value)) => {
                let entry: Entry<KbdRaftConfig> =
                    serde_json::from_slice(value.value()).map_err(storage_read)?;
                Some(*entry.get_log_id())
            }
            None => None,
        };
        let last_purged_log_id = self
            .read_meta(LAST_PURGED_KEY)
            .map_err(storage_read)?
            .flatten();
        Ok(LogState {
            last_purged_log_id,
            last_log_id: last_log_id.or(last_purged_log_id),
        })
    }

    async fn save_vote(&mut self, vote: &Vote<KbdNodeId>) -> Result<(), StorageError<KbdNodeId>> {
        self.write_meta(VOTE_KEY, vote).map_err(storage_write)
    }

    async fn read_vote(&mut self) -> Result<Option<Vote<KbdNodeId>>, StorageError<KbdNodeId>> {
        self.read_meta(VOTE_KEY).map_err(storage_read)
    }

    async fn save_committed(
        &mut self,
        committed: Option<LogId<KbdNodeId>>,
    ) -> Result<(), StorageError<KbdNodeId>> {
        self.write_meta(COMMITTED_KEY, &committed)
            .map_err(storage_write)
    }

    async fn read_committed(
        &mut self,
    ) -> Result<Option<LogId<KbdNodeId>>, StorageError<KbdNodeId>> {
        Ok(self
            .read_meta(COMMITTED_KEY)
            .map_err(storage_read)?
            .flatten())
    }

    async fn last_applied_state(
        &mut self,
    ) -> Result<
        (
            Option<LogId<KbdNodeId>>,
            StoredMembership<KbdNodeId, KbdRaftNode>,
        ),
        StorageError<KbdNodeId>,
    > {
        let state = self.state_machine().map_err(storage_read_state_machine)?;
        Ok((state.last_applied_log, state.last_membership))
    }

    #[allow(clippy::result_large_err)]
    async fn delete_conflict_logs_since(
        &mut self,
        log_id: LogId<KbdNodeId>,
    ) -> Result<(), StorageError<KbdNodeId>> {
        let transaction = self.db.begin_write().map_err(storage_write)?;
        {
            let mut table = transaction.open_table(LOG_TABLE).map_err(storage_write)?;
            let keys = table
                .range(log_id.index..)
                .map_err(storage_write)?
                .map(|entry| entry.map(|(key, _)| key.value()).map_err(storage_write))
                .collect::<Result<Vec<_>, _>>()?;
            for key in keys {
                table.remove(key).map_err(storage_write)?;
            }
        }
        transaction.commit().map_err(storage_write)
    }

    #[allow(clippy::result_large_err)]
    async fn purge_logs_upto(
        &mut self,
        log_id: LogId<KbdNodeId>,
    ) -> Result<(), StorageError<KbdNodeId>> {
        let encoded = serde_json::to_vec(&Some(log_id)).map_err(storage_write)?;
        let transaction = self.db.begin_write().map_err(storage_write)?;
        {
            let mut log_table = transaction.open_table(LOG_TABLE).map_err(storage_write)?;
            let keys = log_table
                .range(..=log_id.index)
                .map_err(storage_write)?
                .map(|entry| entry.map(|(key, _)| key.value()).map_err(storage_write))
                .collect::<Result<Vec<_>, _>>()?;
            for key in keys {
                log_table.remove(key).map_err(storage_write)?;
            }
            let mut meta_table = transaction.open_table(META_TABLE).map_err(storage_write)?;
            meta_table
                .insert(LAST_PURGED_KEY, encoded.as_slice())
                .map_err(storage_write)?;
        }
        transaction.commit().map_err(storage_write)
    }

    async fn append_to_log<I>(&mut self, entries: I) -> Result<(), StorageError<KbdNodeId>>
    where
        I: IntoIterator<Item = Entry<KbdRaftConfig>> + OptionalSend,
    {
        let entries = entries.into_iter().collect::<Vec<_>>();
        let transaction = self.db.begin_write().map_err(storage_write)?;
        {
            let mut table = transaction.open_table(LOG_TABLE).map_err(storage_write)?;
            for entry in entries {
                let bytes = serde_json::to_vec(&entry)
                    .map_err(|error| StorageIOError::write_log_entry(entry.log_id, &error))?;
                table
                    .insert(entry.log_id.index, bytes.as_slice())
                    .map_err(|error| StorageIOError::write_log_entry(entry.log_id, &error))?;
            }
        }
        transaction.commit().map_err(storage_write)
    }

    async fn apply_to_state_machine(
        &mut self,
        entries: &[Entry<KbdRaftConfig>],
    ) -> Result<Vec<CommandResult>, StorageError<KbdNodeId>> {
        let mut state = self.state_machine().map_err(storage_read_state_machine)?;
        let mut responses = Vec::with_capacity(entries.len());
        for entry in entries {
            state.last_applied_log = Some(entry.log_id);
            match &entry.payload {
                EntryPayload::Blank => responses.push(noop_result(&state.runtime, entry.log_id)),
                EntryPayload::Membership(membership) => {
                    state.last_membership =
                        StoredMembership::new(Some(entry.log_id), membership.clone());
                    responses.push(noop_result(&state.runtime, entry.log_id));
                }
                EntryPayload::Normal(event) => {
                    let command_id = event
                        .command_id
                        .as_deref()
                        .unwrap_or(event.event_id.as_str());
                    if let Some(original) = state.command_results.get(command_id) {
                        let mut duplicate = original.clone();
                        duplicate.duplicate = true;
                        responses.push(duplicate);
                        continue;
                    }
                    // A business-logic apply failure (e.g. an invalid work-item
                    // transition) must NOT abort the whole batch via `?`: that
                    // would skip `persist_state_machine` below, which means
                    // `last_applied_log` never advances past this entry — on
                    // every subsequent write (and on replay after a restart)
                    // the same permanently-invalid entry gets retried and
                    // fails again, forever, blocking all later commands too.
                    // `KbdStateV2::apply` never mutates state before an early
                    // `Err` return, so `state.runtime` is safe to reuse as-is
                    // for the rest of this loop and any later entries.
                    let response = match apply_committed_event(&mut state.runtime, event) {
                        Ok(()) => CommandResult {
                            command_id: command_id.to_string(),
                            committed_revision: state.runtime.revision,
                            duplicate: false,
                            state: state.runtime.clone(),
                            apply_error: None,
                        },
                        Err(error) => {
                            tracing::warn!(
                                command_id,
                                log_index = entry.log_id.index,
                                error = %error,
                                "command failed business-logic validation — recorded as failed, log position still advances"
                            );
                            CommandResult {
                                command_id: command_id.to_string(),
                                committed_revision: state.runtime.revision,
                                duplicate: false,
                                state: state.runtime.clone(),
                                apply_error: Some(error.to_string()),
                            }
                        }
                    };
                    state
                        .command_results
                        .insert(command_id.to_string(), response.clone());
                    responses.push(response);
                }
            }
        }
        self.persist_state_machine(&state)
            .map_err(storage_write_state_machine)?;
        Ok(responses)
    }

    async fn begin_receiving_snapshot(
        &mut self,
    ) -> Result<Box<<KbdRaftConfig as RaftTypeConfig>::SnapshotData>, StorageError<KbdNodeId>> {
        Ok(Box::new(Cursor::new(Vec::new())))
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMeta<KbdNodeId, KbdRaftNode>,
        snapshot: Box<<KbdRaftConfig as RaftTypeConfig>::SnapshotData>,
    ) -> Result<(), StorageError<KbdNodeId>> {
        let bytes = snapshot.into_inner();
        let state: DurableStateMachine =
            serde_json::from_slice(&bytes).map_err(storage_read_snapshot)?;
        let durable = DurableSnapshot {
            meta: meta.clone(),
            data: bytes,
        };
        let state_bytes = serde_json::to_vec(&state).map_err(storage_write_snapshot)?;
        let snapshot_bytes = serde_json::to_vec(&durable).map_err(storage_write_snapshot)?;
        let transaction = self.db.begin_write().map_err(storage_write_snapshot)?;
        {
            let mut table = transaction
                .open_table(STATE_TABLE)
                .map_err(storage_write_snapshot)?;
            table
                .insert(STATE_MACHINE_KEY, state_bytes.as_slice())
                .map_err(storage_write_snapshot)?;
            table
                .insert(SNAPSHOT_KEY, snapshot_bytes.as_slice())
                .map_err(storage_write_snapshot)?;
            let mut projection_table = transaction
                .open_table(PROJECTION_TABLE)
                .map_err(storage_write_snapshot)?;
            projection_table
                .insert(PROJECTION_REVISION_KEY, state.runtime.revision)
                .map_err(storage_write_snapshot)?;
        }
        transaction.commit().map_err(storage_write_snapshot)
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<Snapshot<KbdRaftConfig>>, StorageError<KbdNodeId>> {
        let snapshot = self
            .read_json::<DurableSnapshot>(STATE_TABLE, SNAPSHOT_KEY)
            .map_err(storage_read_snapshot)?;
        Ok(snapshot.map(|snapshot| Snapshot {
            meta: snapshot.meta,
            snapshot: Box::new(Cursor::new(snapshot.data)),
        }))
    }

    async fn get_log_reader(&mut self) -> Self::LogReader {
        self.clone()
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        self.clone()
    }
}

fn noop_result(state: &KbdStateV2, log_id: LogId<KbdNodeId>) -> CommandResult {
    CommandResult {
        command_id: format!("raft-system:{}:{}", log_id.leader_id, log_id.index),
        committed_revision: state.revision,
        duplicate: false,
        state: state.clone(),
        apply_error: None,
    }
}

fn io_other(error: impl fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}

fn unix_seconds() -> io::Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(io_other)
}

fn storage_read(error: impl fmt::Display) -> StorageError<KbdNodeId> {
    StorageIOError::read(&io_other(error)).into()
}

fn storage_write(error: impl fmt::Display) -> StorageError<KbdNodeId> {
    StorageIOError::write(&io_other(error)).into()
}

fn storage_read_state_machine(error: impl fmt::Display) -> StorageError<KbdNodeId> {
    StorageIOError::read_state_machine(&io_other(error)).into()
}

fn storage_write_state_machine(error: impl fmt::Display) -> StorageError<KbdNodeId> {
    StorageIOError::write_state_machine(&io_other(error)).into()
}

fn storage_read_snapshot(error: impl fmt::Display) -> StorageError<KbdNodeId> {
    StorageIOError::read_snapshot(None, &io_other(error)).into()
}

fn storage_write_snapshot(error: impl fmt::Display) -> StorageError<KbdNodeId> {
    StorageIOError::write_snapshot(None, &io_other(error)).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kbd_runtime::{Actor, DeviceSigner, Runtime};
    use openraft::{CommittedLeaderId, EntryPayload, Membership};
    use tempfile::tempdir;

    #[test]
    fn quorum_policy_is_safe_for_one_two_and_three_voters() {
        let standalone = QuorumPolicy::new(1, [1]).unwrap();
        let status = standalone.status([1]);
        assert!(status.writable);
        assert!(status.standalone_non_ha);
        assert!(!status.automatic_takeover);

        let two = QuorumPolicy::new(1, [1, 2]).unwrap();
        assert!(!two.status([1]).writable);
        assert!(two.status([1, 2]).writable);
        assert!(two.require_automatic_takeover([1, 2]).is_err());

        let three = QuorumPolicy::new(1, [1, 2, 3]).unwrap();
        assert!(!three.status([1]).writable);
        assert!(three.status([1, 2]).writable);
        assert!(three.status([1, 2]).automatic_takeover);
    }

    #[test]
    fn leader_reclaim_waits_full_monotonic_ttl_after_election_or_renewal() {
        let elected = Instant::now();
        let mut gate = LeaseReclaimGate::new(7, elected);
        let ttl = MonotonicDuration::from_secs(kbd_runtime::DEFAULT_LEASE_TTL_SECONDS as u64);
        assert!(!gate.may_reclaim_at(elected + ttl - MonotonicDuration::from_millis(1)));
        assert!(gate.may_reclaim_at(elected + ttl));

        let renewal = elected + MonotonicDuration::from_secs(30);
        gate.observe_committed_renewal(7, renewal);
        assert!(!gate.may_reclaim_at(elected + ttl));
        assert!(gate.may_reclaim_at(renewal + ttl));

        let re_elected = renewal + MonotonicDuration::from_secs(10);
        gate.observe_leadership(8, re_elected);
        assert!(!gate.may_reclaim_at(renewal + ttl));
        assert!(gate.may_reclaim_at(re_elected + ttl));
    }

    #[test]
    fn pairing_is_one_use_and_allowlist_requires_membership_and_active_device() {
        let directory = tempdir().unwrap();
        let store = RedbRaftStore::open(&directory.path().join("raft.redb")).unwrap();
        let signer = DeviceSigner::generate();
        let device = DeviceRecord {
            device_id: "device-b".into(),
            key_id: signer.key_id().into(),
            public_key: signer.public_key().into(),
            status: DeviceStatus::Active,
            enrolled_at_revision: 2,
            revoked_at_revision: None,
        };
        let offer = store
            .create_pairing_offer(MonotonicDuration::from_secs(60))
            .unwrap();
        assert!(matches!(
            store
                .consume_pairing_offer(&offer.token, device.clone())
                .unwrap(),
            CommandKind::DeviceEnroll { .. }
        ));
        assert!(store
            .consume_pairing_offer(&offer.token, device.clone())
            .is_err());
        assert!(!store
            .is_peer_authorized("endpoint-b", signer.key_id())
            .unwrap());

        let mut durable = DurableStateMachine::default();
        durable
            .runtime
            .devices
            .insert(device.key_id.clone(), device.clone());
        durable.last_membership = StoredMembership::new(
            None,
            Membership::new(
                vec![BTreeSet::from([2])],
                BTreeMap::from([(
                    2,
                    KbdRaftNode {
                        endpoint: "endpoint-b".into(),
                        signer_key_id: device.key_id.clone(),
                        witness: false,
                    },
                )]),
            ),
        );
        store.persist_state_machine(&durable).unwrap();
        assert!(store
            .is_peer_authorized("endpoint-b", signer.key_id())
            .unwrap());

        durable
            .runtime
            .devices
            .get_mut(signer.key_id())
            .unwrap()
            .status = DeviceStatus::Revoked;
        store.persist_state_machine(&durable).unwrap();
        assert!(!store
            .is_peer_authorized("endpoint-b", signer.key_id())
            .unwrap());
    }

    fn project_event(project_root: &Path) -> Event {
        let runtime = Runtime::open(project_root);
        runtime
            .initialize("project-a", "run-a", Actor::operator("device-a", "test"))
            .unwrap();
        runtime.events().unwrap().remove(0)
    }

    #[tokio::test]
    async fn redb_store_recovers_committed_state_and_idempotency() {
        let directory = tempdir().unwrap();
        let store = RedbRaftStore::open(&directory.path().join("raft.redb")).unwrap();
        let mut adapted = store.clone();
        let event = project_event(&directory.path().join("event-source"));
        let command_id = event.command_id.clone().unwrap();
        let entry = Entry {
            log_id: LogId::new(CommittedLeaderId::new(1, 1), 1),
            payload: EntryPayload::Normal(event.clone()),
        };
        let responses = adapted
            .apply_to_state_machine(std::slice::from_ref(&entry))
            .await
            .unwrap();
        assert_eq!(responses[0].committed_revision, 1);
        assert_eq!(store.projection_revision().unwrap(), 1);

        let duplicate = Entry {
            log_id: LogId::new(CommittedLeaderId::new(1, 1), 2),
            payload: EntryPayload::Normal(event),
        };
        let responses = adapted.apply_to_state_machine(&[duplicate]).await.unwrap();
        assert!(responses[0].duplicate);
        assert_eq!(responses[0].committed_revision, 1);

        drop(adapted);
        drop(store);
        let reopened = RedbRaftStore::open(&directory.path().join("raft.redb")).unwrap();
        assert_eq!(reopened.runtime_state().unwrap().revision, 1);
        assert!(reopened.command_result(&command_id).unwrap().is_some());
    }

    /// Regression test for a real production incident: a committed entry
    /// that fails `KbdStateV2::apply` (e.g. an invalid work-item or
    /// lifecycle transition — durable log commit and business-logic
    /// validation are separate Raft concerns) must not permanently wedge
    /// the state machine. Before this fix, `apply_to_state_machine` used
    /// `?` on the per-entry apply error, which skipped `persist_state_machine`
    /// entirely — so `last_applied_log` never advanced past the bad entry,
    /// and every later write (plus a fresh replay on daemon restart) kept
    /// retrying and re-failing on that same entry forever.
    #[tokio::test]
    async fn a_failed_apply_does_not_block_later_entries_or_restart_replay() {
        let directory = tempdir().unwrap();
        let runtime = Runtime::open(directory.path().join("event-source"));
        runtime
            .initialize("project-a", "run-a", Actor::operator("device-a", "test"))
            .unwrap();
        let genesis = runtime.events().unwrap().remove(0);
        let after_genesis = kbd_runtime::replay_events(std::slice::from_ref(&genesis)).unwrap();

        // Ready -> Ready is not a valid lifecycle transition (see
        // `valid_transition`) — a signed, well-formed event that will fail
        // `KbdStateV2::apply` at the semantic-validation step, exactly like
        // a real invalid work-item transition would.
        let invalid_event = runtime
            .prepare_signed_command(
                &after_genesis,
                kbd_runtime::CommandEnvelope {
                    schema_version: "1".into(),
                    project_id: after_genesis.project_id.clone(),
                    run_id: after_genesis.run_id.clone(),
                    command_id: "invalid-transition".into(),
                    expected_revision: after_genesis.revision,
                    actor: Actor::operator("device-a", "test"),
                    lease_id: None,
                    fencing_token: None,
                    command: kbd_runtime::CommandKind::LifecycleTransition {
                        to: kbd_runtime::LifecycleState::Ready,
                        reason: "deliberately invalid for this test".into(),
                    },
                },
            )
            .unwrap();

        // A genuinely valid follow-up event, prepared from the SAME
        // pre-invalid-entry state (Ready) — exactly what state.runtime looks
        // like when apply_to_state_machine reaches this entry, since a
        // failed apply never mutates state.
        let valid_event = runtime
            .prepare_signed_command(
                &after_genesis,
                kbd_runtime::CommandEnvelope {
                    schema_version: "1".into(),
                    project_id: after_genesis.project_id.clone(),
                    run_id: after_genesis.run_id.clone(),
                    command_id: "valid-run".into(),
                    expected_revision: after_genesis.revision,
                    actor: Actor::operator("device-a", "test"),
                    lease_id: None,
                    fencing_token: None,
                    command: kbd_runtime::CommandKind::LifecycleTransition {
                        to: kbd_runtime::LifecycleState::Running,
                        reason: "valid transition after the poisoned entry".into(),
                    },
                },
            )
            .unwrap();

        let store = RedbRaftStore::open(&directory.path().join("raft.redb")).unwrap();
        let mut adapted = store.clone();

        let genesis_entry = Entry {
            log_id: LogId::new(CommittedLeaderId::new(1, 1), 1),
            payload: EntryPayload::Normal(genesis),
        };
        adapted
            .apply_to_state_machine(std::slice::from_ref(&genesis_entry))
            .await
            .unwrap();

        // The invalid entry is applied ALONE, mirroring a single live write
        // whose command fails validation.
        let invalid_entry = Entry {
            log_id: LogId::new(CommittedLeaderId::new(1, 1), 2),
            payload: EntryPayload::Normal(invalid_event),
        };
        let invalid_responses = adapted
            .apply_to_state_machine(std::slice::from_ref(&invalid_entry))
            .await
            .expect("a per-entry apply failure must not fail the whole batch call");
        assert!(
            invalid_responses[0].apply_error.is_some(),
            "the invalid transition must be recorded as a failure"
        );
        assert_eq!(
            invalid_responses[0].committed_revision, 1,
            "a failed apply must not advance the runtime revision"
        );

        // Simulate a daemon restart: reopen the store fresh and confirm the
        // log position already advanced past the poisoned entry, so a cold
        // replay never has to re-attempt (and re-fail on) it again.
        drop(adapted);
        drop(store);
        let reopened = RedbRaftStore::open(&directory.path().join("raft.redb")).unwrap();
        let reopened_state = reopened.state_machine().unwrap();
        assert_eq!(
            reopened_state.last_applied_log.unwrap().index,
            2,
            "log position must advance past a failed entry, not get stuck behind it"
        );
        assert_eq!(reopened_state.runtime.revision, 1);

        // A later, genuinely valid entry — applied in its own separate call
        // by the freshly-reopened store, exactly as a real live write after
        // a restart would be — must still succeed.
        let mut reopened_adapter = reopened.clone();
        let valid_entry = Entry {
            log_id: LogId::new(CommittedLeaderId::new(1, 1), 3),
            payload: EntryPayload::Normal(valid_event),
        };
        let valid_responses = reopened_adapter
            .apply_to_state_machine(std::slice::from_ref(&valid_entry))
            .await
            .unwrap();
        assert!(valid_responses[0].apply_error.is_none());
        assert_eq!(valid_responses[0].committed_revision, 2);
        assert_eq!(reopened.projection_revision().unwrap(), 2);
    }

    #[tokio::test]
    async fn snapshots_round_trip_state_machine_and_projection_revision() {
        let directory = tempdir().unwrap();
        let store = RedbRaftStore::open(&directory.path().join("raft.redb")).unwrap();
        let mut adapted = store.clone();
        let entry = Entry {
            log_id: LogId::new(CommittedLeaderId::new(1, 1), 1),
            payload: EntryPayload::Normal(project_event(&directory.path().join("event-source"))),
        };
        adapted.apply_to_state_machine(&[entry]).await.unwrap();
        let snapshot = adapted.build_snapshot().await.unwrap();

        let replacement = RedbRaftStore::open(&directory.path().join("replacement.redb")).unwrap();
        let mut replacement_adapter = replacement.clone();
        replacement_adapter
            .install_snapshot(&snapshot.meta, snapshot.snapshot)
            .await
            .unwrap();
        assert_eq!(replacement.runtime_state().unwrap().revision, 1);
        assert_eq!(replacement.projection_revision().unwrap(), 1);
    }

    #[tokio::test]
    async fn lease_grants_heartbeats_and_takeovers_apply_only_as_committed_events() {
        let directory = tempdir().unwrap();
        let source = Runtime::open(directory.path().join("event-source"));
        let first_actor = Actor::operator("operator-a", "codex");
        let initialized = source
            .initialize("project-a", "run-a", first_actor.clone())
            .unwrap();
        let claimed = source
            .claim(
                first_actor.clone(),
                initialized.revision,
                "project/phase",
                false,
            )
            .unwrap();
        let first_lease = claimed.lease.clone().unwrap();
        let heartbeat = source
            .heartbeat(
                first_actor,
                claimed.revision,
                first_lease.lease_id,
                first_lease.fencing_token,
            )
            .unwrap();
        let second_actor = Actor::operator("operator-b", "claude");
        source
            .claim(second_actor, heartbeat.revision, "project/phase", true)
            .unwrap();

        let store = RedbRaftStore::open(&directory.path().join("raft.redb")).unwrap();
        let mut adapted = store.clone();
        let entries = source
            .events()
            .unwrap()
            .into_iter()
            .map(|event| Entry {
                log_id: LogId::new(CommittedLeaderId::new(1, 1), event.revision),
                payload: EntryPayload::Normal(event),
            })
            .collect::<Vec<_>>();
        adapted.apply_to_state_machine(&entries).await.unwrap();

        let state = store.runtime_state().unwrap();
        let lease = state.lease.unwrap();
        assert_eq!(lease.fencing_token, 2);
        assert_eq!(state.last_fencing_token, 2);
        assert_eq!(
            (lease.expires_at - lease.heartbeat_at).num_seconds(),
            kbd_runtime::DEFAULT_LEASE_TTL_SECONDS
        );
        assert_eq!(kbd_runtime::DEFAULT_HEARTBEAT_SECONDS, 30);
    }
}
