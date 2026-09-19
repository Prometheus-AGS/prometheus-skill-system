# change-cpc-007-supervisor-and-health

**Title:** Adopt and supervise the pack's services from the Companion with a single health aggregator  
**Repository:** `prometheus-companion`  
**Phase:** control-plane-to-companion  
**Depends on:** `change-cpc-001-integration-contract`, `change-cpc-005-node-hosting`  
**Backend:** native-kbd

## Why

Goal 4 requires unified management, configuration, and health of every pack service. The services stay where they run (D-02); the Companion adopts them through the service manifest from change-cpc-001, using HMA's health-aggregator template and launchagent-supervisor rules, and runs only its own node in-process. Companion spec §16.4 and §25.2 label everything in-process and must be amended (D-05, D-09).

## What Changes

- Port HMA `assets/templates/tauri-tray/health-aggregator/` into `prometheus_substrate::health` (five states, one aggregator).
- Implement the §16 supervisor actor: read the installed pack's `services.manifest.json` via the contract, probe each service, adopt it as `ChildKind::ExternalSidecar`, restart via `launchctl kickstart` / `systemctl --user restart`, backoff 1s→60s, 10 failures → disabled; every retry branch cites a named failure (Companion AGENTS.md §0.2).
- Render the Companion's own launchd plist per HMA rules (ThrottleInterval ≥ 15, KeepAlive dictionary, PID-file lock, one installer).
- Expose `GET /api/v1/services` on the node and drive the tray icon from the aggregator; amend spec §16.4 and §25.2 to "adopted external services plus one in-process node".

## Scope

Files this change may create or edit (tasks.json `files` is the per-task view):

- `crates/prometheus-substrate/src/health.rs`
- `crates/prometheus-substrate/src/services.rs`
- `crates/prometheus-substrate/src/supervisor.rs`
- `crates/prometheus-substrate/tests/supervision.rs`
- `crates/sovereign-sync/src/rest_api.rs`
- `docs/00-architecture-and-implementation-plan.md`
- `scripts/install-companion-service.sh`
- `shared/launchd/ai.prometheus.companion.plist.tmpl`
- `src-tauri/src/tray.rs`

## Capabilities

- `companion-supervision` (new)

## ADDED Requirements

### Requirement: Every manifest service is adopted and reported
WHEN the pack's services are installed, THEN `GET /api/v1/services` lists each manifest entry with a live state from the aggregator.

#### Scenario: Adoption
- **WHEN** the node starts with the pack installed
- **THEN** every label in `services.manifest.json` appears with a state in {Healthy, Degraded, Down, Paused, Starting}

### Requirement: A killed service is restarted within budget
WHEN an adopted service process is killed, THEN the supervisor restarts it through the platform supervisor and the state returns to Healthy within the documented budget.

#### Scenario: Restart
- **WHEN** `kill` is sent to an adopted service's PID
- **THEN** state passes Down then Healthy and the restart is logged with its named failure

### Requirement: Companion is restarted by launchd
WHEN the Companion process dies, THEN launchd restarts it per the rendered plist without crash-loop removal.

#### Scenario: Plist hygiene
- **WHEN** the rendered plist is inspected
- **THEN** it has ThrottleInterval >= 15 and a KeepAlive dictionary

## Cross-repo execution rule (resolves assessment Q3)

The skill-pack KBD run `sovereign-sync-service-reliability-20260829` owns this phase and every change in it. Changes whose Repository is `prometheus-companion` are executed in the Companion checkout, mirrored into the Companion's `openspec/changes/<change-id>/` as `proposal.md` + `tasks.md` for that repository's own history, and their evidence (commands, outputs, commit hashes) is recorded in this change's `verification.md` in the pack. The Companion's runtime records nothing for this phase until its open phase `docs-review-for-build-assessment` is reconciled (assess artifacts exist on disk dated 2026-08-23 while its runtime still reports `ready`); that reconciliation is a task of change-cpc-003. No change mixes repositories: pack-side edits live in pack-side changes.

## Constraints

- Implementation-first, integration-only evidence: no unit tests as delivery evidence; every acceptance criterion below has a command in `verification.md`'s verify block, run after the coherent edit batch, locally only.
- One Cargo build machine-wide at a time; `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Companion AGENTS.md applies: no `unwrap()` in library code, exact pins, iroh in the Infrastructure layer, §0.2 observed problems only, `bash scripts/audit-all.sh` must exit 0 (the scaffold ships the audit scripts, commit 773aa5e).
- New dependencies are a human pin: any crate this change introduces is staged in `docs/decisions/versions-pins-proposal.md` (change-cpc-003) and the change stays BLOCKED until the operator's `versions.toml` edit includes it.
- The pack never depends on the Companion; the Companion consumes pack crates by git rev (D-02).

## Open Questions

- Disposition of the timer services (learning-worker, codex-skills-sync, hooks-logrotate, prometheus-nudge, pk-lint, mem0-compress): adopted as timers or left to launchd.
