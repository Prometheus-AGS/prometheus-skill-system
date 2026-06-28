# change-goal-003: `goal.json` Schema + Goals Directory

**Phase:** goal-loop-support
**Status:** pending
**Sub-phase:** A (core)
**Depends on:** change-goal-002

## Problem

No schema or directory structure exists for goal state. Without a persistent `goal.json`, goals cannot survive session restarts or be resumed across platforms.

## Solution

Define `goal.schema.json` as an additive extension of the existing `loop.json` schema, add `goal-directory-layout.md`, and wire `kbd-goal-start.sh` to validate against the schema.

## Files

- `skills/process/kbd-goal/references/schemas/goal.schema.json` (CREATE)
- `skills/process/kbd-goal/references/goal-directory-layout.md` (CREATE)

## Tasks

- [ ] Write `goal.schema.json` with required fields: `name`, `slug`, `description`, `phases[]`, `active_phase`, `status`, `tool`, `token_budget`, `max_turns_per_phase`, `max_no_progress_turns`, `created`, `updated`
- [ ] `phases[]` items: `{name, type: ideation|spec|creation|deployment, stopping_condition, human_gate: boolean}`
- [ ] Verify backward compatibility: existing `loop.json` files validate without `phases[]`
- [ ] Write `goal-directory-layout.md` documenting `.kbd-orchestrator/goals/<slug>/` contents
- [ ] Update `kbd-goal-start.sh` to validate generated `goal.json` with `jq -e` against schema
