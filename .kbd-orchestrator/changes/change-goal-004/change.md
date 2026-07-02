---
id: change-goal-004
title: Ideation Child-Phase Template
phase: goal-loop-support
subphase: A (core)
depends_on: [change-goal-003]
agent: claude-code
status: done
scope:
  - skills/process/kbd-goal/references/templates/ideation-phase.md
  - agents/kbd-idea-critic.md
  - kbd-goal/SKILL.md
---

# change-goal-004 — Ideation Child-Phase Template

## Problem

No Ideation phase exists in KBD. The `ideation-mindmap` skill generates content but has no convergence loop — no scoring, no critic agent, no stopping condition based on N candidates passing a threshold.

## Solution

Build an Ideation phase template (discovery + critic loop) and a `kbd-idea-critic` subagent that scores candidates against a rubric until ≥ 3 score ≥ 7.0 or `max_turns` is reached, then surfaces `IDEAS.md` to the human.

## Files

- `skills/process/kbd-goal/references/templates/ideation-phase.md` (CREATE)
- `agents/kbd-idea-critic.md` (CREATE)

## Tasks

- Write `ideation-phase.md` template documenting the 4-step convergence loop
- Define rubric dimensions: feasibility (0–10), pain_addressed (0–10), stack_fit (0–10), buildability (0–10); aggregate = mean
- Write `agents/kbd-idea-critic.md`: Sonnet model, scoring rubric in system prompt, JSON output schema
- Define `IDEAS.md` format: scored markdown table + rationale per candidate + survivor section
- Document stopping condition: `≥3 candidates with aggregate ≥ 7.0` → human gate
- Update `kbd-goal/SKILL.md` with Ideation Phase section

## Tasks

- [x] 1. Write `ideation-phase.md` template documenting the 4-step convergence loop
- [x] 2. Define rubric dimensions: feasibility (0–10), pain_addressed (0–10), stack_fit (0–10), buildability (0–10); aggregate = mean
- [x] 3. Write `agents/kbd-idea-critic.md`: Sonnet model, scoring rubric in system prompt, JSON output schema
- [x] 4. Define `IDEAS.md` format: scored markdown table + rationale per candidate + survivor section
- [x] 5. Document stopping condition: `≥3 candidates with aggregate ≥ 7.0` → human gate
- [x] 6. Update `kbd-goal/SKILL.md` with Ideation Phase section
