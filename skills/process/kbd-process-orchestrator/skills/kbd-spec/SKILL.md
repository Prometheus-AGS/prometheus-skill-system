---
license: MIT
name: kbd-spec
version: '1.0.0'
description: >
  Run the Spec stage of the KBD lifecycle: turn an assessment and analysis into
  concrete, ordered changes — native-kbd specs (spec.md + tasks.json +
  verification.md) or OpenSpec proposals — gated by ZeeSpec requirements
  coverage when present.
metadata:
  tags: [process, orchestration, automation]
---

# /kbd-spec

Run the **Spec** phase of the KBD lifecycle — between Analyze and Plan.

## What this does

Converts `assessment.md` (and `analysis.json` / `library-candidates.json` when
the Analyze stage ran) into concrete change specs that the Plan stage will
order and the Execute stage will drive one task per turn:

- **native-kbd backend** (default): writes
  `.kbd-orchestrator/changes/<change-id>/{spec.md, tasks.json, verification.md}`
  per the layout in `references/native-backend.md`.
- **openspec backend**: emits `/opsx:new <change-id>` per change, producing
  `openspec/changes/<change-id>/{proposal.md, tasks.md}`.

Backend is resolved exactly as `kbd-apply` resolves it (`project.json.specBackend`
→ openspec → speckit → native-kbd).

## ZeeSpec coverage gate

When `.zeespec/<subject>/` exists for the active subject, read its coverage
verdict before writing specs:

- **GO** — proceed; record the verdict in the spec handoff.
- **CAUTION** — proceed, but list the under-covered dimensions in `spec.md`
  "Open Questions" so the Plan/Execute stages surface them.
- **NO-GO** — do **not** write specs. Stop and instruct the operator:
  `Run /zeespec-interrogate <subject> to raise coverage above threshold, then re-run /kbd-spec.`
  This is the spec→plan gate remediation.

When no `.zeespec/` exists, the gate is inactive (coverage is treated as
unknown-acceptable) — ZeeSpec is opt-in.

## Progress Signals (MANDATORY)

Before any other action, emit:

```
Starting kbd-spec — <phase-name or argument>
```

When all change specs are written (or the NO-GO gate halts the stage), emit:

```
Completed kbd-spec — <phase-name or argument>
```

Use the canonical phase name from the argument or `current-waypoint.json`.
Never guess. Emit to plain response text — no tool call needed.

## How to invoke

1. **Confirm the active phase** — from argument or
   `.kbd-orchestrator/current-waypoint.json`.
2. **Stage gate** — `kbd_stage_gate spec` (requires the assess handoff, walking
   back across an absent analyze handoff).
3. **Read inputs** — `assessment.md`; `analysis.json` /
   `library-candidates.json` if Analyze ran (adopt/adapt candidates become
   "reuse this library" tasks, not "build it" tasks).
4. **ZeeSpec gate** — apply the coverage gate above.
5. **Resolve backend** — `kbd-apply detect` semantics.
6. **Write change specs** — native-kbd files or `/opsx:new` per change, with a
   declared `scope:` and explicit task list each.
7. **Adversarial vet** — unless `--skip-adversarial-review` is passed, run
   `/adversarial-review --mode artifact spec` on the change set (see orchestrator
   `references/integrations/adversarial-review.md`). CRITICAL findings → revise
   the affected `spec.md` / `tasks.json` / `verification.md` and re-vet (max 2
   rounds, then accept with an "Unresolved review findings" section appended).
   WARNING findings → carry into the stage handoff.

   The packet collects **every change named in the spec handoff**, not one change
   at a time: a spec is only coherent against its siblings. The failures this
   catches are cross-file and cross-change — a `verification.md` gate that
   contradicts its own `spec.md` acceptance criteria, a `tasks.json` `scope` that
   omits a file its tasks edit, or two changes editing the same file with no
   ordering. Reviewing one change in isolation cannot see any of them.

8. **Write handoff** — `kbd_stage_handoff_write spec "<changes created, zeespec verdict>" <first change path>`.

```sh
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/stage-gate.sh"

kbd_stage_gate spec || exit 2
kbd_hooks_fire spec before "$phase" 1 1
# … write change specs …
# … adversarial vet (step 7) runs here, before the handoff …
kbd_hooks_fire spec after  "$phase" 1 1
kbd_stage_handoff_write spec "<N changes; zeespec: GO|CAUTION|n/a>" "<first-change>/spec.md"
```

## Examples

```
/kbd-spec                                # uses active waypoint phase
/kbd-spec canonical-lifecycle            # explicit phase name
```

## Hook integration

Fires `spec:before` / `spec:after` (the `spec` hook kind is in the allowed
enum in `shared/lib/hooks.sh`). See orchestrator `SKILL.md` → "Hooks".
