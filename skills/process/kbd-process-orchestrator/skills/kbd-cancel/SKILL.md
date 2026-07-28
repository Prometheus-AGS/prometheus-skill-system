---
license: MIT
name: kbd-cancel
version: '1.0.0'
argument-hint: '<reason>'
description: >
  Gracefully cancel the active KBD run while preserving its checkpoint and
  immutable audit history.
metadata:
  tags: [process, orchestration, control, cancel]
---

# /kbd-cancel

Cancel the active run as an explicit terminal operator action.

## Progress Signals (MANDATORY)

Before cancellation, emit:

```text
Starting kbd-cancel — <phase-name>
```

After durable cancellation, emit:

```text
Completed kbd-cancel — <phase-name> cancelled
```

## Procedure

1. Require a non-empty cancellation reason.
2. Run `prometheus kbd cancel --reason "<reason>"`.
3. If the runtime is unavailable, create `.kbd-orchestrator/PAUSE` first and
   atomically set the waypoint status to `cancelled`, recording the prior
   status, reason, actor, and timestamp.
4. Preserve all checkpoints and dirty-work metadata. Release any owned lease.
5. Invoke a host-native cancel operation when the harness exposes one.

Cancellation is terminal. Resuming it requires a new run, never an in-place
status edit.
