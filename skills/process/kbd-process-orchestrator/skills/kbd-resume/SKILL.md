---
license: MIT
name: kbd-resume
argument-hint: '[--plan-revision <n>] [--reason <text> --exact-next-work <text>]'
description: >
  Resume a paused KBD run after validating its checkpoint and plan revision.
metadata:
  version: '1.0.0'
  tags: "process, orchestration, control, resume"
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
- `BLOCKED` — the contract, its binding, a calibration reference, a supersession or a deviation's cadence/reason is missing or changed; resolve it explicitly. Prior commands, memory and tool inventories are not fresh authority.
- `CONFLICT` — a governing rule file the active harness loads grants an exception VLBV forbids (for example a compiler check during generation). Stop before dependent work and show the conflict; continue only after the operator records a `KBD-POLICY-ACK` decision for that file's current hash. Never rewrite operator-owned policy, and never treat an exception as authority for tests, builds, dry-runs or generated-code verification.

VLBV is the method for every phase (project default `verificationMethod: "VLBV"`); a deviation exists only as a signed `KBD-METHOD` decision naming the method, its execution cadence and its reason, and it is disclosed in the closure packet. Define acceptance before generation; generated code executes only at the value-phase boundary (B4) and, after an authorized repair, at B5 — see `references/boundaries.md`. Tasks, hours, commits, reviewer requests and compiler problems create no boundary. Preserve original evidence, consumed budgets and unresolved effects across resume and supersession; reconcile an uncertain effect before any retry. Full rules: `references/method-context.md`.

