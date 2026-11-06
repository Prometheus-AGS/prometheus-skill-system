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

## Selected-method recovery and authority

Before dependent work, run `node shared/method-context.mjs --project <project-root> --phase <canonical-phase-id> --write` from the resolved orchestrator root (or read the phase-local `method-context.md` when Node is unavailable) and act on its `status`:

- `RESOLVED_REFERENCES` — continue with the recovered method, contract and stage authority; preserve the same references in role inputs, dispatch and handoff read order.
- `BLOCKED` — a method decision, contract, calibration or supersession is missing or changed; resolve it explicitly. Prior commands, memory and tool inventories are not fresh authority.
- `CONFLICT` — governing host policy conflicts with the selected method. Stop before dependent work and show the conflict; continue only after the operator records a `KBD-POLICY-ACK` decision for that file's current hash. Never rewrite operator-owned policy. When host policy grants an exception — for example one narrow, targeted compiler check after a blocking compile error — name it as host policy, state that it conflicts with the selected method, and say it does not authorize tests, dry-runs or generated-code verification.

For selected VLBV, define acceptance before generation and finish the whole agreed value phase before any generated-code verification; tasks, hours, commits, reviewer requests and compiler problems create no boundary. Preserve original evidence, consumed budgets and unresolved effects across resume and supersession; reconcile an uncertain effect before any retry. Full rules: `references/method-context.md`.

