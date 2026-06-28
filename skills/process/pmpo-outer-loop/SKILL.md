---
license: MIT
name: pmpo-outer-loop
version: '1.0.0'
description: >
  Define and run a standing outer loop — a goal plus feedback sources plus
  termination criteria — that repeatedly drives PMPO/KBD cycles and only
  escalates to the human at declared decision points. The Boris Cherny shape:
  write the loop once, the framework discovers, researches, executes, and
  reports until the goal is met.
metadata:
  tags: [process, orchestration, automation]
---

# /pmpo-outer-loop

Write a loop instead of prompting each step. "I don't prompt Claude anymore; I
have loops running — my job is to write loops." You define the goal, the
feedback sources, and when to stop; the framework runs cycles and pings you only
at decision points.

This is a thin standing wrapper over the existing iterative-evolver — it adds no
new loop engine and runs no daemon. One tick = one evolver cycle (which itself
composes zeespec elicitation, kbd-analyze research, and the KBD execute loop).

## Three commands

### `/loop-define <name>`

Interactively (via `/pmpo-elicit`) build the loop definition and write it to
`.kbd-orchestrator/loops/<name>/loop.json` (schema:
`references/schemas/loop-definition.schema.json`): goal + measurable_criteria,
feedback_sources (command / gh-query / file / url), termination (max_ticks,
max_no_progress_ticks, budget), escalation_points, cadence, and the backing
`evolution_name`.

**Perspective field:** `loop.json` accepts an optional `perspective` field
(`competitive | trend | unique-product | idea-validation | self-learning | auto`,
default `"auto"`). When set, each `/loop-tick` passes `--perspective <value>` to
the pmpo-evolver, pinning that tick's evolution to the named perspective rather
than letting the strategy router decide. Use `"auto"` for normal multi-perspective
cycling; set an explicit perspective when the loop is purpose-built for a specific
research direction (e.g., `"competitive"` for a release-tracking loop).

### `/loop-tick <name>`

Run **exactly one** cycle:

1. Read `loop.json` and the last `journal.md` entry.
2. Collect each `feedback_source` and interpret it.
3. Diff against `goal.measurable_criteria`:
   - **satisfied** → terminate, write a final `/loop-report`.
   - **regression or `max_no_progress_ticks` reached** → escalate via
     `/pmpo-elicit` (continue / re-plan / stop) — a declared decision point.
     See **Stall/regression escalation** below.
   - **otherwise** → run one `/evolve "<evolution_name>"` cycle.
4. Append a `journal.md` entry + a `decision-log.md` entry (dual format).
5. If `cadence.mode == cron`, the scheduled agent re-invokes; if `background`,
   re-arm a background task; if `manual`, stop and wait for the next call.

### `/loop-report <name>`

Render `journal.md`: a dense tick table on top, a narrative per tick below
(serves both advanced and beginner readers).

## Cadence — delegated to platform primitives (no daemon)

- **manual** — you run `/loop-tick` when you choose.
- **background** — a Claude Code background task runs `claude -p "/loop-tick <name>"`.
- **cron** — a scheduled cloud agent invokes the tick on a cron expression.

This skill implements only the tick; the *cadence* is whichever platform
primitive you wire to it.

## Progress Signals (MANDATORY)

**FIRST tool call of every turn:** Read `.kbd-orchestrator/position-reminder.txt` (if it exists). If absent, read `.kbd-orchestrator/current-waypoint.json`.

Before any other action, emit to plain response text (BEFORE any tool call):

```
Starting pmpo-outer-loop — <name> tick <N>
```

When the tick (or define/report) completes, emit:

```
Completed pmpo-outer-loop — <name> tick <N> (<continue|escalate|terminate>)
```

Emit to plain response text — no tool call needed.

## Loop Definition Reference

Full schema: [`references/schemas/loop-definition.schema.json`](references/schemas/loop-definition.schema.json)

Human-readable guide: [`references/loop-schema.md`](references/loop-schema.md)

## Operational Script

[`scripts/loop-tick.sh`](scripts/loop-tick.sh) — bash implementation of a tick:
- reads `loop.json`, evaluates feedback sources
- increments `current_tick`, appends `journal.md`
- exits 0=continue, 1=escalate, 2=terminate

## Termination & escalation guards

The loop never runs unbounded: `max_ticks` is a hard ceiling,
`max_no_progress_ticks` (default 2) escalates on a stall, and `budget`
bounds per-tick wall time. Every escalation routes through `/pmpo-elicit`, so
the human is consulted with a concrete decision, never left guessing.

See `skills/process/pmpo-elicit/references/escalation-points.md` for the
platform routing table and async checkpoint contract.

### Stall/regression escalation — operative protocol

When `max_no_progress_ticks` is reached or a regression is detected:

1. Construct the elicitation:
   - `question`: "Loop `<name>` stalled after `<N>` ticks with no progress. How should we proceed?"
   - `options`: ["Continue — run another tick", "Re-plan — revise the evolution goal", "Stop — terminate loop and write final report"]
   - `context`: "Last tick delta vs measurable_criteria: `<diff>`"
   - `criticality`: blocking
   - `caller`: pmpo-outer-loop/loop-tick

2. **On Claude Code** — use `AskUserQuestion` with the three options. Record choice
   in `loops/<name>/journal.md` latest entry as `escalation_result`.

3. **On other platforms** — call `pmpo-elicit-checkpoint.sh`:
   ```
   bash "${CLAUDE_PLUGIN_ROOT}/skills/process/pmpo-elicit/scripts/pmpo-elicit-checkpoint.sh" \
     ".kbd-orchestrator/loops/<name>/elicitations/pmpo-outer-loop-$(date +%s)" \
     "Loop '<name>' stalled after <N> ticks. How to proceed?" \
     "blocking" "pmpo-outer-loop/loop-tick" \
     "continue" "replan" "stop"
   ```
   Set `loop.json → status = "paused-escalation"`. On resume, call `pmpo-elicit-resume.sh`.

4. Outcome logic:
   - "Continue": reset `no_progress_ticks` to 0, run next tick.
   - "Re-plan": invoke `/pmpo-elicit` again for the new goal text, update `loop.json → goal`,
     reset counters, continue.
   - "Stop": write final `/loop-report`, set `loop.json → status = "terminated-by-operator"`.

Record in `decision-log.md`:
```
### <timestamp> — Loop stall escalation
Loop: <name> | Ticks stalled: <N>
Decision: <continue|replan|stop> | Provenance: <user|implicit>
Elicitation ID: <id>
```

## Examples

```
/loop-define ship-auth      # interactively define the loop
/loop-tick ship-auth        # run one cycle
/loop-report ship-auth      # see where it stands
```

## Relationship to the rest of PMPO

`/loop-tick` → `/evolve` (one cycle) → which runs zeespec (when
under-constrained), kbd-analyze (research), and the KBD assess→…→reflect loop.
Escalations and missing information route through `/pmpo-elicit`. The loop is
the outermost layer; everything else composes beneath it.
