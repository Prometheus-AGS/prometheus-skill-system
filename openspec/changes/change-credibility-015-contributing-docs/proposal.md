---
id: change-credibility-015-contributing-docs
title: Add CONTRIBUTING.md, GitHub issue templates, and deployment-modes doc
phase: phase-credibility-closure
priority: P2
effort: M
wave: 3
parallel: true
agent: claude
status: done
gap_id: P3-A
verdict: BUILD
scope:
  - CONTRIBUTING.md
  - .github/ISSUE_TEMPLATE/bug_report.md
  - .github/ISSUE_TEMPLATE/feature_request.md
  - .github/ISSUE_TEMPLATE/skill_proposal.md
  - docs/deployment-modes.md
---

# change-credibility-015 — Add CONTRIBUTING.md, GitHub issue templates, and deployment-modes doc

## Context

The repository has no CONTRIBUTING.md, no GitHub issue templates, and no documentation of the four deployment modes (Mode 0/1/2/3 from the analysis). External reviewers flagged these as missing signals for a project claiming production-readiness. Well-documented contribution paths and deployment modes demonstrate engineering maturity.

## Scope

1. `CONTRIBUTING.md` — setup, skill development workflow, validation steps, PR checklist
2. `.github/ISSUE_TEMPLATE/bug_report.md` — structured bug report with reproduction steps
3. `.github/ISSUE_TEMPLATE/feature_request.md` — feature request with use case
4. `.github/ISSUE_TEMPLATE/skill_proposal.md` — skill contribution proposal
5. `docs/deployment-modes.md` — Mode 0 (CLI only), Mode 1 (MCP tools), Mode 2 (full daemon stack), Mode 3 (P2P sync) with capability matrix

## Implementation Notes

`CONTRIBUTING.md` outline:
```markdown
# Contributing to prometheus-skill-pack

## Prerequisites
- Node.js 20+, Rust stable, npm

## Setup
git clone --recurse-submodules ...
npm install
bash scripts/install-skills-flat.sh

## Creating a Skill
1. Choose domain: skills/{react,rust,ui-ux,devops,testing,learn,...}
2. mkdir -p skills/<domain>/<skill-name> && cp docs/SKILL_TEMPLATE.md .../SKILL.md
3. Edit frontmatter, write instructions
4. npm run validate:strict skills/<domain>/<skill-name>
5. npm run install:project && test with Claude Code

## PR Checklist
- [ ] All skills validate strict
- [ ] No SSH submodule URLs
- [ ] No hardcoded credentials
- [ ] forge-rs tests pass (cargo test)
- [ ] package-lock.json committed (npm ci clean)
```

`docs/deployment-modes.md` capability matrix:

| Capability | Mode 0 CLI | Mode 1 MCP | Mode 2 Full | Mode 3 P2P |
|---|---|---|---|---|
| forge enrich/validate/reflect | YES | YES | YES | YES |
| surreal-memory KB | NO | YES | YES | YES |
| surface-bridge UI | NO | NO | YES | YES |
| sovereign-sync P2P | NO | NO | NO | YES |
| Required services | none | surreal-memory | +surface-bridge | +sovereign-sync |

## Verification

- `CONTRIBUTING.md` exists and covers setup, skill creation, PR checklist
- Three GitHub issue templates render correctly in the Issues tab
- `docs/deployment-modes.md` has the Mode 0-3 capability matrix
