---
id: skills-overview
title: Skills Overview
sidebar_label: Skills Overview
---

# Skills Overview

See the full chapter:
[docs/guide/08-skills-overview.md](https://github.com/prometheusags/prometheus-skill-pack/blob/main/docs/guide/08-skills-overview.md)

## Skill categories

| Category | Skills | Description |
|----------|--------|-------------|
| `learn/` | 15 skills | Feynman learning engine + P2P sync |
| `process/` | 8 skills | KBD lifecycle, PMPO, evolution |
| `react/` | 4 skills | React entity management |
| `rust/` | 3 skills | Rust development patterns |
| `ui-ux/` | 3 skills | UI/UX design workflows |
| `devops/` | 4 skills | GitOps CI/CD, ArgoCD |
| `testing/` | 3 skills | BDD, E2E, coverage |
| `documentation/` | 2 skills | Doc generation |

## Skill discovery

Skills are discoverable by name via:

```bash
/skill-name                         # Direct invocation
/sync-status                        # Trigger by intent
"check sync status"                 # Natural language trigger
```

The `SkillIndex` in `sovereign-sync` indexes all skill names and descriptions for
keyword-based MCP tool search (`search-skills`).
