---
license: MIT
name: kbd-apply
version: '1.0.0'
description: >
  KBD-owned spec-apply driver. Wraps a spec backend (OpenSpec; Spec Kit via the
  speckit adapter) and drives it ONE task at a time, so KBD stays the source of
  truth: every task boundary fires KBD hooks, emits a plain-text position
  signal, and syncs progress.json and the waypoint. Replaces the broken pattern
  of handing the turn to a bare /opsx:apply that runs outside KBD.
metadata:
  tags: [process, orchestration, automation, openspec, spec-kit]
---

# /kbd-apply

The execute-phase work surface. `/kbd-execute` selects the backend and writes
the dispatch contract; **`/kbd-apply` walks the tasks.**

## Why this exists

Previously, after `/kbd-plan` emitted `/opsx:new`, implementation happened by
invoking bare `/opsx:apply` — an unmodified upstream OpenSpec skill that knows
nothing about KBD. It fired no KBD hooks, wrote no `progress.json`, refreshed no
waypoint. The user "got funneled into openspec and lost all connection to the
execute phase." This driver fixes that by making **KBD own the loop** and
calling the spec backend per task.

> **Hard invariant:** never invoke a backend's "do everything" command (bare
> `/opsx:apply`, `/speckit.implement` without a single-task scope). Drive one
> task at a time through this skill.

## The per-task loop (what the model does each turn)

Before `begin-task`, resolve the plan's **Task model assignments** entry using
the full phase path, change ID and backend task ID. Follow
[task selection protocol](../kbd-plan/references/task-model-selection.md):
recheck the concrete model and native or liter-llm worker route, then pass the
assignment reference, scope, working directory, tools, skills and result handoff
to the documented worker. Record explicit selections for legacy plans and any
changed recommendation. Missing prerequisites remain unresolved; never silently
substitute the native alternative. Continue other dependency-eligible tasks where
possible. Worker results return to this driver, which alone owns begin/end and
canonical completion updates. Assignments do not introduce task-level QA gates.

```sh
ROOT="$KBD_ORCHESTRATOR_ROOT"
APPLY="$ROOT/skills/kbd-apply/kbd-apply.sh"
# Select the change from DERIVED state — see "Which change" below. Never from
# the waypoint's `exactNextCommand`.
CHANGE="<see 'Which change am I on' below>"

# 1. Read the task surface (TSV: id \t done \t title)
"$APPLY" list "$CHANGE"
read -r TOTAL COMPLETE REMAINING < <("$APPLY" progress "$CHANGE")

# 2. For each NOT-done task, one per turn:
"$APPLY" begin-task "$CHANGE" "$ID" "$I" "$TOTAL" "$TITLE"
#   → on the first observed task, opens change:before
#   → opens task:before and prints the canonical change/task signals

#   <<< implement EXACTLY this one task here (edit code/docs) >>>

"$APPLY" end-task "$CHANGE" "$ID" "$I" "$TOTAL" "$TITLE"
#   → marks the task done in the backend, syncs progress.json + waypoint,
#     fires task:after, prints "Completed task <I> of <TOTAL>: <TITLE>"
#   → on the final task, closes change:after and records change progress

# 3. After ALL planned production changes are complete, run the phase's
#    production-path integration and cumulative review gates. Once they pass:
"$APPLY" verify  "$CHANGE"   # backend verify (openspec validate / /opsx:verify)
"$APPLY" archive "$CHANGE"   # backend archive (openspec archive / /opsx:archive)
```

The plain-text "Starting/Completed task i of n" lines are the **user-facing
guarantee** (see `references/per-turn-position-hook.md`); the fired hooks are
the extensibility layer (memory mirror, custom reporters, overrides).

## Task identity

The runtime keys tasks by ID. `begin-task` and `end-task` pass the backend
task ID (the OpenSpec ordinal from `list`, e.g. `1`). When `/kbd-plan` already
registered the change's tasks under other IDs (e.g. `<change>-t1`), the driver
**reuses** them: an exact ID wins, then the task with the same normalized
title (a leading `1.2 ` is ignored), then the unique task with the same
sequence. If the change has registered tasks and none matches, the driver
refuses instead of registering a duplicate. Duplicates would leave the planned
tasks pending forever, so the change could never complete. To avoid mapping
entirely, register plan tasks with the backend IDs.

## Which change am I on

In this order:

1. `current-waypoint.json` `.nextChange`, when present. It is derived from task
   state on every projection, so it follows completion.
2. Otherwise the first entry in `phases/<phase>/progress.json` `changes[]` whose
   status is not `DONE`, honouring the phase plan's order. For a child phase,
   read the child's ledger, not the parent's.

**Never select work from `exactNextCommand`.** It is a stored operator string
that only `RunInitialized`, a checkpoint, a plan revision or an explicit
`set_active_path` rewrites. Completing a change does not touch it. Measured on a
live project 2026-09-18: it named change 01 while changes 01 through 05 were
DONE and 06 was in progress — a 169-revision drift. An agent that followed it
would re-apply finished work. Runtimes that predate `nextChange` show the same
staleness in `.change`, which is why rule 2 exists.

