---
id: change-okf-001-vendor-specs
title: Vendor OKF v0.1 + Karpathy LLM Wiki docs; record adoption decision in CLAUDE.md
phase: phase-okf-llm-wiki-adoption
gaps: [Goal1, Goal4]
priority: P1
effort: S
agent: claude-code
evolver_item_id: null
status: pending
model_class: small
scope:
  - shared/references/okf-v0.1.md
  - shared/references/llm-wiki-pattern.md
  - CLAUDE.md
---

# change-okf-001 — Vendor specs and record OKF adoption decision

## Context

The OKF v0.1 spec and Karpathy LLM Wiki pattern doc exist only under
`.kbd-orchestrator/phases/phase-okf-llm-wiki-adoption/inputs/`. Goals must be
checkable against committed artifacts, and CLAUDE.md is the canonical source
for cross-cutting decisions (documentation-hierarchy rule).

## Tasks

- [ ] Copy inputs/okf-v0.1.md → shared/references/okf-v0.1.md
- [ ] Copy inputs/llm-wiki-karpathy.md → shared/references/llm-wiki-pattern.md
- [ ] Add CLAUDE.md section: OKF v0.1 adoption decision + cross-repo ownership
      split (format = prometheus-knowledge-rs; skills/schema/hooks = this repo)
- [ ] Verify: npm run validate passes; both reference files render clean
