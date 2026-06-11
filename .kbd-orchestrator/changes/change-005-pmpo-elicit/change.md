---
id: change-005-pmpo-elicit
title: pmpo-elicit ask-or-research primitive
phase: canonical-lifecycle
gaps: [G5]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: proposed
scope:
  - skills/process/pmpo-elicit/SKILL.md
  - skills/process/pmpo-elicit/references/schemas/elicitation.schema.json
  - skills/process/pmpo-elicit/references/integration-contract.md
---

# change-005 — pmpo-elicit

## Context

When the process is missing information it either asks everything upfront
(zeespec) or silently decides. The plan calls for a reusable primitive: ask the
user for the answer OR its source, and ALWAYS offer "research it for me."

## Scope

In:

- New `skills/process/pmpo-elicit/SKILL.md` — callable from any stage with
  `{question, hints[], criticality, write_back_path}`. Presents four option
  classes: (1) direct answers, (2) "here's the source" (fetch+extract,
  provenance recorded), (3) **"research it for me"** (research brief, budget
  `max_sources:6, max_minutes:10`, returns answer+confidence+evidence),
  (4) "decide for me" (explicit implicit). Inline-fallback mode this phase
  (child-isolated research arrives in the child-loops phase). Declares Progress
  Signals.
- `references/schemas/elicitation.schema.json` (request + result with provenance
  user|source|research|implicit, confidence, evidence, cost).
- `references/integration-contract.md` — how kbd-analyze/zeespec/any stage call
  it.

Out: zeespec interrogate.md rewrite (later phase) — contract documented here so
that change is mechanical when it lands.

## Tasks

- [ ] 1. Write elicitation.schema.json + integration-contract.md
- [ ] 2. Write pmpo-elicit/SKILL.md (4 option classes, budget guards, signals)
- [ ] 3. validate:strict + validate:signals green; npm run build registers it

## Verification

validate:strict clean for the new skill; validate:signals green (not
baselined); build symlinks the new skill into .claude-plugin.
