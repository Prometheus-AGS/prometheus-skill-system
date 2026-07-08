---
id: change-drs-007-docs-updates
title: Update SKILLS.md, README.md, and docs to register deep-research skill
phase: phase-deep-research-skill
priority: P2
effort: S
wave: 3
agent: general-purpose
status: pending
gap_id: G-06
verdict: BUILD
depends_on: change-drs-006-validation
scope:
  - README.md (add research category + deep-research skill entry)
  - docs/CONTRIBUTING.md (add research category guidance)
  - marketplace/marketplace.json (add deep-research to skill list)
---

# change-drs-007 — Docs Updates

## Context

After validation passes, register the new skill in README.md and marketplace
config so it is discoverable. No SKILLS.md exists — use README.md and the
marketplace manifest.

## README.md Changes

Add a new "Research" category section under the skills table or list. Entry:

```markdown
### Research

| Skill | Description | Version |
|-------|-------------|---------|
| [deep-research](skills/research/deep-research/) | 10-stage pipeline: query decomposition → web search → retrieval → verification → knowledge graph → report synthesis. Produces OKF-aligned `.research` packages with citations, confidence scores, and contradiction tracking. | 1.0.0 |
```

## marketplace/marketplace.json Changes

Add entry to the skills array:

```json
{
  "name": "deep-research",
  "path": "skills/research/deep-research",
  "version": "1.0.0",
  "category": "research",
  "description": "10-stage deep research pipeline with verification, knowledge graph, and OKF report export",
  "tags": ["research", "deep-research", "knowledge-graph", "citations", "verification"]
}
```

## docs/CONTRIBUTING.md Changes

Add a short "Research skills" section noting:
- Research skills live in `skills/research/<skill-name>/`
- Sub-skills use `deep-research-stage-0N` naming convention
- Scripts must produce JSON output
- `.research` package format follows OKF v0.1 + Prometheus extensions

## Acceptance Criteria

- [ ] README.md has "Research" category with deep-research entry
- [ ] marketplace/marketplace.json has deep-research entry (valid JSON)
- [ ] docs/CONTRIBUTING.md has research skills guidance
- [ ] All changed files pass `npm run check-format` (or `npm run format` auto-fixes)
