---
id: learner-model
title: learner-model
---

# learner-model

The authoritative runtime for learner state in the Feynman loop: a Loro 1.13
CRDT document tracking per-concept mastery, FSRS-6 spaced-repetition cards,
and gap records, exposed over a JSON-RPC stdin/stdout interface.

Mastery updates use PFA: `mastery_new = mastery_old + 0.3 × (score −
mastery_old)` once a concept has ≥5 observations. The simplified FSRS-6
scheduler decides when `learn-retain` should re-test a concept.

*Canonical source: [`substrate/learner-model`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/substrate/learner-model).*
