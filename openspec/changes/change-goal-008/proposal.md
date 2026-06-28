# change-goal-008: Codex CLI Bridge

**Phase:** goal-loop-support
**Status:** pending
**Sub-phase:** B (integration)
**Depends on:** change-goal-002, change-goal-006
**Library:** codex-goal-native (ADOPT, v≥0.128.0)

## Problem

Codex uses AGENTS.md for context (not SKILL.md) and requires `goals/continuation.md` + `goals/budget_limit.md` prompt files to drive its Ralph loop. KBD must generate these and inject phase context without clobbering user's existing AGENTS.md.

## Solution

Ship Codex-specific prompt templates, a setup script that installs them to `~/.codex/goals/`, and a context prefix generator that writes `.codex-kbd-context.md` for user-controlled @include in AGENTS.md.

## Files

- `skills/process/kbd-goal/templates/codex/continuation.md` (CREATE)
- `skills/process/kbd-goal/templates/codex/budget_limit.md` (CREATE)
- `skills/process/kbd-goal/references/platforms/codex.md` (CREATE)
- `scripts/kbd-goal-codex-setup.sh` (CREATE)

## Tasks

- [ ] Write `continuation.md` template: re-read STATE.md, take next unchecked TASKS.md item, implement, run tests
- [ ] Write `budget_limit.md` template: write progress wrap-up to STATE.md → budget_summary and stop gracefully
- [ ] Write `kbd-goal-codex-setup.sh`: copies templates to `~/.codex/goals/`; sets `goals.enabled = true` in `~/.codex/config.toml`
- [ ] Write `.codex-kbd-context.md` generator function in setup script: writes phase context file with goal slug, active phase, TASKS.md path, stopping condition
- [ ] Document user instruction: add `@include .codex-kbd-context.md` to project AGENTS.md
- [ ] Write `codex.md` platform reference with full routing + setup procedure
- [ ] Update `kbd-goal/SKILL.md` Codex section
