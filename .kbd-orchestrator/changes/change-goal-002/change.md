---
id: change-goal-002
title: "`/kbd-goal` Unified Entry Point Skill"
phase: goal-loop-support
subphase: A (core)
depends_on: [change-goal-001]
agent: claude-code
status: done
scope:
  - skills/process/kbd-goal/SKILL.md
  - scripts/kbd-goal-start.sh
  - skills/process/kbd-goal/scripts/kbd-goal-start.sh
---

# change-goal-002 — `/kbd-goal` Unified Entry Point Skill

## Problem

No unified entry point exists for goal-driven loops. Users must manually chain `/loop-define` → `/loop-tick` → child phases, requiring deep knowledge of the orchestrator internals.

## Solution

Build `skills/process/kbd-goal/SKILL.md` + `scripts/kbd-goal-start.sh` — a skill that:
- Accepts a goal description + optional `--phases` and `--stop` flags
- Detects the active AI tool from environment / waypoint
- Creates `goal.json` + child phases automatically
- Routes to the correct platform strategy
- Is installed to all 5 platform skill directories

## Files

- `skills/process/kbd-goal/SKILL.md` (CREATE)
- `skills/process/kbd-goal/scripts/kbd-goal-start.sh` (CREATE)

## Tasks

- Write `SKILL.md` with valid agentskills frontmatter (name, version, license, tags)
- Document three invocation modes: single-phase, multi-phase (`--phases`), explicit stop (`--stop`)
- Document platform detection logic (env var `$TOOL` → waypoint `tool` field → default `claude-code`)
- Write `kbd-goal-start.sh`: parse args, create `goal.json`, call `kbd-new-child` per phase
- Add install entries for `~/.claude/skills/`, `~/.kimi-code/skills/`, `~/.opencode/skills/`, `~/.zed/skills/` in `install-skills-flat.sh`
- Validate SKILL.md: `npm run validate:strict skills/process/kbd-goal`

## Tasks

- [x] 1. Write `SKILL.md` with valid agentskills frontmatter (name, version, license, tags)
- [x] 2. Document three invocation modes: single-phase, multi-phase (`--phases`), explicit stop (`--stop`)
- [x] 3. Document platform detection logic (env var `$TOOL` → waypoint `tool` field → default `claude-code`)
- [x] 4. Write `kbd-goal-start.sh`: parse args, create `goal.json`, call `kbd-new-child` per phase
- [x] 5. Add install entries for `~/.claude/skills/`, `~/.kimi-code/skills/`, `~/.opencode/skills/`, `~/.zed/skills/` in `install-skills-flat.sh`
- [x] 6. Validate SKILL.md: `npm run validate:strict skills/process/kbd-goal`