Treat `exactNextCommand` as the operator's stated intent — useful context for
*why* this phase is running, never an answer to *what to run next*.

## Subcommands

| Command | Effect |
|---|---|
| `detect [dir]` | print backend id: `openspec`, `speckit`, or empty |
| `list <change>` | tasks as TSV `id⇥done⇥title` |
| `progress <change>` | `total complete remaining` |
| `begin-task <change> <id> <i> <n> <title>` | open a missing `change:before`, then fire `task:before` + position signals |
| `end-task <change> <id> <i> <n> <title>` | mark done + sync + close `task:after`; the final task also closes `change:after` |
| `mark-done <change> <id>` | flip one task done AND sync the ledger (runtime transition + progress.json); no hooks, and it prints that none fired |
| `reconcile [<phase>] [--repair] [--json]` | compare each change's backend done flags (`tasks.json` / `tasks.md`) with the canonical ledger; one line per drifted task; exit 0 clean, 1 drift, 2 invalid/incomplete. `--repair` repairs only unambiguous active-phase tasks through canonical boundaries and re-checks |
| `verify <change>` | backend verify; non-zero exit = fail |
| `archive <change>` | backend archive |

### Ledger reconciliation

`mark-done` used to flip only the backend flag, so a phase could show every
`tasks.json` done while the ledger counted 13 of 25. It now syncs the ledger like
`end-task` (minus hooks), and `reconcile` is the check that proves backend and
ledger agree. Inputs are read-only: canonical authority replayed by the installed runtime in an isolated temporary snapshot, backend task files, and phase progress counters. The scan never starts backend initialization, migrates native tasks, fires lifecycle hooks, or writes live projections. Dated archived artifacts are supported; ambiguous archives are errors.

Exit **0** means the complete scan is clean; **1** means drift; **2** means invalid input or incomplete inspection. JSON preserves `phase`, `clean`, `drifted`, `drift` and adds `errors`; diagnostics stay off JSON stdout. Never infer clean from missing files, unavailable runtime state, or a failed scan.

Drift includes backend-ahead, ledger-ahead, unmappable or ledger-only identities, cancellations, archived discrepancies, and projection counts. `--repair` requires the selected phase to be active and only closes unambiguous, active backend-complete tasks through canonical boundaries. It preserves cancellations and archived history, stops after a failed transition, and rescans actual state. Legacy progress counts may be repaired; runtime-owned projections cannot be edited directly. Repeat repair is idempotent. The default selects the actual active nested phase; an explicit phase selects that phase's changes.

Run reconciliation before Reflect and at delivery-cadence refresh boundaries.

## Backends

Three spec engines are supported; **`openspec` is the default**:

| Engine | Artifacts | Adapter |
|---|---|---|
| `openspec` (default) | `openspec/changes/<change>/{proposal.md,tasks.md}` | `os_*` in `kbd-apply.sh` |
| `speckit` (GitHub Spec Kit v1.x) | `.specify/` + `specs/<change>/{spec.md,plan.md,tasks.md}` | `sk_*` in `kbd-apply.sh` |
| `native-kbd` | `.kbd-orchestrator/changes/<change>/tasks.json` | `nk_*` in `kbd-apply.sh` |

### Detection and pinning

Backend resolution is change-scoped first, then repo-wide:

1. `.kbd-orchestrator/project.json` `specBackend` pins the backend
   (`"openspec"`, `"speckit"`, `"native-kbd"`, or `"auto"`). A pin always wins.
2. Change-scoped: `.kbd-orchestrator/changes/<c>/{tasks.json,change.md}` →
   `native-kbd`; `openspec/changes/<c>/{proposal.md,tasks.md}` → `openspec`;
   `specs/<c>/{tasks.md,spec.md,plan.md}` → `speckit`.
3. Repo-wide: `openspec/` dir → `openspec`; `.specify/` dir or
   `specs/*/tasks.md` → `speckit`; `.kbd-orchestrator/changes/*/tasks.json|change.md`
   → `native-kbd`.

Engine metadata (CLI, pinned versions, adapter pointers) lives in
[`config/spec-engines.json`](../../../../config/spec-engines.json); the full
guide — including the adapter contract and how to add or update an engine — is
[`docs/guide/spec-engines.md`](../../../../docs/guide/spec-engines.md).
See `references/spec-backend-interface.md` for the full `SpecBackend` contract
and the verified OpenSpec command/JSON mappings.

## Progress Signals (MANDATORY)

```
Starting kbd-apply — <change>
Completed kbd-apply — <change> (<n>/<n> tasks, verified + archived)
```

Per-task `Starting/Completed task <i> of <n>: <title>` signals are emitted by
the driver's `begin-task`/`end-task` — relay them verbatim to the user.

## Relationship to other skills

- `/kbd-execute` — selects backend, writes `execution.md`, then defers task
  execution to this skill.
- `/kbd-reflect` — consumes the per-task `progress.json` this driver maintains.
- Child loops (`/kbd-new-child`, `/kbd-next-child`) use this **same** driver, so
  nested phases get identical per-task reporting (see change-006).
