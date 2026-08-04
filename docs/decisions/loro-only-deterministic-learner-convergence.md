# Decision: converge learner state from immutable Loro evidence

**Status:** accepted · 2026-08-03 · release 1.7.0

## Context

Merging mutable mastery values and scheduler fields makes results depend on
import order. Mixed Automerge/Loro descriptions also obscured the actual storage
contract.

## Decision

Loro is the only learner CRDT. Observations and reviews are immutable,
uniquely keyed evidence. After local writes and remote imports, mastery,
evidence counters, and scheduling state are deterministically refolded from the
same ordered evidence set. Conservative invariants include non-decreasing
counters and earliest due date. Migration writes a new Loro snapshot and
preserves the original.

## Alternatives considered

- Last-writer-wins derived state was rejected because clocks and import order
  change learning outcomes.
- Maintaining two CRDT engines was rejected because parity cannot be proven
  cheaply.
- Deleting migrated snapshots was rejected because rollback evidence matters.

## Consequences

Imports perform a fold and evidence consumes additional storage. In return,
commutativity, associativity, idempotency, and migration continuity are testable.

## Verification

Property tests permute writes/imports and assert identical mastery, counters,
cards, and due dates. Migration tests compare semantic state and retain the
source snapshot.
