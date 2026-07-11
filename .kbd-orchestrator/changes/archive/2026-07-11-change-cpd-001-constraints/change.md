---
id: change-cpd-001-constraints
title: "Author .kbd-orchestrator/constraints.md so the artifact-refiner QA gate covers this phase onward"
phase: phase-codex-plugin-distribution-and-ci
gaps: [G-06]
priority: P1
effort: S
agent: claude-code
evolver_item_id: null
status: DONE
model_class: frontier
depends_on: []
scope:
  - .kbd-orchestrator/constraints.md
---

# change-cpd-001-constraints

**Objective.** Give KBD phases a real QA gate by declaring constraints artifact-refiner validates.

## Tasks

- [x] Author constraints.md: generated Codex artifacts must be in sync (validate:codex passes), no committed secrets, docs updated when the plugin surface changes, generators idempotent
- [x] Document the constraint format artifact-refiner reads (see references/integrations/artifact-refiner.md)
- [x] Note the gate applies from this phase forward
