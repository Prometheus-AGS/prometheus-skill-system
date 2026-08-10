---
license: MIT
name: kbd-pause
version: '1.0.0'
argument-hint: '<reason>'
description: >
  Gracefully pause the active KBD run, checkpoint its exact position, and
  prevent every harness from steering execution until an operator resumes it.
metadata:
  tags: [process, orchestration, control, pause]
---

# /kbd-pause

Pause the active KBD run. Operator intent always outranks agent continuation.

## Progress Signals (MANDATORY)

Before acting, emit:

```text
Starting kbd-pause — <phase-name>
```

After the checkpoint is durable, emit:

```text
Completed kbd-pause — <phase-name> paused
```

## Procedure

1. Resolve the project containing `.kbd-orchestrator/`.
2. Require a non-empty reason from the arguments or the operator.
3. If `prometheus kbd` is available, run:
   `prometheus kbd pause --reason "<reason>"`.
4. Otherwise create `.kbd-orchestrator/PAUSE` first, then atomically update the
   waypoint to `status: "paused"`, preserving its prior status as
   `previousStatus` and recording `pauseReason`, `pausedAt`, and `pausedBy`.
5. Report the last completed work, exact next work, dirty-work summary, and
   current plan revision. Do not execute the next command.

Never remove `PAUSE`, resume work, or reinterpret ordinary prose as a resume.
