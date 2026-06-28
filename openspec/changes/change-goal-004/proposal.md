# change-goal-004: Ideation Child-Phase Template

**Phase:** goal-loop-support
**Status:** pending
**Sub-phase:** A (core)
**Depends on:** change-goal-003

## Problem

No Ideation phase exists in KBD. The `ideation-mindmap` skill generates content but has no convergence loop — no scoring, no critic agent, no stopping condition based on N candidates passing a threshold.

## Solution

Build an Ideation phase template (discovery + critic loop) and a `kbd-idea-critic` subagent that scores candidates against a rubric until ≥ 3 score ≥ 7.0 or `max_turns` is reached, then surfaces `IDEAS.md` to the human.

## Files

- `skills/process/kbd-goal/references/templates/ideation-phase.md` (CREATE)
- `agents/kbd-idea-critic.md` (CREATE)

## Tasks

- [ ] Write `ideation-phase.md` template documenting the 4-step convergence loop
- [ ] Define rubric dimensions: feasibility (0–10), pain_addressed (0–10), stack_fit (0–10), buildability (0–10); aggregate = mean
- [ ] Write `agents/kbd-idea-critic.md`: Sonnet model, scoring rubric in system prompt, JSON output schema
- [ ] Define `IDEAS.md` format: scored markdown table + rationale per candidate + survivor section
- [ ] Document stopping condition: `≥3 candidates with aggregate ≥ 7.0` → human gate
- [ ] Update `kbd-goal/SKILL.md` with Ideation Phase section
