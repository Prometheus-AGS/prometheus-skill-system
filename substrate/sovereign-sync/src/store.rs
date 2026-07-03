use std::path::Path;

use redb::{Database, TableDefinition};

use crate::error::SyncError;

// peers: endpoint_id_hex → json metadata
const PEERS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("peers");
// versions: domain_name → version_vector_json
const VERSIONS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("versions");
// sessions: session_id → session_state_json
const SESSIONS_TABLE: TableDefinition<&str, &str> = TableDefinition::new("sessions");

pub struct SyncStore {
    db: Database,
}

impl SyncStore {
    pub fn open(path: &Path) -> Result<Self, SyncError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SyncError::Storage(format!("Failed to create store dir: {e}")))?;
        }
        let db = Database::create(path)
            .map_err(|e| SyncError::Storage(format!("Failed to open redb: {e}")))?;
        // Initialize tables
        {
            let wtxn = db
                .begin_write()
                .map_err(|e| SyncError::Storage(e.to_string()))?;
            wtxn.open_table(PEERS_TABLE)
                .map_err(|e| SyncError::Storage(e.to_string()))?;
            wtxn.open_table(VERSIONS_TABLE)
                .map_err(|e| SyncError::Storage(e.to_string()))?;
            wtxn.open_table(SESSIONS_TABLE)
                .map_err(|e| SyncError::Storage(e.to_string()))?;
            wtxn.commit()
                .map_err(|e| SyncError::Storage(e.to_string()))?;
        }
        Ok(Self { db })
    }

    pub fn default_path() -> std::path::PathBuf {
        dirs_next::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("sovereign-sync")
            .join("state.redb")
    }

    pub fn upsert_peer(&self, endpoint_id: &str, metadata_json: &str) -> Result<(), SyncError> {
        let wtxn = self
            .db
            .begin_write()
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        {
            let mut table = wtxn
                .open_table(PEERS_TABLE)
                .map_err(|e| SyncError::Storage(e.to_string()))?;
            table
                .insert(endpoint_id, metadata_json)
                .map_err(|e| SyncError::Storage(e.to_string()))?;
        }
        wtxn.commit().map_err(|e| SyncError::Storage(e.to_string()))
    }

    pub fn get_peer(&self, endpoint_id: &str) -> Result<Option<String>, SyncError> {
        let rtxn = self
            .db
            .begin_read()
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        let table = rtxn
            .open_table(PEERS_TABLE)
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        Ok(table
            .get(endpoint_id)
            .map_err(|e| SyncError::Storage(e.to_string()))?
            .map(|v| v.value().to_string()))
    }

    pub fn set_version(&self, domain: &str, vv_json: &str) -> Result<(), SyncError> {
        let wtxn = self
            .db
            .begin_write()
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        {
            let mut table = wtxn
                .open_table(VERSIONS_TABLE)
                .map_err(|e| SyncError::Storage(e.to_string()))?;
            table
                .insert(domain, vv_json)
                .map_err(|e| SyncError::Storage(e.to_string()))?;
        }
        wtxn.commit().map_err(|e| SyncError::Storage(e.to_string()))
    }

    pub fn get_version(&self, domain: &str) -> Result<Option<String>, SyncError> {
        let rtxn = self
            .db
            .begin_read()
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        let table = rtxn
            .open_table(VERSIONS_TABLE)
            .map_err(|e| SyncError::Storage(e.to_string()))?;
        Ok(table
            .get(domain)
            .map_err(|e| SyncError::Storage(e.to_string()))?
            .map(|v| v.value().to_string()))
    }
}
