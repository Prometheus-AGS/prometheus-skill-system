---
id: learner-model
title: learner-model
---

# learner-model

`learner-model` stores learning evidence in Loro 1.13. Observations and reviews
are immutable records with unique IDs; mastery, evidence counters, FSRS cards,
and due dates are deterministic folds, not independently merged mutable fields.

## Why evidence is the authority

Two peers may import the same evidence in different orders. Recomputing from a
stable `(concept, evidence-id)` order makes the result commutative, associative,
and idempotent. Conservative merge rules keep evidence counters non-decreasing
and select the earliest due date when schedules disagree.

Mastery uses the configured PFA-style fold after sufficient evidence. The exact
derived value can change when new evidence arrives, but importing an already-seen
evidence ID cannot increment counters or move a card again.

## Migration

Legacy Loro snapshots are read into the new evidence model and written to a new
snapshot. The original snapshot is preserved. Certification checks semantic
continuity, evidence uniqueness, deterministic refolding, and conservative due
dates before the migrated pointer can become current.

The worker and Sovereign Sync import path both call the same fold after local
writes or remote Loro updates.

*Canonical source: [`substrate/learner-model`](https://github.com/Prometheus-AGS/prometheus-skill-system/tree/main/substrate/learner-model).*
