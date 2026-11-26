# Outcome-first phases

An outcome-first phase is scoped to **one** smallest independently useful user
outcome. It reaches its single boundary run (B4) as soon as that outcome is
built, so unrelated or speculative work cannot hold delivery back.

It is **opt-in per new phase**. Phases created without the flag, and every
phase that already exists, behave exactly as before. Nothing here adds a
boundary, a gate or a test suite: the phase still has one B4 run, defined in
[boundaries.md](boundaries.md).

## Start one

```bash
bash skills/kbd-new-phase/kbd-new-phase.sh <name> \
  --outcome-first --outcome "<the one user outcome this phase must prove>"
```

The script passes the flags to `prometheus kbd phase create`. It needs the
canonical runtime and stops, writing nothing, when the runtime is unavailable.
`/kbd-next-phase` does not take the flag; start an outcome-first phase with
`/kbd-new-phase`.

## Assess: record the outcome

Complete the record that `goals.md` opens with. Write `UNKNOWN` where a fact is
not established.

| Field | Content |
|---|---|
| Actor | Who gets the result |
| Job | What they are trying to get done |
| Observed need or material risk | Evidence of the need, or the risk if it is absent |
| Real entry point | The shipped command, screen or API the actor uses |
| Acceptance oracle | The observation that shows it worked, and its source of truth |
| Hard dependencies | What must exist first, or none |
| Why now | Why this outranks the other eligible outcomes |

The adopted contract already holds `actor`, `usefulOutcome`, `reasonNow`,
`productionEntry` and `oracle` for each scenario; fill those from this record.
Job, need or risk, hard dependencies and rank stay in `goals.md` and
`assessment.md`. The contract schema is unchanged.

When several outcomes are eligible, rank them by:

1. operator priority, when the operator has stated one;
2. material user impact;
3. time to value, to break a tie.

Take the first. Work that is speculative and low in consequence goes to a later
phase, not onto this phase's critical path. Record the outcomes that were not
chosen, with the reason, so the choice can be reviewed.

## Plan: order and dependencies

Order changes by what the outcome needs first. Register a real dependency so
resume can act on it:

```bash
prometheus kbd change register --phase <phase> --id <change> --title "<title>" \
  --sequence <n> --depends-on <earlier-change-id>
```

`--depends-on` names a change already registered in the same phase and may be
repeated. Leave it out when a change can proceed on its own.

## Resume: what gets selected

Within an outcome-first phase the runtime selects, in ledger order:

1. the change in progress;
2. the first pending change whose dependencies are all complete or skipped;
3. a blocked change, only when nothing else is eligible.

A blocked change is not dropped. It appears in `blockedChanges` with its
blockers until it is cleared. Task order inside a change is unchanged: a
blocked task blocks its change.

`current-waypoint.json`, `position-reminder.txt` and the phase `progress.json`
carry, for an outcome-first phase only:

| Field | Meaning |
|---|---|
| `outcome` | The outcome statement given at creation |
| `lastEvidence` | The latest gate receipt for the phase, else the last completed task |
| `nextAuthorizedAction` | The next action derived from canonical state |
| `blockedChanges` | Blocked changes that were not selected |

`nextAuthorizedAction` describes position. It grants nothing: a boundary run
still needs its authorization, and a repair still needs the owner's.

## Changing priority

Ledger order holds inside an adopted phase. A different priority goes through
the paths that already exist; nothing reorders or invalidates signed work on
its own.

| Need | Path |
|---|---|
| Change what the phase must prove | `kbd-contract-adopt.sh <contract> --supersedes <decision> --amendment <manifest> --signature <sig>` |
| Stop this phase in favour of another | `prometheus kbd phase close --disposition superseded`, then a new phase |
| Note intent without changing scope | `prometheus kbd revise --reason <text> --exact-next-work <text>` |

## Pilot comparison

Before making outcome-first the default, run one phase with it and record the
comparison against a comparable earlier phase:

| Measure | Outcome-first phase | Earlier phase |
|---|---|---|
| Time from phase start to first verified outcome | | |
| Behavior reached (scenarios PASS / adopted) | | |
| Repeat attempts on the same blocker | | |
| Unauthorized actions, false stops, weakened controls | | |
