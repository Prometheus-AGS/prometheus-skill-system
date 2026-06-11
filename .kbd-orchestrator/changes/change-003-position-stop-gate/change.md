---
id: change-003-position-stop-gate
title: Stop-gate position footer enforcement
phase: position-and-handoff-guarantee
gaps: [F1]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: done
scope:
  - shared/scripts/position-stop-gate.sh
  - hooks/hooks.json
  - shared/scripts/tests/test-position-stop-gate.sh
---

# change-003 — Stop-gate position footer enforcement

## Context

Injection (change-002) makes the model see position; this gate makes omission
self-correcting: if the final assistant message lacks the position footer, block
the stop once with the rendered footer as the continuation reason. Honest
ceiling: one enforced retry, never an infinite loop.

## Scope

In:

- New `shared/scripts/position-stop-gate.sh`:
  - Reads stdin JSON (`stop_hook_active`, `transcript_path`, `session_id`).
  - Exit 0 fast when: `stop_hook_active` true; no `.kbd-orchestrator`; waypoint
    status terminal (`phase_complete`/`reflect_complete`); transcript missing.
  - Extracts last assistant text from the JSONL transcript via python3.
  - If text lacks `prometheus-position` sentinel AND `Position:` literal →
    emit `{"decision":"block","reason":"<instruction + rendered footer>"}`.
  - Soft cap: `~/.prometheus/position-stop-block.txt` records
    `<session_id>:<transcript mtime+size hash>`; matching entry → exit 0.
  - All failure paths exit 0 (never break the Stop chain accidentally).
- `hooks/hooks.json`: insert as FIRST command in the Stop group, no `|| true`,
  timeout 5000.
- New `shared/scripts/tests/test-position-stop-gate.sh`: fixture transcript
  without footer → block JSON; with footer → exit 0 silent;
  `stop_hook_active:true` → exit 0; soft-cap second call → exit 0.

## Tasks

- [x] 1. Write `shared/scripts/position-stop-gate.sh` (+x)
- [x] 2. Wire FIRST into hooks.json Stop group
- [x] 3. Write test; run green

## Verification

Test green; hooks.json parses; existing Stop hooks unchanged in order after the gate.
