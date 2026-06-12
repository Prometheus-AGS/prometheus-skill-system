---
id: change-004-outer-loop
title: pmpo-outer-loop standing-loop runner
phase: outer-loop-and-ux
gaps: [U4]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: proposed
scope:
  - skills/process/pmpo-outer-loop/SKILL.md
  - skills/process/pmpo-outer-loop/references/schemas/loop-definition.schema.json
---

# change-004 — pmpo-outer-loop

## Context

The Boris Cherny shape: "I don't prompt Claude anymore; I have loops running —
my job is to write loops." The user defines a goal + feedback sources once; the
framework runs cycles and only escalates at decision points. Wraps the EXISTING
evolver (no new loop engine, no daemon).

## Scope

In:

- New `skills/process/pmpo-outer-loop/SKILL.md` — three commands:
  - `/loop-define <name>` (interactive, via pmpo-elicit) writes
    `.kbd-orchestrator/loops/<name>/loop.json`.
  - `/loop-tick <name>` = exactly ONE cycle: collect feedback_sources → diff vs
    goal criteria → satisfied? terminate + final report : regression/stall
    (max_no_progress_ticks)? escalate via pmpo-elicit : run one `/evolve
    "<evolution_name>"` cycle (which composes zeespec, kbd-analyze, KBD
    execute) → append journal.md + decision-log.
  - `/loop-report <name>` renders journal.md (dual format).
  - Cadence delegated to platform primitives (manual / background task / cron
    scheduled agent) — documented, not implemented as a daemon.
  - Declares Progress Signals.
- `references/schemas/loop-definition.schema.json`: goal{description,
  measurable_criteria}, feedback_sources[] (command/gh-query/file/url),
  termination{goal_satisfied, max_ticks, max_no_progress_ticks, budget},
  escalation_points[], cadence{mode, schedule}, evolution_name.

Out: a running daemon (delegated to platform primitives).

## Tasks

- [ ] 1. Write loop-definition.schema.json
- [ ] 2. Write pmpo-outer-loop/SKILL.md (define/tick/report, evolver wrap, signals)
- [ ] 3. validate:strict + validate:signals green; build registers it

## Verification

validate:strict clean for the new skill; validate:signals green (not baselined);
build symlinks it; schema is valid JSON.
