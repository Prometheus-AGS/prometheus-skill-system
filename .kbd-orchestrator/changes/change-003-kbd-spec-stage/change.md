---
id: change-003-kbd-spec-stage
title: kbd-spec stage skill
phase: canonical-lifecycle
gaps: [G3]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: proposed
scope:
  - skills/process/kbd-process-orchestrator/skills/kbd-spec/SKILL.md
  - skills/process/kbd-process-orchestrator/SKILL.md
---

# change-003 — kbd-spec stage

## Context

The canonical lifecycle has a Spec stage between Analyze and Plan, but no skill
owns it; changes are created ad hoc during planning. kbd-spec formalizes change
creation and is where zeespec coverage gates the lifecycle.

## Scope

In:

- New `KBD/skills/kbd-spec/SKILL.md`:
  - Consumes `analysis.json` (when present) + phase goals.
  - Creates native changes (`spec.md` + `tasks.json` + `verification.md` per the
    native-backend layout from change-001) OR emits `/opsx:new <id>` when the
    active backend is openspec.
  - Reads zeespec coverage when `.zeespec/` exists; records the GO/CAUTION/NO-GO
    verdict in its handoff. NO-GO → instruct the operator to run
    `/zeespec-interrogate` before proceeding (spec→plan gate remediation).
  - Stage gate (`kbd_stage_gate spec`) + handoff write
    (`kbd_stage_handoff_write spec ...`).
  - Declares Progress Signals (Starting/Completed kbd-spec).
- `KBD/SKILL.md`: add `/kbd-spec` to Quick Start + lifecycle narrative.

Out: native backend itself (change-001), analyze stage (change-004).

## Tasks

- [ ] 1. Write kbd-spec/SKILL.md (creation + zeespec gate + stage gate/handoff + signals)
- [ ] 2. Register in KBD SKILL.md lifecycle + Quick Start
- [ ] 3. validate:strict + validate:signals green for the new skill

## Verification

`npm run validate:strict skills/process/kbd-process-orchestrator` clean;
`npm run validate:signals` still green (kbd-spec not baselined).
