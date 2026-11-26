---
id: outcome-first-phases
title: Outcome-first phases
---

# Outcome-first phases

An outcome-first phase is scoped to one smallest independently useful user
outcome. It reaches its single boundary run as soon as that outcome is built,
so unrelated or speculative work cannot hold delivery back.

It is opt-in per new phase. Phases created without the flag, and every phase
that already exists, behave as before. It adds no boundary, gate or test suite.

## Start one

```bash
/kbd-new-phase report-export --outcome-first \
  --outcome "An operator can export a signed report from the CLI"
```

The phase's `goals.md` opens with a record to complete in Assess: actor, job,
observed need or material risk, real entry point, acceptance oracle, hard
dependencies and why now. Anything not needed to prove the outcome belongs in
another phase.

When several outcomes are eligible, rank them by operator priority, then
material user impact, and use time to value to break a tie.

## What resume selects

| Order | Selected |
|---|---|
| 1 | The change in progress |
| 2 | The first pending change whose dependencies are complete |
| 3 | A blocked change, only when nothing else is eligible |

Declare a real dependency when registering a change:

```bash
prometheus kbd change register --phase report-export --id change-b \
  --title "Sign the export" --sequence 2 --depends-on change-a
```

A blocked change stays visible. The waypoint, the position reminder and the
phase `progress.json` show:

| Field | Meaning |
|---|---|
| `outcome` | The outcome statement given at creation |
| `lastEvidence` | The latest gate receipt for the phase, else the last completed task |
| `nextAuthorizedAction` | The next action derived from canonical state |
| `blockedChanges` | Blocked changes that were not selected |

`nextAuthorizedAction` describes position and grants nothing. A boundary run
still needs its [authorization](./boundary-authorization.md).

## Changing priority

Ledger order holds inside an adopted phase. To change what the phase must
prove, adopt a signed contract amendment. To stop the phase in favour of
another, close it with the `superseded` disposition and start a new phase.
Recording intent with `prometheus kbd revise` does not reorder work.

## Limits

- `/kbd-next-phase` does not take the flag. Start an outcome-first phase with
  `/kbd-new-phase`.
- It needs the canonical KBD runtime.
- It is a pilot. Compare one outcome-first phase with a comparable earlier
  phase (time to first verified outcome, behavior reached, repeat attempts)
  before making it the default. The comparison table is in the orchestrator's
  `references/outcome-first.md`.
