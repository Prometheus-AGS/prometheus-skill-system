---
id: change-slli-001-l3-outer-loop-skill
title: L3 outer-loop skill (pmpo-outer-loop)
phase: self-learning-loop-integration
gaps: [L3-LOOP-1, L3-LOOP-2, L3-LOOP-3]
priority: 4 of 10
agent: claude-code
status: done
scope:
  - skills/process/pmpo-outer-loop/SKILL.md
  - skills/process/pmpo-outer-loop/references/loop-schema.md
  - skills/process/pmpo-outer-loop/scripts/loop-tick.sh
  - .kbd-orchestrator/loops/test-loop/loop.json
---

# change-slli-001-l3-outer-loop-skill — L3 outer-loop skill (pmpo-outer-loop)

## Summary

Create `skills/process/pmpo-outer-loop/` with three commands (`/loop-define`, `/loop-tick`, `/loop-report`) and the canonical `loop.json` schema. This is the L3 layer that was identified as entirely absent: `.kbd-orchestrator/loops/` does not exist and no skill files back the commands.

## Files Created

- `skills/process/pmpo-outer-loop/SKILL.md`
- `skills/process/pmpo-outer-loop/references/loop-schema.md`
- `skills/process/pmpo-outer-loop/scripts/loop-tick.sh`

## loop.json Schema

```json
{
  "name": "string",
  "goal": "string — machine-checkable condition",
  "feedback": [{"type": "command|file|url|gh-query", "source": "string"}],
  "termination": {
    "max_ticks": "number",
    "max_no_progress_ticks": "number",
    "budget": "string e.g. '2h' or '5 USD'"
  },
  "escalation": "never|always|declared",
  "escalation_conditions": ["string"],
  "cadence": "manual|background|cron",
  "evolution_name": "string|null",
  "current_tick": "number",
  "no_progress_ticks": "number",
  "status": "active|paused|completed|escalated",
  "last_tick_at": "ISO8601",
  "created_at": "ISO8601"
}
```

## Acceptance Criteria

- `ls ~/.claude/skills/pmpo-outer-loop/SKILL.md` → exists (after install)
- `/loop-define test-loop` creates `.kbd-orchestrator/loops/test-loop/loop.json` with all 6 required parameter groups
- `/loop-tick test-loop` increments `current_tick` and writes feedback snapshot
- `/loop-report test-loop` renders readable progress table in plain text
- Validation rejects `loop.json` missing any of the 6 required parameter groups

## Tasks

- [x] 1. `ls ~/.claude/skills/pmpo-outer-loop/SKILL.md` → exists (after install)
- [x] 2. `/loop-define test-loop` creates `.kbd-orchestrator/loops/test-loop/loop.json` with all 6 required parameter groups
- [x] 3. `/loop-tick test-loop` increments `current_tick` and writes feedback snapshot
- [x] 4. `/loop-report test-loop` renders readable progress table in plain text
- [x] 5. Validation rejects `loop.json` missing any of the 6 required parameter groups
