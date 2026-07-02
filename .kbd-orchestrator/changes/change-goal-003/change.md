---
id: change-goal-003
title: "`goal.json` Schema + Goals Directory"
phase: goal-loop-support
subphase: A (core)
depends_on: [change-goal-002]
agent: claude-code
status: done
scope:
  - skills/process/kbd-goal/references/schemas/goal.schema.json
  - skills/process/kbd-goal/references/goal-directory-layout.md
---

# change-goal-003 — `goal.json` Schema + Goals Directory

## Problem

No schema or directory structure exists for goal state. Without a persistent `goal.json`, goals cannot survive session restarts or be resumed across platforms.

## Solution

Define `goal.schema.json` as an additive extension of the existing `loop.json` schema, add `goal-directory-layout.md`, and wire `kbd-goal-start.sh` to validate against the schema.

## Files

- `skills/process/kbd-goal/references/schemas/goal.schema.json` (CREATE)
- `skills/process/kbd-goal/references/goal-directory-layout.md` (CREATE)

## Tasks

- Write `goal.schema.json` with required fields: `name`, `slug`, `description`, `phases[]`, `active_phase`, `status`, `tool`, `token_budget`, `max_turns_per_phase`, `max_no_progress_turns`, `created`, `updated`
- `phases[]` items: `{name, type: ideation|spec|creation|deployment, stopping_condition, human_gate: boolean}`
- Verify backward compatibility: existing `loop.json` files validate without `phases[]`
- Write `goal-directory-layout.md` documenting `.kbd-orchestrator/goals/<slug>/` contents
- Update `kbd-goal-start.sh` to validate generated `goal.json` with `jq -e` against schema

## Tasks

- [x] 1. Write `goal.schema.json` with required fields: `name`, `slug`, `description`, `phases[]`, `active_phase`, `status`, `tool`, `token_budget`, `max_turns_per_phase`, `max_no_progress_turns`, `created`, `updated`
- [x] 2. `phases[]` items: `{name, type: ideation|spec|creation|deployment, stopping_condition, human_gate: boolean}`
- [x] 3. Verify backward compatibility: existing `loop.json` files validate without `phases[]`
- [x] 4. Write `goal-directory-layout.md` documenting `.kbd-orchestrator/goals/<slug>/` contents
- [x] 5. Update `kbd-goal-start.sh` to validate generated `goal.json` with `jq -e` against schema
