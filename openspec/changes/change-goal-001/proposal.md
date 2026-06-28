# change-goal-001: Separated Evaluator Subagent

**Phase:** goal-loop-support
**Status:** pending
**Sub-phase:** A (core)
**Depends on:** none

## Problem

Every platform except Claude Code and Codex lacks a separated evaluator — a model instance that grades whether a goal's stopping condition is met independently from the builder model. Self-grading bias causes the builder to over-report success.

## Solution

Build `agents/kbd-goal-evaluator.md` — a dedicated subagent with:
- Small/fast model (Haiku-class) for cost efficiency
- Read-only tool access only
- System prompt that grades a stopping condition against STATE.md / test output → returns exactly `PASS` or `FAIL` + one-sentence reason

## Files

- `agents/kbd-goal-evaluator.md` (CREATE)

## Tasks

- [ ] Write `agents/kbd-goal-evaluator.md` with valid YAML frontmatter
- [ ] System prompt: strict condition grader, no write access, JSON output `{verdict: "PASS"|"FAIL", reason: string}`
- [ ] Set `model: claude-haiku-4-5-20251001` in frontmatter
- [ ] Set `disable-model-invocation: false` (orchestrator may auto-invoke)
- [ ] Manual test: FAIL case (failing tests → STATE.md) returns FAIL
- [ ] Manual test: PASS case (all green) returns PASS
