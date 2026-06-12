---
id: change-005-dual-ux
title: Dual-audience UX — decision-log, kbd-status --explain, ux_profile
phase: outer-loop-and-ux
gaps: [U5]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: proposed
scope:
  - skills/process/kbd-process-orchestrator/references/templates/decision-log.template.md
  - skills/process/kbd-process-orchestrator/skills/kbd-status/SKILL.md
  - skills/process/kbd-process-orchestrator/references/schemas/project.template.json
---

# change-005 — Dual-audience UX

## Context

Advanced users need dense status; beginners need decisions explained (what was
decided, why, what to learn). One artifact + status mode serves both.

## Scope

In:

- New `references/templates/decision-log.template.md` — per-entry format:
  `## D-NNN · <decision>   [stage · date]` + TL;DR / Why / Alternatives /
  Learn-more lines. (kbd-analyze and pmpo-outer-loop already reference a
  decision-log; this is the template.)
- `kbd-status/SKILL.md`: add a `--explain` flag — expands decision-log header
  lines into full entries and adds a "what happens next and why" narrative.
  Default (no flag) stays dense (header lines only).
- `project.template.json`: add `ux_profile: "beginner" | "advanced"` — sets the
  default kbd-status verbosity (beginner → --explain on by default). NEVER gates
  information; only ordering/expansion.

## Tasks

- [ ] 1. Write decision-log.template.md
- [ ] 2. kbd-status --explain (expand entries + next-and-why narrative)
- [ ] 3. ux_profile in project.template.json; validate:strict green

## Verification

validate:strict clean; the template exists; kbd-status SKILL.md documents
--explain and its dense default; ux_profile documented as verbosity-only.
