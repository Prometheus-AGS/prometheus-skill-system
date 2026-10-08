# Spec Engines

The skill pack drives spec-driven implementation through pluggable **spec
engines**. `/kbd-apply` (in `skills/process/kbd-process-orchestrator/skills/kbd-apply/`)
wraps one engine at a time and walks its tasks ONE at a time so KBD stays the
source of truth: every task boundary fires the KBD hooks, emits a plain-text
position signal, and syncs `progress.json` and the waypoint.

Engine metadata (CLI, pinned version, adapter pointer) is registered in
[`config/spec-engines.json`](../../config/spec-engines.json). The Node helper
[`shared/scripts/spec-engine-info.mjs`](../../shared/scripts/spec-engine-info.mjs)
prints the registry and — with `--root <dir>` — the same detection result the
bash driver computes.

## OpenSpec — the default

[OpenSpec](https://github.com/Fission-AI/OpenSpec) (`@fission-ai/openspec`,
CLI `openspec`) is the **default engine**. Its artifacts live under
`openspec/changes/<change>/{proposal.md,design.md,specs/,tasks.md}`. The pack
pins the exact version in `package.json` (`@fission-ai/openspec`, exact pin —
currently 1.14.0; the authoritative value is whatever `package.json` says).
The adapter is `os_*` in `kbd-apply.sh`: `list` and `progress` come from
`openspec instructions apply --change <id> --json`, `mark_done` flips the
matching checkbox in `tasks.md`, `verify` runs `openspec validate`, and
`archive` runs `openspec archive <id> --yes` (the `--yes` is required — the
interactive prompt has no stdin here).

## Spec Kit — fully supported alternative

[GitHub Spec Kit](https://github.com/github/spec-kit) (v1.x, CLI `specify`,
installed with `uv tool install specify-cli`, pinned 1.1.2 in
`config/spec-engines.json`) writes its artifacts under `.specify/` (templates
and scripts) and `specs/<slug>/{spec.md,plan.md,tasks.md}`. A "change" for the
adapter is the feature-dir slug under `specs/`.

### Detection

- Repo-wide: a `.specify/` directory, or any `specs/*/tasks.md`, selects
  `speckit`.
- Change-scoped: `specs/<change>/tasks.md`, `specs/<change>/spec.md`, or
  `specs/<change>/plan.md` (v1 feature dirs may be driven before `tasks.md` is
  generated) selects `speckit` for that change.

### Pinning

Pin the engine explicitly in `.kbd-orchestrator/project.json`:

```json
{ "specBackend": "speckit" }
```

Valid values: `"openspec"`, `"speckit"`, `"native-kbd"`, `"auto"` (detect from
the repository shape; `openspec` wins when `openspec/` exists). A pin always
overrides detection.

### Updating when a new Spec Kit release ships

1. Update `pinnedVersion` in `config/spec-engines.json` to the new exact
   version (dependency pins are exact, never ranges).
2. Verify the adapter ops — `sk_list` / `sk_progress` / `sk_mark_done` /
   `sk_verify` / `sk_archive` in `kbd-apply.sh` — against the new release's
   `specs/` layout (checkbox format of `tasks.md`, presence of `spec.md`,
   `plan.md`, `tasks.md` in the feature dir).
3. No code change is expected unless the layout changes. If it does, update
   the adapter functions and the detection shapes together, and record the
   reason in `config/spec-engines.json`'s `rationale` field.

`sk_verify` is a structural gate — every checkbox in `specs/<change>/tasks.md`
checked AND `specs/<change>/spec.md` present — because Spec Kit's own
`/speckit.analyze` is model-driven (it dispatches an LLM, not a CLI) and
cannot run inside the driver's non-interactive loop. `sk_archive` is
best-effort and pack-side: Spec Kit has no native archive command, so the
adapter moves `specs/<change>` to `specs/archive/<date>-<change>`, mirroring
native-kbd's archive.

## Native-kbd

The pack-owned backend. Source of truth:
`.kbd-orchestrator/changes/<change>/tasks.json` (schema:
`references/schemas/change-tasks.schema.json`); `tasks.md` is a regenerated
view. Detected change-scoped by `tasks.json` or a legacy `change.md`, and
repo-wide by `.kbd-orchestrator/changes/*/tasks.json|change.md`. It is the
always-available fallback; no external pin.

## The adapter contract

Every engine implements five operations as `<prefix>_*` functions in
`kbd-apply.sh` (`os_`, `sk_`, `nk_`):

| Op | Signature | Output / effect |
|---|---|---|
| `list` | `<change>` | TSV on stdout: `id<TAB>done(0|1)<TAB>title`, one line per task |
| `progress` | `<change>` | `total complete remaining` on stdout |
| `mark_done` | `<change> <id>` | flip one task to done in the engine's own artifact |
| `verify` | `<change>` | structural/CLI gate; non-zero exit = fail |
| `archive` | `<change>` | move the change out of the active area |

The `b_*` dispatchers (`b_list`, `b_progress`, `b_mark_done`, `b_verify`,
`b_archive`) resolve the backend per change via `backend_detect` (pin first,
then change-scoped shape, then repo-wide shape) and route to the right
adapter. Unknown backends fail loudly for list/progress/mark_done and are a
safe no-op pass for verify/archive. The driver never invokes a backend's
"implement everything" command — it calls the backend per task.

The TSV contract (`id\tdone\ttitle`) is what the per-task loop, reconcile, and
the position model consume; adapters must keep it stable.

## Adding a new spec engine

1. **Implement the five ops** as `<prefix>_` functions in
   `skills/process/kbd-process-orchestrator/skills/kbd-apply/kbd-apply.sh`,
   honoring the TSV contract above.
2. **Add a detection shape** in `backend_detect` — both the change-scoped
   branch (what marks *this* change as yours) and the repo-wide heuristic —
   and add your engine id to the `specBackend` pin `case` list.
3. **Register the engine** in `config/spec-engines.json` under `engines`
   (CLI name, exact `pinnedVersion`, `rationale` pointing at this document,
   adapter prefix). Third-party engines that are not shipped in the pack can
   be recorded under `customEngines`.
