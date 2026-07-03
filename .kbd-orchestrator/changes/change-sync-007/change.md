# change-sync-007: redb persistence for sync state

**Phase:** phase-learn-sovereign-sync
**Tier:** 1 (parallelize with 004, 005, 006 after 003)
**Status:** pending
**Library:** cand-005 (redb 4.1.0)
**Gap:** G-02

## Summary

Implement crash-safe embedded persistence for P2P node state using redb 4.1.0.
Replaces Temporal.io (rejected). Stores peer metadata, Loro version vectors,
and sync session state.

## Files to change

- `substrate/sovereign-sync/src/store.rs` — new file
- `substrate/sovereign-sync/Cargo.toml` — add redb = "4.1"

## Tables

```rust
// peer table: EndpointId → (last_seen_ts, capabilities_json)
const PEERS_TABLE: TableDefinition<[u8; 32], &[u8]> = TableDefinition::new("peers");

// version table: domain_key (string) → Loro version vector (bytes)
const VERSIONS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("versions");

// session table: session_id (uuid string) → sync_session_state (json)
const SESSIONS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("sessions");
```

DB path: `~/.local/share/sovereign-sync/state.redb`

## Tasks

- [ ] Write SyncStore struct wrapping redb::Database
- [ ] Implement CRUD for all three tables
- [ ] Implement atomic transaction pattern for session updates
- [ ] Test: write peer, restart, read peer (crash-safe)
