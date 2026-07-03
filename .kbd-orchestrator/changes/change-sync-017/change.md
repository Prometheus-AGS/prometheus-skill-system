# change-sync-017: Integration tests

**Phase:** phase-learn-sovereign-sync
**Tier:** 4 (after Tier 3)
**Status:** pending
**Gap:** G-12

## Summary

Write integration tests covering the 8 key behavioral invariants: Loro
migration, privacy enforcement, iroh gossip discovery, MCP server, AG-UI
streaming, SkillIndex search, UAR passthrough mode, and BossFang prefix
collision avoidance.

## Files to change

- `substrate/sovereign-sync/tests/integration/` — new directory
  - `loro_migration_test.rs`
  - `privacy_gate_test.rs`
  - `iroh_gossip_test.rs`
  - `mcp_server_test.rs`
  - `ag_ui_stream_test.rs`
  - `skill_index_test.rs`
  - `uar_passthrough_test.rs`
  - `prefix_tools_test.rs`
- `substrate/storage-provider/tests/loro_migration_test.rs`

## Test coverage targets

1. Loro migration: old automerge bytes → new Loro doc (migration path)
2. Privacy enforcement: `LocalOnly` domain returns error at sync boundary
3. iroh gossip: two-node topic subscription + peer discovery
4. MCP server: rmcp stdio client connects, lists tools, calls `sync_status`
5. AG-UI stream: POST /sovereign/agent/run → RUN_STARTED → RUN_FINISHED
6. SkillIndex: keyword search on "sync" returns sync skills
7. UAR passthrough: `UAR_SKILL_SERVICE_URL` set → only sync tools exposed
8. BossFang collision avoidance: `--prefix-tools` → all tools prefixed "sovereign:"

## Tasks

- [ ] Write all 8 integration tests
- [ ] Ensure each test is independent (no shared state)
- [ ] Run `cargo test` in substrate workspace — all pass
