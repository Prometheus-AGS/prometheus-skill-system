# native-kbd spec backend

The PMPO-native spec backend — the always-available fallback behind the same
SpecBackend interface as OpenSpec and Spec Kit (`spec-backend-interface.md`),
purpose-built for one-task-per-turn driving by `kbd-apply`. There is no "do
everything" command to mis-invoke, which is the entire control-model mismatch
that drove OpenSpec users out of the KBD loop.

## Directory layout

```
.kbd-orchestrator/
  specs/                         # living capability specs (optional; archive merges into these)
    <capability>.md
  changes/
    <change-id>/
      spec.md                    # proposal + delta spec (legacy: change.md accepted)
      tasks.json                 # SOURCE OF TRUTH — per-task state (schema: change-tasks.schema.json)
      tasks.md                   # GENERATED human view (banner warns; regenerated on every mutation)
      verification.md            # acceptance criteria; optional fenced verify: commands
    archive/<date>-<change-id>/  # retired changes (mv on archive)
```

## tasks.json is the source of truth

Schema: `references/schemas/change-tasks.schema.json`.

```json
{
  "changeId": "change-001-native-kbd-backend",
  "schemaVersion": "1",
  "tasks": [
    { "id": "1", "title": "Write the schema", "done": true,
      "doneAt": "2026-06-11T18:00:00Z", "doneBy": "claude-code",
      "files": ["references/schemas/change-tasks.schema.json"], "verify": null, "notes": null }
  ]
}
```

Why JSON, not checkboxes:

- **One-task-per-turn from day one** — `nk_mark_done` is an atomic `jq`
  mutation (tmp + `mv`), the same pattern `waypoint.sh` uses.
- **Cross-tool ledger** — `doneBy`/`doneAt` record which tool completed each
  task, which a checkbox cannot carry.
- **No ordinal fragility** — task ids are explicit, unlike OpenSpec's
  positional ordinals; reordering tasks does not renumber state.

`tasks.md` is regenerated after every mutation for humans and carries a
`<!-- GENERATED … -->` banner. Hand-edits to `tasks.md` are overwritten —
edit `tasks.json` (or the spec) instead.

## Lazy migration from legacy change.md

Older native changes tracked tasks as inline `- [ ] 1. title` checkboxes in
`change.md`. On the first `nk_list`/`nk_mark_done`, `kbd-apply` parses those
checkboxes into `tasks.json` once (the original `change.md` is preserved). All
subsequent state lives in `tasks.json`.

## Backend selection

`project.json` gains `"specBackend": "openspec" | "native-kbd" | "speckit" | "auto"`.
Default (absent or `auto`) detection order: openspec → speckit → native-kbd.
An explicit value pins the backend. New `kbd-init` projects should default to
`native-kbd`.

## Adapter functions (in kbd-apply.sh)

`nk_detect`, `nk_list`, `nk_progress`, `nk_mark_done`, `nk_verify`,
`nk_archive` — wired into the `b_*` dispatch arms. Hook firing, progress.json
sync, position.json sync, and the plain-text position signal are all provided
by the shared `begin-task`/`end-task` driver loop, identical to every other
backend — that is the payoff of slotting in as an adapter rather than a new
driver.
