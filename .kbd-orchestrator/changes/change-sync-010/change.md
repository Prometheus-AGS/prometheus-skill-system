# change-sync-010: REST API (Axum routes, daemon/server modes)

**Phase:** phase-learn-sovereign-sync
**Tier:** 2 (parallelize with 008, 009, 011 after Tier 1)
**Status:** pending
**Library:** cand-009 (axum 0.8.x)
**Gap:** G-02, G-NEW-1

## Summary

Implement REST API routes for daemon and server modes. This is the interface
used by the Tauri sidecar, Axum harnesses, and the sovereign-client Rust SDK.

## Files to change

- `substrate/sovereign-sync/src/routes/mod.rs` — route composition
- `substrate/sovereign-sync/src/routes/sync.rs` — sync endpoints
- `substrate/sovereign-sync/src/routes/skills.rs` — SkillIndex endpoints
- `substrate/sovereign-sync/src/routes/health.rs` — health check

## Endpoints

```
GET  /health
GET  /api/v1/skills               → Vec<SkillEntry>
GET  /api/v1/skills/search?q=...  → Vec<SkillEntry> (keyword search)
POST /api/v1/sync/push            → SyncResult
POST /api/v1/sync/pull            → SyncResult
GET  /api/v1/sync/status          → SyncStatus
GET  /api/v1/sync/peers           → Vec<PeerInfo>
```

## Port (default: 7892)

Tauri sidecar connects to localhost:7892. No TLS in local mode.

## Tasks

- [ ] Implement Axum router with all routes
- [ ] Wire AppState (config, skill index, p2p node handle)
- [ ] Add JSON error responses (consistent envelope)
- [ ] Wire into main.rs --mode daemon and --mode server branches
- [ ] Test: curl /health returns 200
- [ ] Test: GET /api/v1/skills returns populated list from skills_dir
