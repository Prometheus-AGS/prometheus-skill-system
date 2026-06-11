---
id: change-003-sycophancy-artifact-gate
title: Generalize sycophancy gate to reflection/assessment artifacts
phase: safeguards
gaps: [H3]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: done
scope:
  - shared/scripts/lib/sycophancy.sh
  - shared/scripts/sycophancy-check-reflection.sh
  - shared/scripts/sycophancy-check-artifact.sh
  - shared/scripts/pipeline-enforce.sh
  - hooks/hooks.json
  - shared/scripts/tests/test-sycophancy-artifact.sh
---

# change-003 — Sycophancy gate generalization

## Context

The sycophancy gate fires only for the `reflector` SubagentStop. Main-loop
`kbd-reflect`/assessment artifacts (written via Write, not a subagent) are
ungated. PostToolUse cannot un-write, so teeth come from a progress.json flag +
a pipeline-enforce block at the next lifecycle boundary.

## Scope

In:

- New `shared/scripts/lib/sycophancy.sh` — extract from
  sycophancy-check-reflection.sh: `_find_mcp_bin`, strictness mapping,
  `_mcp_analyze` (JSON-RPC), score/pattern parsing, rejection-counter helpers
  (generalize the counter to a per-artifact key). Graceful skip when binary
  absent.
- `shared/scripts/sycophancy-check-reflection.sh` → thin wrapper sourcing the
  lib; SubagentStop behavior UNCHANGED.
- New `shared/scripts/sycophancy-check-artifact.sh` (PostToolUse Write|Edit,
  path-filtered to `**/reflection.md` and `**/assessment.md`): analyze the
  written file; on reject → exit 2 with Delta/Root-Cause/Corrective-Actions
  feedback AND set `reflect_gate:"rejected"` in the phase progress.json; on pass
  → clear the flag. Same 2-rejection soft cap; binary-absent → exit 0.
- `shared/scripts/pipeline-enforce.sh`: extend the matcher to
  `kbd-new-phase|kbd-next-phase` and block (exit 2) when the active phase's
  progress.json has `reflect_gate == "rejected"` — converting the unblockable
  PostToolUse into a hard gate at the next boundary.
- Wire sycophancy-check-artifact.sh into the PostToolUse group.
- New `shared/scripts/tests/test-sycophancy-artifact.sh`: with binary absent →
  graceful exit 0; flag set/clear logic via a stubbed analyzer; pipeline-enforce
  blocks kbd-new-phase while reflect_gate rejected, passes when cleared.

## Tasks

- [x] 1. Extract shared/scripts/lib/sycophancy.sh; make check-reflection a thin wrapper
- [x] 2. Write sycophancy-check-artifact.sh (flag set/clear)
- [x] 3. Extend pipeline-enforce.sh with the reflect_gate rule
- [x] 4. Wire into hooks.json; write test; run green

## Verification

Test green; reflector SubagentStop gate behaves identically (regression);
pipeline-enforce blocks kbd-new-phase while reflect_gate rejected.
