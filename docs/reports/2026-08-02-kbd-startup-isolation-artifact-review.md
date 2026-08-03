# KBD Startup Isolation Artifact Review

**Date:** 2026-08-02

**Scope:** Committed source, manifests, lockfiles, service templates, installer tests, and the local commit series from `b29602c` through the startup-isolation changes.

**Method:** Static inspection only. KBD, its skill, the installed daemon, launchd bootstrap, and live runtime data were not used.

## Verdict

**PASS.** The reviewed artifacts preserve the journal/Loro authority, remove startup coupling between HTTP and production networking, and contain no active OpenRaft, lease-ownership, Raft-database, or focus-path control behavior. The only remaining `raft.redb` references are preservation/migration code that renames existing files to archives, records checksums, proves rollback, and never opens them as an authority database.

## Evidence

| Gate | Result | Evidence |
|---|---|---|
| OpenRaft removal | PASS | Case-insensitive scans of the KBD runtime, sync daemon/client, mobile crate, and Prometheus CLI found no `openraft` or `kbd_raft` source, manifest, or lockfile dependency. |
| Obsolete ownership API | PASS | Rust and manifest scans found no `Lease`, `lease_id`, `fencing_token`, or heartbeat ownership API. CRDT claims remain the distinct supported claim model. |
| Raft database use | PASS | No code opens a Raft database. `raft.redb` appears only in `control-plane-recover.rs` and `live_migration_proof.rs`, where it is copied/hashed/renamed to `raft.redb.archive` and rollback instructions are generated. |
| Focus-path behavior | PASS | No `KBD_FOCUS_PROJECT_PATH` or `focus_project_path` behavior remains. Hook fallbacks now require the declared `.prometheus/project.json` identity instead of an active/focus path. Installer tests retain only negative checks that reject the removed option. |
| Redb restriction | PASS | The active Sovereign Sync dependency graph contains only direct `redb 2.6.3`, used by `src/store.rs` at `state.redb`. `redb 4.1.0` is absent after disabling the unused `storage-provider` iroh-docs default feature in Sovereign Sync and learner-model dependencies. |
| N0 preservation | PASS | Production uses `Endpoint::builder(presets::N0)`. `presets::Minimal` is confined to the deterministic in-memory lookup test constructor. No relay-clearing call exists. |
| HTTP/P2P isolation | PASS | Axum and N0 each own a named, dedicated two-worker Tokio runtime. HTTP communicates with P2P only through bounded commands, events, replies, and synchronous status snapshots. |
| Readiness semantics | PASS | Static `/health` reads no state. Startup `/ready` reports typed diagnostic progress. Full `/ready` checks only local authority, enforces per-project and aggregate bounds, and includes P2P as informational data. |
| Service policy | PASS | The canonical launchd template renders `ProcessType=Interactive`, `KeepAlive=true`, and `ThrottleInterval=10`; the render-only installer test asserts all three. |

## Commands used

```text
rg -i 'openraft|kbd_raft' <reviewed crates and tools>
rg 'KBD_FOCUS_PROJECT_PATH|focus_project_path' <reviewed crates, shared scripts, installers>
rg '\b(Lease|lease_id|fencing_token|heartbeat)\b' <reviewed Rust and manifests>
rg 'Database::|state\.redb|raft\.redb' <reviewed crates and tools>
cargo tree --manifest-path substrate/sovereign-sync/Cargo.toml -i redb@2.6.3
cargo tree --manifest-path substrate/sovereign-sync/Cargo.toml -i redb@4.1.0
bash shared/scripts/tests/test-position-render.sh
bash shared/scripts/tests/test-kbd-registry-service-install.sh
plutil -lint shared/launchagents/ai.prometheus.sovereign-sync.plist
```

The `redb@4.1.0` inverse dependency query correctly returned “package ID specification did not match any packages”; the `redb@2.6.3` query resolved only to Sovereign Sync.

## Reviewed local commits

- `6b64366` — single-pass authority startup and protected registration
- `0276007` — dedicated HTTP startup runtime and diagnostic router
- `82c4811` — isolated/supervised production P2P runtime
- `62fa24a` — aggregate readiness deadline
- `4c7b817` — warm health latency sampler and budget exits
- `ca78e51` — launchd Interactive policy
- `6911c4e` — 18-authority and slow-initializer regression fixtures

## Deployment boundary

This review certifies artifacts only. It does not certify the installed binary, signatures, runtime registry, live N0 connectivity, launchd timings, SIGKILL recovery, or latency thresholds. Those remain gated behind the full lower-tier test matrix, the single release build, backups/checksums, installation, and live acceptance. KBD itself remains prohibited until those gates pass.
