# Reflection — phase-demo-epsilon (honest test fixture)

## Goal Achievement

- G1: NOT MET — the data pipeline was not completed; ingestion works but
  the transformation step produces NaN for 12% of records in the test dataset
- G2: MET — real-time sync verified across two nodes, CRDT merge correct
- G3: NOT MET — no tests were written for the transformation layer (G1's gap)

## Delta

G1 is in a worse state than at phase start. At the beginning of the phase, the
transformation step did not exist. Now it exists but introduces a data corruption
bug: records with null `timestamp` fields divide by zero in the normalization
formula. The formula uses `(value - min) / (max - min)` without a guard for
`max == min`, and 12% of the test records have identical min and max.

This was introduced in change-extval-003 and not caught because change-extval-003
had no tests.

## Root Cause

The missing guard is a calculation bug, not a design flaw. The transformation
algorithm is correct in concept. The specific failure mode (`max == min`) was
not considered during implementation. No tests existed to catch it because tests
were deferred to "after the happy path works."

This is a direct instance of the anti-pattern this project prohibits: code written
before tests.

## Corrective Actions

1. Fix the NaN bug in the transformation formula: add a guard `if max == min { return 0.0 }`.
2. Add a test for the `max == min` case before merging the fix.
3. Apply the TDD protocol strictly to all remaining G1 work — tests first, then implementation.

## Recommended Next Phase

Replay the transformation layer under TDD. The bug is a one-line fix but the
test gap is structural — do not ship without coverage of the edge cases.
