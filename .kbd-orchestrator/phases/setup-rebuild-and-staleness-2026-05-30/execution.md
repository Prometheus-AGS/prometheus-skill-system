# Execution: setup-rebuild-and-staleness-2026-05-30

**Backend selected**: `openspec` (executed natively in Claude Code)
**Executor**: claude-code (claude-opus-4-8)
**Dispatch**: direct — all 6 changes touch one crate (`tools/prometheus-cli`);
no cross-tool handoff. KBD remains the source of truth via progress.json.

## Why native dispatch

All source edits are scoped to `tools/prometheus-cli/crates/prometheus-cli/src/commands/setup.rs`
and `main.rs`. No parallelizable source-spec work that would benefit from Roo/Cursor handoff.

## Execution order (strict)

1. change-staleness-001-add-stale-component-status
2. change-staleness-002-add-staleness-comparator         (depends 001)
3. change-staleness-003-wire-staleness-into-detection    (depends 002)
4. change-staleness-004-add-rebuild-flag-and-installers  (depends 003)
5. change-staleness-005-improve-check-output-grouping    (depends 001)
6. change-staleness-006-verify-and-integration-test      (depends 001-005)

## QA gate disposition

| Change | Files | QA |
|--------|-------|-----|
| 001 | setup.rs (1 file but core enum) | opt-in QA |
| 002 | setup.rs (1 file) | opt-in QA per prior reflection's "blast radius > file count" lesson |
| 003 | setup.rs (1 file) | opt-in QA |
| 004 | setup.rs + main.rs (2 files; major behavior change) | **QA required** |
| 005 | setup.rs (1 file) | opt-in QA |
| 006 | (verification only) | skip |

## Verification gate at end

Synthetic stale → `--rebuild` → re-check → 0 stale; all unit tests green; clippy clean.
