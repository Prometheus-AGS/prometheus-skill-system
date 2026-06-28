# change-goal-007: Claude Code Bridge

**Phase:** goal-loop-support
**Status:** pending
**Sub-phase:** B (integration)
**Depends on:** change-goal-002, change-goal-006
**Library:** claude-code-goal-native (ADOPT)

## Problem

On Claude Code, native `/goal` is faster and uses Haiku as evaluator without any extra infrastructure. KBD should delegate to it rather than reimplementing the loop, but must orchestrate above it for multi-phase goals.

## Solution

Document the Claude Code routing strategy: single-phase Creation → delegate to `claude /goal`; multi-phase → KBD orchestrates, delegates per-phase Creation to `claude /goal --worktree`; Ideation and Spec phases always owned by KBD.

## Files

- `skills/process/kbd-goal/references/platforms/claude-code.md` (CREATE)
- `skills/process/kbd-goal/SKILL.md` (UPDATE: Claude Code section)

## Tasks

- [ ] Write `claude-code.md` platform reference with routing decision table
- [ ] Document `claude /goal --tokens <budget> "<stopping_condition>"` invocation for single-phase
- [ ] Document `claude /goal --worktree "<phase-stopping-condition>"` for per-phase Creation in multi-phase
- [ ] Update `kbd-goal/SKILL.md` Claude Code section with routing logic
- [ ] Add detection: check for `CLAUDE_CODE` env var or `claude` binary in PATH
