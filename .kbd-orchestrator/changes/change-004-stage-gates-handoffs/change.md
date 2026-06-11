---
id: change-004-stage-gates-handoffs
title: Stage handoff artifacts and precondition gates
phase: position-and-handoff-guarantee
gaps: [F3]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: done
scope:
  - skills/process/kbd-process-orchestrator/shared/lib/stage-gate.sh
  - skills/process/kbd-process-orchestrator/references/schemas/handoff.schema.json
  - skills/process/kbd-process-orchestrator/skills/kbd-assess/SKILL.md
  - skills/process/kbd-process-orchestrator/skills/kbd-plan/SKILL.md
  - skills/process/kbd-process-orchestrator/skills/kbd-execute/SKILL.md
  - skills/process/kbd-process-orchestrator/skills/kbd-reflect/SKILL.md
  - skills/process/kbd-process-orchestrator/shared/lib/tests/test-stage-gate.sh
---

# change-004 — Stage handoff artifacts and precondition gates

## Context

Stage transitions today rely on convention; nothing records what a stage
produced for the next one, and nothing but pipeline-enforce.sh (Bash-only,
execute/reflect-only) gates ordering. Handoffs make every stage's output the
next stage's explicit input and give the position model its stage edge data.

## Scope

In:

- New `KBD/shared/lib/stage-gate.sh`:
  - `kbd_stage_gate <stage>` — stage order table
    `assess → analyze → spec → plan → execute → reflect` (analyze/spec slots
    reserved now; gate treats a missing handoff for a stage that has no skill
    yet as implicitly skipped).
  - Previous stage's `handoffs/<stage>.handoff.json` must exist with
    `skipped:false|true`; missing → exit 2 printing exact remediation command.
  - Legacy mode: phase dir has no `handoffs/` at all → warn to stderr, return 0.
  - `kbd_stage_handoff_write <stage> <nextStage> <summary> [outputs...]` —
    atomic write of the handoff JSON (temp+mv).
- New `KBD/references/schemas/handoff.schema.json` (stage, completedAt,
  outputs[], nextStage, summaryForNext, skipped, skipReason).
- SKILL.md edits (mechanical, "Hook integration"-adjacent section) for
  kbd-assess, kbd-plan, kbd-execute, kbd-reflect: call the gate first, write the
  handoff on completion.
- New `KBD/shared/lib/tests/test-stage-gate.sh` (follow existing test layout
  under shared/lib or shared/scripts/tests — match repo convention found at
  implementation time).

## Tasks

- [x] 1. Write stage-gate.sh (gate + handoff writer)
- [x] 2. Write handoff.schema.json
- [x] 3. Edit 4 stage SKILL.md files (gate call + handoff write instructions)
- [x] 4. Test; run green

## Verification

Test green; fixture phase without handoffs dir passes with warning (legacy);
missing-handoff case exits 2 with remediation text.
