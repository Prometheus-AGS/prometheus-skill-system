## Why

KBD cancellation is terminal, but the runtime has no supported way to begin a successor run for the same immutable project. Operators are therefore forced to leave a stale terminal waypoint or reconstruct canonical storage outside the signed command path.

## What Changes

- Add an operator-signed command for starting a successor run from a terminal lifecycle.
- Preserve the prior run's immutable audit and project authority while resetting run-scoped workflow state.
- Teach causal folding to distinguish ordered successor runs from concurrent rollover conflicts.
- Add `prometheus kbd run start` and make `/kbd-new-phase` use it when the current run is terminal.
- Release the local PAUSE valve only after the rollover event and compatibility projections commit.

## Capabilities

### New Capabilities

- `kbd-run-lifecycle`: Defines safe successor-run creation, audit continuity, conflict behavior, and CLI/skill integration.

### Modified Capabilities

None.

## Impact

This changes the public KBD command and event contracts in `kbd-runtime`, the `prometheus` CLI, compatibility projections, the Sovereign Sync consumer, and the `kbd-new-phase` skill. Existing journals remain readable and unchanged.
