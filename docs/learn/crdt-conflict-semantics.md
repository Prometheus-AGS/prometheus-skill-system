# CRDT conflict semantics — learner-model

The current learner-model uses Loro-backed storage and schema `1.1.0`.
`substrate/learner-model/src/store.rs` owns normalization and derived-state folds;
`types.rs` owns persisted field shapes. The former Automerge/LWW design is
superseded and must not guide current writes.

## Evidence, not merged counters

Concept observations are keyed by stable observation ID. The fold sorts them by
timestamp and ID, then derives mastery and FSRS state from immutable priors.
Reimporting the same evidence does not add an observation merely because it arrived
from another device. Distinct observations remain distinct even when their score
and source happen to match.

Mastery stays at its prior before the fifth observation. From that observation
onward the fold applies `mastery + 0.3 * (score - mastery)`, clamped to `[0, 1]`.
Review observations advance scheduling through their recorded rating and timestamp;
the derived due date is the earliest review-derived date, or the prior due date
when no review exists. Repetition and lapse counts cannot fall below their priors.
Do not write independent mastery or FSRS counters as competing replicated facts.

Sessions are deduplicated by session ID, keeping the record with the later
`ended_at`, then sorted by start time and ID. Gaps remain keyed by gap ID; load,
save and import normalize key/record identity.

## Recoverable migration

Older observation arrays become keyed evidence. Missing legacy IDs are derived
from canonical content and position. Before a migrated document replaces the old
snapshot, the store retains its original bytes at
`learner/<learner-id>/migrations/pre-1.1-<blake3>.loro`. Keep these backups and
verify the actual import/replay boundary before claiming convergence on an update.

See [Loro evidence and migration](/docs/knowledge-learning/loro-evidence-and-migration)
for the reader-facing reference. These are source contracts; existing unit or
property checks do not certify the current phase. Run the appropriate local
integration only after all production changes are complete.
