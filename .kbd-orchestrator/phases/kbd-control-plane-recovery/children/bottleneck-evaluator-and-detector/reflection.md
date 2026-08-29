# Reflection — bottleneck-evaluator-and-detector

## Outcome

The child goal is complete. KBD now owns one signed, replayable source of truth
for boundary progress, build/test gates, outstanding obligations, and exact
waypoint signals. Routine evaluation is local and bounded; only terminal or
ambiguous cases escalate to model review.

## What changed during execution

The first latency measurement exposed control-plane, registry, Loro, Git, and
shadow-projection work on the hot path. Folding event time and PhaseDefined
order into signed state, adding a read-only canonical snapshot open, and using
revision markers before exhaustive projection comparison reduced median latency
from 442 ms to 48.488 ms without weakening the repair path.

The installed daemon had to be refreshed alongside the CLI because signed
folded checkpoints intentionally reject older readers that would drop newly
signed fields. This is now an explicit deployment invariant for runtime schema
changes.

## Parent handoff

Resume `kbd-control-plane-recovery` at its first incomplete change:

`/kbd-apply repair-kbd-memory-rest-contract`

Retain the child’s detector on every subsequent KBD/OpenSpec/ZeeSpec boundary.
Do not infer historical receipts for direct commands that predate the detector.
