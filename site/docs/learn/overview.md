---
id: overview
title: Learn Domain Overview
sidebar_label: Overview
---

# Learn Domain — Feynman-Spine

The Learn domain supplies a **Feynman-Spine** learning procedure with goal,
corpus, explanation, practice and retention artifacts. Skill requests require the
selected harness and their actual collaborators; source instructions alone do not
certify learning outcomes or credential issuance.

## The learning arc

```
/learn-goal       → set what you want to learn
/learn-survey     → diagnostic placement + recursion floor
/learn-plan       → concept DAG + curriculum builder
/feynman-loop     → core Feynman PMPO loop (explain, gap-find, re-study)
/learn-grade      → corpus-grounded grading with disclosed screening
/learn-retain     → FSRS-6 spaced retrieval
/learn-practice   → deliberate practice track
/learn-certify    → local self-issued credential procedure
```

## Support skills

| Skill | Purpose |
|-------|---------|
| `/learn-kb` | KB registry + adapter management |
| `/learn-about-system` | Prometheus stack meta-learning |
| `/learn-harness` | Harness detection + capability map |
| `/ui-surface` | Cross-harness UI rendering primitive |

The optional `/sync-status`, `/sync-peers`, and `/sync-push` skills are
distributed by `prometheus-companion`, which owns cross-machine replication.

## Core invariants

- **Self-reported fluency NEVER closes a Feynman loop** — all 3 mastery conditions must hold
- **Grading needs evidence** — sycophancy screening is a bounded diagnostic and does not guarantee correctness or an independent model
- **KB destinations are explicit** — local reads stay local, while configured Dify/Palace and downstream inference destinations require separate ownership and authorization
