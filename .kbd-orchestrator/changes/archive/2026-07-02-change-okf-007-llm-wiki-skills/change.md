---
id: change-okf-007-llm-wiki-skills
title: llm-wiki skill (ingest/query/lint operations) + layer-3 wiki schema doc
phase: phase-okf-llm-wiki-adoption
gaps: [Goal4]
priority: P1
effort: L
agent: claude-code
evolver_item_id: null
status: pending
model_class: frontier
depends_on: [change-okf-001]
scope:
  - skills/documentation/llm-wiki/SKILL.md
  - skills/documentation/llm-wiki/references/
---

# change-okf-007 — llm-wiki skill family

## Context

No wiki skill exists in this repo; the Karpathy integration is hooks-only.
Karpathy's architecture needs a layer-3 schema document (conventions +
workflows) and user-invocable operations: ingest, query, lint. All present
through pk (CLI or knowledge_* MCP tools), OKF-formatted per the vendored spec.

## Tasks

- [ ] skills/documentation/llm-wiki/SKILL.md: three operations — ingest
      (source → wiki integration + index/log update), query (answer with
      citations; file good answers back as wiki pages), lint (contradictions,
      orphans, stale claims, missing cross-references)
- [ ] references/wiki-schema.md: the layer-3 schema doc — directory structure,
      OKF conventions, ingest/query/lint workflows, index/log formats
- [ ] references/okf-conformance.md: producer checklist distilled from §9
- [ ] Frontmatter: version, license, metadata.tags — npm run validate:strict passes
- [ ] Final examples verified against change-okf-004 output format
