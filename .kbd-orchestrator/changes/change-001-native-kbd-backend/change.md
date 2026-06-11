---
id: change-001-native-kbd-backend
title: Native PMPO spec backend (native-kbd) in kbd-apply
phase: canonical-lifecycle
gaps: [G2]
priority: P1
effort: L
agent: claude-code
evolver_item_id: null
status: done
scope:
  - skills/process/kbd-process-orchestrator/skills/kbd-apply/kbd-apply.sh
  - skills/process/kbd-process-orchestrator/references/schemas/change-tasks.schema.json
  - skills/process/kbd-process-orchestrator/references/native-backend.md
  - skills/process/kbd-process-orchestrator/references/spec-backend-interface.md
  - skills/process/kbd-process-orchestrator/shared/lib/tests/test-kbd-apply-native.sh
---

# change-001 — Native PMPO spec backend (native-kbd)

## Context

`kbd-apply.sh` dispatches over `BACKEND` (openspec|speckit) and `die`s with
"no spec backend detected" when neither is present — so native `change.md`
specs cannot be driven one-task-per-turn. This is the OpenSpec control-model
mismatch the user wants removed: a PMPO-native backend behind the same
SpecBackend interface, designed for one-task driving + hook integration from
the start.

## Scope

In:

- New `change-tasks.schema.json` — `tasks.json` source-of-truth format:
  `{changeId, schemaVersion, tasks:[{id,title,done,doneAt,doneBy,files,verify,notes}]}`.
- `kbd-apply.sh`:
  - `nk_detect`: native-kbd when `.kbd-orchestrator/changes/<change>/tasks.json`
    OR legacy `change.md` exists.
  - `nk_list`: read tasks.json TSV; **lazy migrate** a legacy `change.md`
    checkbox list into tasks.json on first call (original preserved).
  - `nk_progress`: jq counts.
  - `nk_mark_done`: atomic jq set done=true/doneAt/doneBy (`$KBD_TOOL`),
    regenerate a `tasks.md` human view with a generated-file banner.
  - `nk_verify`: run fenced `verify:` commands from verification.md if present,
    else structural check (all tasks done + spec.md exists). Non-zero = fail.
  - `nk_archive`: move `changes/<id>` → `changes/archive/<date>-<id>/`.
  - Extend `backend_detect()` to return `native-kbd` as the final fallback;
    honor `project.json.specBackend` ("openspec"|"native-kbd"|"speckit"|"auto",
    default auto) when present.
  - Add native-kbd arms to `b_list/b_progress/b_mark_done/b_verify/b_archive`.
- `references/native-backend.md` (layout + adapter doc) and a native-kbd row in
  `spec-backend-interface.md`.
- New `test-kbd-apply-native.sh`: fixture tasks.json → list/progress/mark_done
  TSV + atomic update + tasks.md regen; legacy change.md → lazy migration;
  detect precedence with specBackend.

Out: position-sync wiring (change-002), spec-creation skill (change-003).

## Tasks

- [x] 1. Write change-tasks.schema.json
- [x] 2. Add nk_* adapter functions + backend_detect fallback + specBackend honoring
- [x] 3. Wire native-kbd into b_* dispatch arms
- [x] 4. Write native-backend.md + spec-backend-interface.md row
- [x] 5. Write test-kbd-apply-native.sh; run green

## Verification

`bash .../shared/lib/tests/test-kbd-apply-native.sh` green; existing
test-kbd-apply-*.sh still green (no openspec/speckit regression).
