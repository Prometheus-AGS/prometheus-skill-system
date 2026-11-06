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

Before dependent work, run `node shared/method-context.mjs --project <project-root> --phase <canonical-phase-id> --write` from the resolved orchestrator root (or read the phase-local `method-context.md` when Node is unavailable) and act on its `status`:

- `RESOLVED_REFERENCES` — continue with the recovered method, contract and stage authority; preserve the same references in role inputs, dispatch and handoff read order.
- `BLOCKED` — a method decision, contract, calibration or supersession is missing or changed; resolve it explicitly. Prior commands, memory and tool inventories are not fresh authority.
- `CONFLICT` — governing host policy conflicts with the selected method. Stop before dependent work and show the conflict; continue only after the operator records a `KBD-POLICY-ACK` decision for that file's current hash. Never rewrite operator-owned policy. When host policy grants an exception — for example one narrow, targeted compiler check after a blocking compile error — name it as host policy, state that it conflicts with the selected method, and say it does not authorize tests, dry-runs or generated-code verification.

For selected VLBV, define acceptance before generation and finish the whole agreed value phase before any generated-code verification; tasks, hours, commits, reviewer requests and compiler problems create no boundary. Preserve original evidence, consumed budgets and unresolved effects across resume and supersession; reconcile an uncertain effect before any retry. Full rules: `references/method-context.md`.

