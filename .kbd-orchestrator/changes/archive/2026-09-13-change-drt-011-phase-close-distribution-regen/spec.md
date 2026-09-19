# change-drt-011-phase-close-distribution-regen

**Title:** C-01 reconciliation — regenerate the distribution for this child's skill edits
**Repository:** `prometheus-skill-pack`
**Phase:** deep-research-onyx-parity › stage-10-hang-investigation
**Goal:** certification precondition for G3 (named as the C-01 reconciliation owner by change-drt-008's and change-drt-009's Constraints)
**Depends on:** `change-drt-008-hang-capture-harness`, `change-drt-009-stage10-hang-fix-and-certification` — every `skills/**` edit this change mirrors must land first
**Backend:** native-kbd

## Why

C-01 (generated artifacts in sync) is violated in the working tree: `dist/plugins/` still carries the old heredoc `export-package.sh` and lacks every file this child adds under `skills/research/deep-research/tests/hang/` and (conditionally) `skills/process/adversarial-review/`. The deferral contract recorded in drt-008/009 names THIS change as the reconciliation owner, owed before any phase commit.

## What Changes

- Run the distribution generator **twice**; the two outputs must be byte-identical (determinism proof).
- `npm run check:distribution`, `npm run validate:codex`, and `npm run check:skills-index` all PASS locally.
- Hashes, dates, and check results recorded in `verification.md` Evidence.

## Scope

- `dist/plugins/**` (regenerated outputs only; no source edits). The whole-tree generator also syncs earlier sibling changes' accumulated deferrals — the accounting in verification.md makes that explicit.

## Capabilities

- `research-pipeline-execution (distribution reconciliation)`

## ADDED Requirements

### Requirement: The distribution is deterministic and in sync
Two consecutive generator runs SHALL produce byte-identical trees, and every distribution check SHALL pass locally.

#### Scenario: Regeneration proven
- **WHEN** the generator runs twice
- **THEN** the recorded hashes are identical and all three checks PASS, with dates and hashes in verification.md

## Constraints

- Local-only validation; no hosted CI.
- No source edits: this change only regenerates and records.
- A check that cannot run is recorded BLOCKED with the reason.

## Open Questions

- None. If a check fails, the failure is a finding against whichever source edit caused the drift, not a reason to widen this change.
