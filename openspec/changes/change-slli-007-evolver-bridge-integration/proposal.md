# change-slli-007-evolver-bridge-integration

**Phase**: self-learning-loop-integration
**Status**: DONE
**Priority**: 8 of 10
**Gaps closed**: BRIDGE-1, BRIDGE-2

## Summary

Wire `evolver-bridge.json` bidirectional handoff into `iterative-evolver` (writes `execution_results` after each change) and `kbd-reflect` (reads bridge and reports per-item completion back to evolver `state.json`). Documents the bridge schema that was missing from the codebase.

## Files Modified

### `skills/process/iterative-evolver/SKILL.md`

Add section: **Evolver Bridge — Write-back**

When `.kbd-orchestrator/phases/<current>/evolver-bridge.json` exists:
- After each change completes, append to `execution_results`:
  ```json
  {
    "change_id": "change-slli-XXX",
    "evolver_item_id": "evolver-item-N",
    "status": "completed|skipped|failed",
    "completed_at": "ISO8601"
  }
  ```

### `skills/process/kbd-process-orchestrator/SKILL.md`

Add section: **Evolver Bridge — Reflect Read-back**

When `/kbd-reflect` runs and `evolver-bridge.json` exists:
1. Read `item_to_change_map` and `execution_results`
2. For each evolver item, compute status from mapped changes
3. Write status back to `.evolver/evolutions/<name>/state.json` under `kbd_results`
4. If all items are completed: set evolver `current_iteration` status to `ready_for_reflect`

## Files Created

### `openspec/changes/change-slli-007-evolver-bridge-integration/bridge-schema.md`

Canonical documentation of the `evolver-bridge.json` schema (was previously only described in code comments).

## Acceptance Criteria

- After `/kbd-execute` with an evolver bridge present, `execution_results` array in bridge has an entry for each completed change
- After `/kbd-reflect`, the evolver's `state.json` shows `kbd_results` updated
- `/evolve status <name>` shows accurate per-item completion derived from KBD phase
- When no bridge file exists, all existing behavior is unchanged
