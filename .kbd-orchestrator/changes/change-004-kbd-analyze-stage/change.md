---
id: change-004-kbd-analyze-stage
title: kbd-analyze research stage skill
phase: canonical-lifecycle
gaps: [G1]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: proposed
scope:
  - skills/process/kbd-process-orchestrator/skills/kbd-analyze/SKILL.md
  - skills/process/kbd-process-orchestrator/references/research-pipeline.md
  - skills/process/kbd-process-orchestrator/references/schemas/library-candidates.schema.json
  - skills/process/kbd-process-orchestrator/skills/kbd-plan/SKILL.md
  - skills/process/kbd-process-orchestrator/shared/lib/hooks.sh
  - skills/process/kbd-process-orchestrator/SKILL.md
---

# change-004 — kbd-analyze stage

## Context

KBD's lifecycle names Analyze but ships no skill — Analyze is prose with no
artifacts. This change makes it real: an engineering-landscape research stage
(libraries, frameworks, stack discovery) feeding spec/plan. Decision (from
plan): NEW skill, not promotion of evolve-analyze — KBD writes engineering
research into `.kbd-orchestrator/`; evolver writes business landscape into
`.evolver/`.

## Scope

In:

- New `KBD/skills/kbd-analyze/SKILL.md` — tiered pipeline:
  (1) `gh search repos/code` → (2) Context7/docfork → (3) registries
  (`npm view`/`cargo search`/PyPI) → (4) firecrawl/tavily. Two modes:
  stack-specified (rank candidates per assessment gap) and stack-discovery
  (2-3 scored stack options; contested = score gap <15% → escalate via
  pmpo-elicit when available, else flag for the user). Writes `analysis.md`,
  `library-candidates.json`, optional `stack-recommendation.md`. Skippable via
  `--skip`. Declares Progress Signals. Stage gate + handoff. Delegates to
  evolve-analyze when `evolver-bridge.json` present.
- `references/research-pipeline.md` (tier procedure + budget caps, shared SoT).
- `references/schemas/library-candidates.schema.json` (candidates with
  adopt|adapt|reference|reject verdicts, evidence, maintenance,
  research_budget).
- `kbd-plan/SKILL.md`: read library-candidates.json; annotate adopt/adapt
  changes with `library: cand-###`.
- `shared/lib/hooks.sh`: add `analyze` and `spec` to the allowed hook-kind enum.
- `KBD/SKILL.md`: lifecycle narrative + Quick Start + state-file table +
  status enum (analysis_ready) + model_routing.

Out: pmpo-elicit (change-005) — kbd-analyze references it but degrades to a
user flag when absent.

## Tasks

- [ ] 1. Write library-candidates.schema.json + research-pipeline.md
- [ ] 2. Write kbd-analyze/SKILL.md (pipeline, modes, gate/handoff, signals)
- [ ] 3. kbd-plan consumes candidates; hooks.sh enum; KBD SKILL.md updates
- [ ] 4. validate:strict + validate:signals green

## Verification

validate:strict clean; validate:signals green (kbd-analyze not baselined);
hooks.sh accepts `analyze`/`spec` kinds (extend test-hooks.sh if it enumerates).
