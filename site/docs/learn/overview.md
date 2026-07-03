---
id: overview
title: Learn Domain Overview
sidebar_label: Overview
---

# Learn Domain — Feynman-Spine

The Learn domain implements a complete learning and education capability using the
**Feynman-Spine** methodology. It spans 15 skills across 4 architectural layers.

## The learning arc

```
/learn-goal       → set what you want to learn
/learn-survey     → diagnostic placement + recursion floor
/learn-plan       → concept DAG + curriculum builder
/feynman-loop     → core Feynman PMPO loop (explain, gap-find, re-study)
/learn-grade      → sycophancy-corrected external grading
/learn-retain     → FSRS-6 spaced retrieval
/learn-practice   → deliberate practice track
/learn-certify    → OB 3.0 / W3C VC certification
```

## Support skills

| Skill | Purpose |
|-------|---------|
| `/learn-kb` | KB registry + adapter management |
| `/learn-about-system` | Prometheus stack meta-learning |
| `/learn-harness` | Harness detection + capability map |
| `/ui-surface` | Cross-harness UI rendering primitive |
| `/sync-status` | Check P2P sync status |
| `/sync-peers` | Manage P2P peers |
| `/sync-push` | Push CRDT domains to peers |

## Core invariants

- **Self-reported fluency NEVER closes a Feynman loop** — all 3 mastery conditions must hold
- **Pedagogical sycophancy is blocked architecturally** — `learn-grade` routes through sycophancy-correction
- **KB content is NEVER forwarded to external APIs** — privacy guarantee at all layers
