use serde::{Deserialize, Serialize};

/// Sync domain status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub node_state: String,
    pub peers: Vec<PeerInfo>,
}

/// Connected peer info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: String,
    pub addr: Option<String>,
}

/// A skill search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResult {
    pub name: String,
    pub description: String,
}

/// AG-UI SSE event emitted by the stream endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgUiEvent {
    TaskAccepted {
        task_id: String,
    },
    Progress {
        task_id: String,
        message: String,
        percent: u8,
    },
    Done {
        task_id: String,
        result: serde_json::Value,
    },
    Error {
        task_id: String,
        error: String,
    },
    EventAppended {
        project_id: String,
        event_id: String,
        replica_id: String,
        lamport: u64,
        frontier: kbd_runtime::CausalFrontier,
    },
    ClaimAcquired {
        project_id: String,
        claim: kbd_runtime::ClaimRecord,
    },
    ClaimConflict {
        project_id: String,
        conflict: kbd_runtime::ConflictRecord,
    },
    SingletonViolation {
        project_id: String,
        conflict: kbd_runtime::ConflictRecord,
    },
    Ping,
}
