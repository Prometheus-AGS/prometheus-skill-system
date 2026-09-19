# change-cpc-003-companion-workspace

**Title:** Create the Companion root workspace and substrate crate consuming pack crates by git rev  
**Repository:** `prometheus-companion`  
**Phase:** control-plane-to-companion  
**Depends on:** `change-cpc-002-skill-ffi-kbd-mobile-split`  
**Backend:** native-kbd

## Why

The Companion is a 14-line Tauri scaffold with no `crates/`, an empty `versions.toml`, and a spec (Appendix A) that assumes a skill-pack superworkspace that does not exist. The scaffold ships its audit gate: `git ls-tree 773aa5e scripts/` in the Companion lists `audit-a11y.sh audit-all.sh audit-deps.sh audit-generated.sh audit-layer.sh audit-naming.sh audit-progress.sh audit-secret-leak.sh audit-skill-desc.sh audit-tokens.sh audit-ui.sh`, and `bash scripts/audit-all.sh` returned PASS 7 / FAIL 0 / SKIP 3 on 2026-09-02 (recorded in this phase's assessment). Every Companion change uses it as its gate; task 6 re-records that listing as execution evidence. Every later Companion change needs a workspace, a substrate crate with desktop and headless presets, and pinned git-rev dependencies on the pack's open crates at a commit that already contains change-cpc-002 (D-03, D-04).

## What Changes

- Add a root `Cargo.toml` workspace with members `crates/prometheus-substrate` and `src-tauri` (HMA `scaffold-tauri-tray.sh` shape; graph-explorer precedent).
- Scaffold `crates/prometheus-substrate` with features `desktop` and `headless` and git-rev dependencies on `kbd-runtime`, `storage-provider` (default-features off), `learner-model`, and `skill-index` from `Prometheus-AGS/prometheus-skill-system` at the commit that lands change-cpc-002.
- Stage the pins the operator must add to `versions.toml` (a human edit; agents are denied) in `docs/decisions/versions-pins-proposal.md`: iroh 1.0.3, iroh-gossip 0.101, iroh-mdns-address-lookup 0.5.0, iroh-mainline-address-lookup 0.5.0, loro 1.13, keyring =3.6.3 (inherited), dirs_next, axum 0.8, rmcp 1.8, redb 2, tauri-plugin-stronghold 2.3.2, and any dependency a later Companion change introduces (each such change stays BLOCKED until its pins land). The change stays BLOCKED until the operator's edit lands.
- Rewrite spec Appendix A for git-rev consumption; initialize `openspec/` in the Companion, mirror this change there, and reconcile the Companion's open phase `docs-review-for-build-assessment` (record its on-disk assess transition in its runtime) so the Companion runtime is consistent before any mirrored change lands.

## Scope

Files this change may create or edit (tasks.json `files` is the per-task view):

- `.kbd-orchestrator/phases/docs-review-for-build-assessment/progress.json`
- `Cargo.toml`
- `crates/prometheus-substrate/Cargo.toml`
- `crates/prometheus-substrate/src/lib.rs`
- `docs/00-architecture-and-implementation-plan.md`
- `docs/decisions/versions-pins-proposal.md`
- `openspec/changes/change-cpc-003-companion-workspace/proposal.md`
- `openspec/config.yaml`
- `src-tauri/Cargo.toml`

## Capabilities

- `companion-workspace` (new)

## ADDED Requirements

### Requirement: Workspace builds the substrate crate in both presets
`cargo check -p prometheus-substrate --features headless` and `--features desktop` SHALL each succeed as a narrowly targeted diagnostic; `bash scripts/audit-all.sh` SHALL exit 0.

#### Scenario: Headless preset
- **WHEN** the headless preset is checked
- **THEN** no tauri dependency is selected

#### Scenario: Pins present
- **WHEN** `versions.toml` `[pins]` is populated by the operator
- **THEN** `bash scripts/audit-deps.sh` passes with no wildcard or caret ranges

### Requirement: Pack crates are consumed by git rev only
The Companion SHALL depend on pack crates by `git` + `rev`; no `path` dependency into the pack and no committed `[patch]` to local checkouts.

#### Scenario: Committed manifest
- **WHEN** the workspace manifests are inspected
- **THEN** every pack crate dependency carries a `rev` and no `[patch]` section targets a local path

### Requirement: Companion runtime is reconciled before mirroring
WHEN the Companion's runtime status is read after this change, THEN phase `docs-review-for-build-assessment` shows its assess stage recorded, matching the artifacts on disk.

#### Scenario: Reconciled
- **WHEN** `prometheus kbd --path <companion> status --json` runs
- **THEN** the assess stage for that phase is complete, not `ready`

## Cross-repo execution rule (resolves assessment Q3)

The skill-pack KBD run `sovereign-sync-service-reliability-20260829` owns this phase and every change in it. Changes whose Repository is `prometheus-companion` are executed in the Companion checkout, mirrored into the Companion's `openspec/changes/<change-id>/` as `proposal.md` + `tasks.md` for that repository's own history, and their evidence (commands, outputs, commit hashes) is recorded in this change's `verification.md` in the pack. The Companion's runtime records nothing for this phase until its open phase `docs-review-for-build-assessment` is reconciled (assess artifacts exist on disk dated 2026-08-23 while its runtime still reports `ready`); that reconciliation is a task of change-cpc-003. No change mixes repositories: pack-side edits live in pack-side changes.

## Constraints

- Implementation-first, integration-only evidence: no unit tests as delivery evidence; every acceptance criterion below has a command in `verification.md`'s verify block, run after the coherent edit batch, locally only.
- One Cargo build machine-wide at a time; `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Companion AGENTS.md applies: no `unwrap()` in library code, exact pins, iroh in the Infrastructure layer, §0.2 observed problems only, `bash scripts/audit-all.sh` must exit 0 (the scaffold ships the audit scripts, commit 773aa5e).
- The pack never depends on the Companion; the Companion consumes pack crates by git rev (D-02).

## Open Questions

- Exact pack commit to pin: the one that lands change-cpc-002.
