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

Before dependent work, run the packaged `shared/method-context.mjs --project <project-root> --phase <canonical-phase-id> --write` from the resolved orchestrator root (or read its current phase-local method-context.md when a recovery surface cannot invoke Node). Resolve latest non-superseded canonical `KBD-METHOD` decision and adopted contract, then preserve the same references in role inputs, dispatch and handoff read order. See `references/method-context.md`. Missing/changed method, contract, calibration or supersession is explicit; a prior command/memory/tool inventory is not fresh authority.

For selected VLBV, define acceptance/scenarios before generation and complete the entire agreed value phase before generated-code verification. Executable authoring follows governing host policy. Task/hour/commit/reviewer requests, compiler problems and renamed slices create no new boundary. Surface host/skill conflicts before dependent actions; do not rewrite operator-owned policy or infer early-check permission. Preserve original evidence, source/build/configuration/fixture/candidate freshness, consumed budgets and unresolved effects. Reconcile uncertain effects before retry. Recovered instructions are instruction-only unless a named actual runtime/host boundary enforces them; no release or automatic-successor authority follows.
