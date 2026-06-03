# SpecBackend interface

A `SpecBackend` is the contract that lets `kbd-apply` (the KBD-owned execute
driver) wrap **any** spec-driven tool task-by-task without forking the loop.
KBD owns the loop; the backend is a subroutine that answers four questions and
performs two mutations.

> **Hard invariant:** `kbd-apply` NEVER hands the turn to a backend's own
> "implement everything" command (e.g. bare `/opsx:apply` or
> `/speckit.implement` with no task scope). Those commands run outside KBD and
> are exactly the seam this phase repairs. KBD calls the backend per task.

## Operations

| Op | Returns / Effect | Purpose |
|---|---|---|
| `detect()` | backend id or empty | is this backend usable in the cwd? |
| `list_tasks(change)` | `[{id, title, done}]` | the working task surface |
| `progress(change)` | `{total, complete, remaining}` | counts for the position signal |
| `mark_done(change, id)` | mutation | flip one task to complete |
| `verify(change)` | pass/fail + report | post-change quality/spec check |
| `archive(change)` | mutation | retire the completed change |

`list_tasks` task shape is normalized to **exactly** `{id, title, done}` so the
driver is backend-agnostic. Adapters map their native fields into this shape.

---

## Adapter: OpenSpec  (verified against `openspec` CLI, v-on-PATH, 2026-06-03)

| Op | Concrete command | Mapping |
|---|---|---|
| `detect` | `[ -d openspec ] && command -v openspec` | dir + CLI both present |
| `list_tasks` | `openspec instructions apply --change <c> --json` | `.tasks[] → {id, title:.description, done}` |
| `progress` | same call, read `.progress` | `{total, complete, remaining}` (verified keys) |
| `mark_done` | edit the task checkbox in `openspec/changes/<c>/tasks.md` (`- [ ]` → `- [x]`) | OpenSpec tracks task state in `tasks.md`; `apply --json` re-reads it |
| `verify` | `openspec validate <c>` (and/or `/opsx:verify`) | non-zero exit = fail |
| `archive` | `openspec archive <c>` (or `/opsx:archive`) | moves to `openspec/changes/archive/` and updates specs |

**Verified JSON shape** of `openspec instructions apply --change <c> --json`:

```json
{
  "changeName": "…", "changeDir": "…", "schemaName": "spec-driven",
  "state": "in_progress | blocked | all_done",
  "progress": { "total": 10, "complete": 0, "remaining": 10 },
  "tasks": [ { "id": "1", "description": "…", "done": false } ],
  "contextFiles": { "...": ["path"] }, "instruction": "…"
}
```

Notes:
- `state: "blocked"` → missing artifacts; driver surfaces and stops (do not loop).
- `state: "all_done"` → driver skips straight to `verify` → `archive`.
- The task `id` is a string (`"1"`); preserve it as-is for `mark_done`.

---

## Adapter: GitHub Spec Kit  (see change-007 — documented, not yet wired)

| Op | Concrete command | Mapping |
|---|---|---|
| `detect` | `[ -d .specify ] || ls specs/*/tasks.md` | Spec Kit layout |
| `list_tasks` | parse `specs/<feature>/tasks.md` checklist | `- [ ] T001 …` → `{id:"T001", title, done:false}` |
| `progress` | derive from parsed checklist | counts of `[x]` vs total |
| `mark_done` | check the box in `tasks.md` | `- [ ]` → `- [x]` for that id |
| `verify` | `/speckit.analyze` (quality gate) | cross-artifact consistency |
| `archive` | n/a (Spec Kit has no archive) — KBD marks the feature done | document the asymmetry |

Spec Kit's `/speckit.implement` is the "do everything" command and is therefore
**not** used as the per-task executor — the driver implements each task itself
(or delegates a single task), consistent with the hard invariant above.

---

## Driver loop (pseudo-contract, implemented in `kbd-apply.sh` + SKILL.md)

```
change = active change from waypoint
tasks  = backend.list_tasks(change)
p      = backend.progress(change)
for (i, t) in enumerate(tasks where not t.done):
    kbd_hooks_fire task before "<change>:<t.id>" i p.total   # → reporter + memory
    emit  "Starting task <i> of <p.total>: <t.title>"        # plain-text guarantee
    <implement exactly task t — self or single delegated agent>
    backend.mark_done(change, t.id)
    sync progress.json (tasks_done++) + refresh waypoint
    kbd_hooks_fire task after  "<change>:<t.id>" i p.total
    emit  "Completed task <i> of <p.total>: <t.title>"
# final task fires on_change_complete via index==total sentinel
run artifact-refiner QA gate
backend.verify(change) && backend.archive(change)
```
