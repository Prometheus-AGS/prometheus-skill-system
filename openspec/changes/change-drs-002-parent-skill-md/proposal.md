---
id: change-drs-002-parent-skill-md
title: Write skills/research/deep-research/SKILL.md — parent orchestration entry point
phase: phase-deep-research-skill
priority: P0
effort: M
wave: 1
agent: general-purpose
status: pending
gap_id: G-02
verdict: BUILD
depends_on: change-drs-001-directory-structure
scope:
  - skills/research/deep-research/SKILL.md
---

# change-drs-002 — Parent SKILL.md

## Context

The parent `SKILL.md` is the primary entry point loaded by any harness. It must:
- Pass `npm run validate:strict` (requires: `name`, `description`, `license`, `version`, `metadata.tags`)
- Define triggers that activate the skill across all supported platforms
- Orchestrate the 10-stage pipeline via prose instructions referencing sub-skills
- Document the `.research` package format (OKF v0.1 + extensions)
- Document integration with surreal-memory, liter-llm, sycophancy-correction, Feynman skills

## Design decisions (from analysis)

- `allowed-tools`: `file_system web_search code_interpreter sequential_thinking memory browser tavily firecrawl`
- `model_routing`: tiered — frontier/medium/small by stage reasoning load
- Sub-skills invoked via path reference: `skills/research/deep-research/skills/stage-0N-<name>/SKILL.md`
- Stage execution: sequential default
- OKF v0.1 base format with Prometheus research extensions for `.research` package

## SKILL.md frontmatter

```yaml
---
name: deep-research
description: >
  10-stage deep research pipeline: Planner → Search → Retrieve → Collect →
  Verify → Resolve → Graph → Cite → Report → Export. Produces persistent
  .research packages (OKF-aligned knowledge assets) with citations, confidence
  scores, knowledge graphs, and contradiction tracking. Integrates with
  surreal-memory, liter-llm, sycophancy-correction, and Feynman learning skills.
  Supersedes disposable report generation with structured knowledge infrastructure.
license: MIT
version: '1.0.0'
allowed-tools: file_system web_search code_interpreter sequential_thinking memory browser tavily firecrawl
model_routing:
  policy_source: ".kbd-orchestrator/project.json → model_policy"
  phases:
    research-plan: frontier
    research-search: medium
    research-retrieve: medium
    research-collect: medium
    research-verify: frontier
    research-resolve: frontier
    research-graph: frontier
    research-cite: small
    research-synthesize: frontier
    research-export: small
  routing_reference: "references/pipeline-architecture.md"
triggers:
  keywords:
    - research
    - deep research
    - investigate
    - analyze
    - deep dive
    - comprehensive report
    - what is the current state of
    - competitive analysis
    - market research
    - technology evaluation
    - literature review
    - knowledge synthesis
    - due diligence
    - study
  semantic: >
    Any request requiring synthesis from multiple web sources with citations,
    verification, and structured knowledge output. Research questions, competitive
    analysis, technology evaluations, due diligence, and any topic requiring
    evidence-backed findings across multiple sources.
metadata:
  author: Prometheus AGS
  version: '1.0.0'
  category: research
  tags: [research, deep-research, knowledge-graph, okf, pipeline, citations, verification]
---
```

## SKILL.md body sections (required)

1. **When to Use** — activation scenarios
2. **Quick Start** — `/deep-research <query>` or `/research <query>`
3. **10-Stage Pipeline** — table + prose, sub-skill references
4. **.research Package Format** — OKF frontmatter + Prometheus extensions
5. **Integration Guide** — surreal-memory, liter-llm, Feynman quality gate
6. **Examples** — 3 example queries with expected outputs
7. **Common Issues** — troubleshooting

## Acceptance Criteria

- [ ] `skills/research/deep-research/SKILL.md` exists
- [ ] `npm run validate:strict skills/research/deep-research` exits 0
- [ ] Frontmatter contains: name, description, license, version, metadata.tags
- [ ] Body contains all 7 required sections
- [ ] Sub-skill references use relative paths: `skills/research/deep-research/skills/stage-0N-<name>/SKILL.md`
- [ ] File is under 500 lines (detailed content in references/)
