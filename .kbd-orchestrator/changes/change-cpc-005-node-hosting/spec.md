# change-cpc-005-node-hosting

**Title:** Host the node in-process from the Tauri setup hook with a headless preset and state-location continuity  
**Repository:** `prometheus-companion`  
**Phase:** control-plane-to-companion  
**Depends on:** `change-cpc-004-relocate-sovereign-sync`  
**Backend:** native-kbd

## Why

No reference example hosts an axum server inside a Tauri process; clawdesk is the only precedent. The node must start from the setup hook, expose the socket surface for external callers, run headless without a window or vault unlock, resolve the same data root as the launchd daemon, and survive the keychain ACL boundary between two signed binaries (D-03, D-08, D-06).

## What Changes

- Add `prometheus_substrate::node` that builds the sovereign-sync `AppState` and serves the axum router on the Unix socket from `tauri::async_runtime::spawn` inside `setup`, with the startup router (`/health`, `/ready`) bound first and state via `app.manage`.
- Add the headless entry point (`prometheus-companion-headless`, feature `headless`) that runs the same node with no window and no Stronghold vault.
- Paths: the node resolves `PROMETHEUS_DATA_DIR` then `dirs_next::data_local_dir()`; never Tauri's path resolver for shared state. Socket path per contract v1.
- Credential store: on first run detect whether `prometheus-kbd-device` is readable; GUI path surfaces the keychain ACL prompt; headless path requires `PROMETHEUS_DEVICE_KEY_FILE` and fails closed with the documented message if absent.
- Amend spec §15 (Tauri events for the UI, SSE external), §4.1 (process model and MCP list), and allocate the socket in the spec.

## Scope

Files this change may create or edit (tasks.json `files` is the per-task view):

- `crates/prometheus-substrate/src/bin/prometheus-companion-headless.rs`
- `crates/prometheus-substrate/src/identity.rs`
- `crates/prometheus-substrate/src/node.rs`
- `crates/prometheus-substrate/src/paths.rs`
- `crates/prometheus-substrate/tests/continuity.rs`
- `docs/00-architecture-and-implementation-plan.md`
- `src-tauri/src/lib.rs`

## Capabilities

- `companion-node-hosting` (new)

## ADDED Requirements

### Requirement: Node serves the socket surface from inside the Tauri process
WHEN the Companion (desktop or headless) starts, THEN `/ready` on the contract socket answers within the startup budget and the pack's `prometheus kbd status --json` reads through it.

#### Scenario: Headless start
- **WHEN** the headless binary starts on a fresh data root with `PROMETHEUS_DEVICE_KEY_FILE` set
- **THEN** `/ready` is 200 and a signed typed mutation via the CLI commits

#### Scenario: Headless without key file
- **WHEN** the headless binary starts without a key file and no readable keychain item
- **THEN** it exits non-zero with the documented message and writes nothing

### Requirement: State written by the daemon is read by the Companion-hosted node
WHEN a data root was populated by the pack's daemon at revision N, THEN the Companion-hosted node reports revision N and can append N+1 with the same device identity or a re-enrolled one.

#### Scenario: Continuity
- **WHEN** the two-process continuity test runs
- **THEN** journal revision, frontier, the signed audit export (`signed_audit_jsonl`), the receipt set, and the run history match before the append, and after it the revision advances by one with every prior receipt and run still present

## Cross-repo execution rule (resolves assessment Q3)

The skill-pack KBD run `sovereign-sync-service-reliability-20260829` owns this phase and every change in it. Changes whose Repository is `prometheus-companion` are executed in the Companion checkout, mirrored into the Companion's `openspec/changes/<change-id>/` as `proposal.md` + `tasks.md` for that repository's own history, and their evidence (commands, outputs, commit hashes) is recorded in this change's `verification.md` in the pack. The Companion's runtime records nothing for this phase until its open phase `docs-review-for-build-assessment` is reconciled (assess artifacts exist on disk dated 2026-08-23 while its runtime still reports `ready`); that reconciliation is a task of change-cpc-003. No change mixes repositories: pack-side edits live in pack-side changes.

## Constraints

- Implementation-first, integration-only evidence: no unit tests as delivery evidence; every acceptance criterion below has a command in `verification.md`'s verify block, run after the coherent edit batch, locally only.
- One Cargo build machine-wide at a time; `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Companion AGENTS.md applies: no `unwrap()` in library code, exact pins, iroh in the Infrastructure layer, §0.2 observed problems only, `bash scripts/audit-all.sh` must exit 0 (the scaffold ships the audit scripts, commit 773aa5e).
- New dependencies are a human pin: any crate this change introduces is staged in `docs/decisions/versions-pins-proposal.md` (change-cpc-003) and the change stays BLOCKED until the operator's `versions.toml` edit includes it.
- The pack never depends on the Companion; the Companion consumes pack crates by git rev (D-02).

## Open Questions

- Whether the GUI-hosted node should offer one-time re-enrollment instead of relying on the keychain ACL prompt (UX decision for the Companion's own phase).
