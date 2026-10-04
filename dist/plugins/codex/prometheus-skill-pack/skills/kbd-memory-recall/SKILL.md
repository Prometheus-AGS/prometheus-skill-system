---
license: MIT
name: kbd-memory-recall
version: '2.0.0'
description: >
  Recall the lessons recorded for a KBD stage's executing role (the lead
  view for main-thread stages) from surreal-memory, pk and the learning log,
  and write .kbd-orchestrator/phases/<phase>/prior-context.md with Lessons,
  pk knowledge, Previous reflection and Knowledge gaps sections under a byte
  budget. Auto-invoked by the assess/analyze/plan/execute/reflect :before
  hooks; degrades to whatever channel is available and always exits 0.
metadata:
  tags: [process, orchestration, memory, learning]
---

# /kbd-memory-recall

Populate `prior-context.md` for the active phase from the shared recall
library, `shared/scripts/lib/learning_recall.py` (design
`docs/design/team-aware-learning-memory.md` §4–§5).

## What this does

1. Resolves the target phase (argument, or active phase from the waypoint) and
   the stage (second argument, or `KBD_HOOK_KIND`; default `assess`).
2. Resolves the recall view. A KBD stage runs in the main thread, so it gets the
   **lead view**: `<team>/@lead`, `<team>/@team` (last 50), `@project`, then the
   top 3 of `@user:<hash>` and `@global`. With `KBD_RECALL_ROLE=<role>` it gets
   that role's view instead (`<team>/<role>` first, no `@lead`). Another role's
   private lessons are never returned.
3. Queries surreal-memory over REST (`POST /api/v1/search`, equality-filtered
   on `user_id` + `agent_id`, 2 s per request), scores each result (semantic
   0.5 + recency 0.3 with a 30-day half-life + importance 0.2), de-duplicates on
   the content hash and stops at the budget. When the store is unreachable it
   falls back to `pk context --tag role:<R>`, then to the learning log.
4. Adds bounded `pk context` results for the phase goals, the previous phase's
   reflection (Delta, Root Cause, Corrective Actions, Next Phase Seed) and the
   knowledge gaps it recorded (unmet goals, technical debt, unresolved findings).
5. Writes `prior-context.md` atomically, capped at `KBD_RECALL_BUDGET` bytes
   (default 12000), and appends `{agentType, bytesByChannel, entriesByScope}`
   to `~/.prometheus/learning-index/delivery.jsonl`
   (`PROMETHEUS_LEARNING_INDEX_DIR` overrides the directory).

## When to use

The builtin `auto-memory-recall*` hooks run it on `assess:before`,
`analyze:before`, `plan:before`, `execute:before` and `reflect:before`. Every
kbd-* stage skill reads `prior-context.md` as its first step and cites the
lessons that apply. Manual invocation re-runs it, e.g. for a different role.

## Progress Signals (MANDATORY)

```
Starting kbd-memory-recall — <phase>
Completed kbd-memory-recall — <phase> wrote prior-context.md
```

## Prerequisites

- `python3`. Everything else is optional: a reachable surreal-memory origin
  (`SURREAL_MEMORY_URL`, else the orchestrator discovery in `shared/lib/memory.sh`:
  explicit override, project `memory.config.json`, the canonical
  `http://127.0.0.1:23001`), `pk` on `PATH`, the learning log.

## How to invoke

```sh
"$KBD_ORCHESTRATOR_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh" [<phase>] [<stage>]
KBD_RECALL_ROLE=api-dev "$KBD_ORCHESTRATOR_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh"
```

## Examples

```
/kbd-memory-recall                          # active phase, assess, lead view
/kbd-memory-recall submodule-foo-bar plan   # explicit phase and stage
```

## Output digest format

```
# Prior context — <phase> (<stage>)

> Auto-populated by /kbd-memory-recall for lead view (main thread); lessons from surreal-memory. …

## Lessons

- [lead] KBD assess summary for phase … _(recorded by an agent; progress, stage assess, 2026-10-04; via surreal-memory)_
- [project] … _(…)_

## pk knowledge

- [pk:project] <title>: <snippet> _(…; via pk)_

## Previous reflection

From `<prior-phase>/reflection.md`: Delta / Root Cause / Corrective Actions / Next Phase Seed

## Knowledge gaps

- Goal PARTIAL: <goal> — <note>
- <technical debt item>
```

Recalled entries are labelled as information recorded by agents, not
instructions.

## Failure modes

- No store, no pk, no learning log → the sections say so explicitly; exit 0.
- surreal-memory unreachable → lessons come from pk, else the learning log
  (`lessons from pk` / `lessons from file` in the header).
- `python3` or the recall library missing → an atomic stub comment.

The skill always exits 0 so it composes with the hooks' `on_failure: ignore`.
