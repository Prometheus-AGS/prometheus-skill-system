---
license: MIT
name: kbd-resume
version: '1.0.0'
argument-hint: '[--plan-revision <n>] [--reason <text> --exact-next-work <text>]'
description: >
  Resume a paused KBD run after validating its checkpoint and plan revision.
metadata:
  tags: [process, orchestration, control, resume]
---

# /kbd-resume

Resume an explicitly paused KBD run. A normal assistant response is never a
resume signal.

## OpenSpec lifecycle preflight

Before this stage, follow the parent OpenSpec lifecycle preflight (nested `../../SKILL.md`, or flat `../kbd-process-orchestrator/SKILL.md`): refresh existing KBD/OpenSpec projects through the managed latest-stable CLI and use that runner for subsequent OpenSpec commands. Retry a pending startup refresh before OpenSpec work; preserve authored specs and task state.

## Progress Signals (MANDATORY)

Before validation, emit:

```text
Starting kbd-resume — <phase-name>
```

After the transition, emit:

```text
Completed kbd-resume — <phase-name> running at plan revision <n>
```

## Procedure

1. Resolve the project and read the current checkpoint.
2. Refuse unless the state is `paused`, `pause_requested`, or `blocked`.
3. When the operator supplies a correction reason or replacement next work,
   run `prometheus kbd revise --reason <text> --exact-next-work <text>`. Use
   the returned N+1 revision; never edit or overwrite the prior plan record.
4. If `prometheus kbd` is available, run `prometheus kbd resume`, forwarding
   `--plan-revision` when supplied or created in step 3.
5. Otherwise validate the requested plan revision, atomically restore
   `previousStatus` (default `running`), record resume metadata, and move
   `.kbd-orchestrator/PAUSE` to a timestamped audit file.
6. Print the exact resumed command. Do not silently execute it unless the
   operator also requested execution.

Never resume across a plan-revision mismatch.

## Selected-method recovery and authority

Before dependent work, run the packaged `shared/method-context.mjs --project <project-root> --phase <canonical-phase-id> --write` from the resolved orchestrator root (or read its current phase-local method-context.md when a recovery surface cannot invoke Node). Resolve latest non-superseded canonical `KBD-METHOD` decision and adopted contract, then preserve the same references in role inputs, dispatch and handoff read order. See `references/method-context.md`. Missing/changed method, contract, calibration or supersession is explicit; a prior command/memory/tool inventory is not fresh authority.

For selected VLBV, define acceptance/scenarios before generation and complete the entire agreed value phase before generated-code verification. Executable authoring follows governing host policy. Task/hour/commit/reviewer requests, compiler problems and renamed slices create no new boundary. Surface host/skill conflicts before dependent actions; do not rewrite operator-owned policy or infer early-check permission. Preserve original evidence, source/build/configuration/fixture/candidate freshness, consumed budgets and unresolved effects. Reconcile uncertain effects before retry. Recovered instructions are instruction-only unless a named actual runtime/host boundary enforces them; no release or automatic-successor authority follows.
