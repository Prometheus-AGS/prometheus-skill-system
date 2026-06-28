# change-goal-014: `loop.json` Schema Extension

**Phase:** goal-loop-support
**Status:** pending
**Sub-phase:** B (integration)
**Depends on:** change-goal-003 (must align with goal.json)

## Problem

The existing `loop.json` schema has no concept of `phases[]` or per-phase stopping conditions. Goals with multiple phases cannot be represented as a loop, so `/loop-tick` cannot drive goal phase advancement.

## Solution

Extend `loop-definition.schema.json` with optional `phases[]` and `goal_slug` fields (backward-compatible), update `loop-schema.md`, and ensure `kbd-goal-start.sh` writes both a `goal.json` and a corresponding `loop.json` so `/loop-tick` can drive the outer goal loop.

## Files

- `skills/process/pmpo-outer-loop/references/schemas/loop-definition.schema.json` (UPDATE)
- `skills/process/pmpo-outer-loop/references/loop-schema.md` (UPDATE)

## Tasks

- [ ] Add optional `phases[]` to `loop-definition.schema.json`: array of `{name, type, stopping_condition, human_gate}`
- [ ] Add optional `goal_slug` field: link to `.kbd-orchestrator/goals/<slug>/goal.json`
- [ ] Verify backward compatibility: existing `loop.json` files without new fields still validate
- [ ] Update `loop-schema.md`: document new fields, add examples, cross-reference `goal.schema.json`
- [ ] Update `kbd-goal-start.sh` (goal-002): write `loop.json` to `.kbd-orchestrator/loops/<slug>/` alongside `goal.json`, so `/loop-tick` can drive goal advancement
- [ ] Test: run `jq -e` schema validation on existing loop.json files to confirm no regressions
