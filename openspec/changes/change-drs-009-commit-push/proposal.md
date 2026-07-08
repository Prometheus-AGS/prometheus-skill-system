---
id: change-drs-009-commit-push
title: Commit and push deep-research skill to main
phase: phase-deep-research-skill
priority: P1
effort: S
wave: 4
agent: general-purpose
status: pending
gap_id: G-07
verdict: BUILD
depends_on: change-drs-008-install-smoke-test
scope:
  - skills/research/ (new directory tree, all files)
  - README.md (updated)
  - marketplace/marketplace.json (updated)
  - docs/CONTRIBUTING.md (updated)
---

# change-drs-009 — Commit + Push

## Context

All changes are on the local branch. Commit with a conventional commits message
and push to origin/main.

## Pre-commit Checklist

Before committing:
- [ ] `git status` shows only expected files (skills/research/, README.md, marketplace/, docs/)
- [ ] No `.env`, secrets, or temp files staged
- [ ] `npm run validate:strict skills/research/deep-research` exits 0 (already verified in change-006)
- [ ] `npm run check-format` exits 0 (already verified in change-007)

## Commit Message

```
feat(research): add deep-research skill — 10-stage pipeline with OKF output

Implements the deep research skill defined in docs/deep-research-skill-playbook.md.

Key additions:
- skills/research/deep-research/ — parent orchestration skill
- 10 stage sub-skills (stage-01-planner through stage-10-export)
- 5 automation scripts (run-research, export-package, verify-sources, build-graph,
  detect-contradictions)
- 5 structured templates (research-plan, source-evaluation, contradiction-resolution,
  report-template, research-package-manifest)
- 9 reference documents (pipeline architecture, OKF format, model routing, integrations)
- 4 lifecycle hooks (pre-research, post-stage, on-contradiction, post-export)
- 4 subagent descriptors (research-planner, source-verifier, contradiction-resolver,
  report-synthesizer)

Pipeline: Planner → Search → Retrieve → Collect → Verify → Resolve → Graph →
Cite → Report → Export

Output: OKF-aligned .research packages with citations, knowledge graph,
confidence scores, and optional Feynman quality gate.

Integrations: surreal-memory, liter-llm-bridge, sycophancy-correction, pmpo-elicit.
```

## Push Command

```bash
git push origin main
```

## Acceptance Criteria

- [ ] `git status` after add shows only expected files
- [ ] Commit created with conventional commits message
- [ ] `git push origin main` exits 0
- [ ] Commit appears on origin/main (`git log --oneline -1`)
