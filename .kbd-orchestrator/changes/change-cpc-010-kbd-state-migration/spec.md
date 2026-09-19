# change-cpc-010-kbd-state-migration

**Title:** Migrate the unmarked phase projections and prove signed state continuity  
**Repository:** `prometheus-skill-pack`  
**Phase:** control-plane-to-companion  
**Depends on:** none  
**Backend:** native-kbd

## Why

Seven phase `progress.json` files lack the `generatedBy: kbd-runtime` marker and trigger a refusal on every projection (kbd-runtime `lib.rs:5210`). Goal 5 requires existing signed state (journal, `project.loro`, receipts, run history, frontier) to migrate intact; the adopt-or-archive decision per file is unmade.

## What Changes

- Add an `adopt-or-archive` migration in kbd-runtime that, per unmarked projection, either adopts it as a runtime-owned projection when its content matches canonical state or archives it under `.kbd-orchestrator/archives/` with a receipt; expose it as `prometheus kbd migrate --projections` with `--dry-run`.
- Run it against the seven files and record the receipt; assert frontier and revision unchanged by the migration and that the refusal loop no longer fires; add the process-level test case.

## Scope

Files this change may create or edit (tasks.json `files` is the per-task view):

- `.kbd-orchestrator/archives/`
- `.kbd-orchestrator/phases/ideation-and-decision-tools/progress.json`
- `.kbd-orchestrator/phases/kimi-desktop-extensibility/progress.json`
- `.kbd-orchestrator/phases/mobile-skill-portability/progress.json`
- `.kbd-orchestrator/phases/openspec-mirror-drift-cleanup/progress.json`
- `.kbd-orchestrator/phases/prometheus-exec-code-execution-engine/progress.json`
- `.kbd-orchestrator/phases/uar-frontend-workspace-repair/progress.json`
- `.kbd-orchestrator/phases/uar-host-execution/progress.json`
- `substrate/kbd-runtime/src/lib.rs`
- `substrate/kbd-runtime/src/live_migration_proof.rs`
- `tools/prometheus-cli/crates/prometheus-cli/src/commands/kbd.rs`
- `tools/prometheus-cli/crates/prometheus-cli/tests/kbd.rs`

## Capabilities

- `kbd-projection-migration` (new)

## ADDED Requirements

### Requirement: Unmarked projections are adopted or archived with receipts
WHEN the migration runs, THEN every unmarked `progress.json` is either adopted (marker added, content verified against canonical state) or archived with a receipt, and no canonical event is created.

#### Scenario: Seven files
- **WHEN** the migration runs on this repo
- **THEN** seven receipts exist and `prometheus kbd status --json` shows the same revision and frontier as before

### Requirement: Refusal loop is quiet afterwards
WHEN any phase transition runs after migration, THEN no refusal line is emitted.

#### Scenario: Transition
- **WHEN** a typed stage transition runs
- **THEN** stderr contains no `refusing to overwrite` line

## Constraints

- Implementation-first, integration-only evidence: no unit tests as delivery evidence; every acceptance criterion below has a command in `verification.md`'s verify block, run after the coherent edit batch, locally only.
- One Cargo build machine-wide at a time; `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Constraints C-01..C-05 apply; `npm run validate:codex` and `docs/codex-plugin.md` in the same change when plugin surfaces or install flow move; `shared/services.manifest.json` regenerated in the same change when a plist or unit changes.
- The pack never depends on the Companion; the Companion consumes pack crates by git rev (D-02).
