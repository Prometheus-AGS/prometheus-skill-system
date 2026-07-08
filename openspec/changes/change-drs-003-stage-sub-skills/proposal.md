---
id: change-drs-003-stage-sub-skills
title: Write all 10 stage sub-skill SKILL.md files (stage-01 through stage-10)
phase: phase-deep-research-skill
priority: P0
effort: L
wave: 1
agent: general-purpose
status: pending
gap_id: G-03
verdict: BUILD
depends_on: change-drs-002-parent-skill-md
scope:
  - skills/research/deep-research/skills/stage-01-planner/SKILL.md
  - skills/research/deep-research/skills/stage-02-search/SKILL.md
  - skills/research/deep-research/skills/stage-03-retrieve/SKILL.md
  - skills/research/deep-research/skills/stage-04-collect/SKILL.md
  - skills/research/deep-research/skills/stage-05-verify/SKILL.md
  - skills/research/deep-research/skills/stage-06-resolve/SKILL.md
  - skills/research/deep-research/skills/stage-07-graph/SKILL.md
  - skills/research/deep-research/skills/stage-08-cite/SKILL.md
  - skills/research/deep-research/skills/stage-09-report/SKILL.md
  - skills/research/deep-research/skills/stage-10-export/SKILL.md
---

# change-drs-003 — Stage sub-skill SKILL.md files

## Context

Each of the 10 pipeline stages needs a self-contained `SKILL.md` with:
- Valid strict-mode frontmatter (name, description, license, version, metadata.tags)
- Input/output contract (typed fields with descriptions)
- Stage-specific instructions (3-5 steps)
- Integration note (which infrastructure component this stage uses)
- Example (concrete input → output)

**Naming convention (from analysis):**
- Directory: `stage-0N-<name>/`
- Frontmatter `name:`: `deep-research-stage-0N` (prefixed to avoid namespace collision)

## Sub-skills Table

| Dir | name | Purpose | Key integration | Model class |
|-----|------|---------|-----------------|-------------|
| stage-01-planner | deep-research-stage-01 | Decompose query → sub-questions + plan | zeespec-interrogator (optional) | frontier |
| stage-02-search | deep-research-stage-02 | Web search → source URLs | firecrawl_search, tavily_search | medium |
| stage-03-retrieve | deep-research-stage-03 | Retrieve + chunk content | firecrawl_scrape, kreuzberg | medium |
| stage-04-collect | deep-research-stage-04 | Index sources into graph | surreal-memory create_entity | medium |
| stage-05-verify | deep-research-stage-05 | Verify source credibility | sycophancy-correction detect_sycophancy | frontier |
| stage-06-resolve | deep-research-stage-06 | Resolve contradictions | pmpo-elicit (human escalation) | frontier |
| stage-07-graph | deep-research-stage-07 | Build knowledge graph | surreal-memory create_relation | frontier |
| stage-08-cite | deep-research-stage-08 | Generate citations + confidence | surreal-memory add_memory | small |
| stage-09-report | deep-research-stage-09 | Synthesize report | learn-grade quality gate | frontier |
| stage-10-export | deep-research-stage-10 | Export .research package | OKF v0.1 format | small |

## Frontmatter pattern (apply to all 10)

```yaml
---
name: deep-research-stage-0N
description: <one-sentence purpose, max 120 chars>
license: MIT
version: '1.0.0'
metadata:
  author: Prometheus AGS
  category: research
  tags: [deep-research, stage-0N, <stage-specific-tag>]
---
```

## Required body sections per sub-skill

1. **Purpose** — one paragraph
2. **Input** — typed field list
3. **Output** — typed field list
4. **Instructions** — 3-5 numbered steps
5. **Integration** — which infrastructure component, how invoked
6. **Example** — concrete input → output block

## Acceptance Criteria

- [ ] All 10 `SKILL.md` files exist (one per stage dir)
- [ ] Each has valid strict-mode frontmatter (name matches `deep-research-stage-0N` pattern)
- [ ] Each has all 6 required body sections
- [ ] `npm run validate:strict skills/research/deep-research` still exits 0 (sub-skills are scanned recursively)
- [ ] No sub-skill file exceeds 200 lines
