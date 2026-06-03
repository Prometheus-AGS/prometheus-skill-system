# Plan — kbd-execute-spec-wrapping-and-nesting

- **Project:** prometheus-skill-system
- **Phase:** kbd-execute-spec-wrapping-and-nesting-2026-06-03
- **Date:** 2026-06-03
- **Source assessment:** `assessment.md` (findings F1–F7)
- **Change backend:** **native KBD** (deliberate deviation — see below)

## Backend decision (and why it deviates from the default rule)

The standard rule is "OpenSpec present → emit OpenSpec changes." OpenSpec *is*
present (dir + `openspec` CLI on PATH). **This phase intentionally uses native
KBD change files instead**, for one reason: the phase exists to repair the
broken `/opsx:apply` execute seam (F1). Driving these changes *through* that
seam would mean dogfooding the broken path to fix the broken path — the exact
trap the assessment flags. Native KBD keeps the loop fully under KBD/Claude Code
control while we build and verify the wrapper. Once F1 lands, OpenSpec wrapping
resumes as the default for later phases.

The work here is also doc + shell edits to the orchestrator skill itself, which
maps cleanly to native changes and does not need spec-level traceability.

## Ordered change list

Order honors the assessment priority: **F1 + F2 first; Spec Kit (F4) last.**
Each change is independently committable. `Model class` drives execute routing.

---

### change-001-spec-backend-interface  ⟶ FIRST
**Addresses:** F1, F4 (foundation) · **Model class:** medium · **Agent:** claude-code

Define the backend-agnostic driver contract so the driver (002) and adapters
(001 OpenSpec, 007 Spec Kit) share one interface.

Tasks:
- [ ] Write `skills/process/kbd-process-orchestrator/references/spec-backend-interface.md` defining `SpecBackend`: `list_tasks() → [{id,title,done}]`, `mark_done(id)`, `verify()`, `archive()`, `progress() → {total,complete,remaining}`.
- [ ] Document the **OpenSpec adapter** mapping: `list_tasks` ← `openspec instructions apply --change <c> --json` (`.tasks`, `.progress`); `mark_done` ← edit `tasks.md` checkbox / openspec CLI; `verify` ← `openspec validate`/`/opsx:verify`; `archive` ← `/opsx:archive`.
- [ ] State the invariant: **KBD owns the loop; the adapter is a subroutine.** Bare `/opsx:apply` is never invoked.

Acceptance: interface doc exists; OpenSpec adapter mapping is concrete (real CLI commands, verified against `openspec --help`).

---

### change-002-kbd-apply-driver  ⟶ CORE FIX
**Addresses:** F1 (root cause) · **Model class:** frontier · **Agent:** claude-code

Create the KBD-owned apply driver that wraps the spec backend task-by-task.

Tasks:
- [ ] New skill `skills/process/kbd-process-orchestrator/skills/kbd-apply/{SKILL.md,kbd-apply.sh}`.
- [ ] `kbd-apply.sh`: resolve active change from waypoint → call adapter `list_tasks` → **loop one task at a time**: `kbd_hooks_fire task before <name> <i> <n>` → emit plain-text `Starting task <i> of <n>: <title>` → (model implements that single task) → `mark_done` → sync `progress.json` (`tasks_done++`) + refresh waypoint → `kbd_hooks_fire task after <name> <i> <n>` → emit `Completed task <i> of <n>`.
- [ ] On final task: fire `on_change_complete` sentinel (`INDEX==TOTAL`), run artifact-refiner QA gate, then `verify` → `archive`.
- [ ] SKILL.md documents invocation, the per-task contract, and the no-bare-`/opsx:apply` rule.

Acceptance: running `kbd-apply` against a change with N tasks fires `task:before`/`task:after` N times, writes `progress.json` after each, and emits a plain-text position line per task. No call to bare `/opsx:apply`.

---

### change-003-rewrite-execute-dispatch
**Addresses:** F1, F3 · **Model class:** medium · **Agent:** claude-code

Make `kbd-execute` dispatch to the new driver and delete the false claim.

Tasks:
- [ ] In `skills/.../kbd-execute/SKILL.md`: **remove** the line claiming "task:before/task:after are fired per OpenSpec task by `/opsx:apply`." Replace with: execute writes the dispatch contract; `kbd-apply` drives tasks and fires per-task hooks.
- [ ] In `prompts/execute.md` → "Dispatch Protocol" / "If backend = openspec and self-executing": replace the soft "treat OpenSpec tasks as the working surface … sync after each task" with "invoke `kbd-apply` (it owns the per-task loop); never invoke bare `/opsx:apply`."
- [ ] Clarify the plan/execute boundary (F3): change *creation* stays in `kbd-plan` (`/opsx:new`); change *execution* is `kbd-apply` via `kbd-execute`. Document in both SKILLs.

Acceptance: no file claims `/opsx:apply` fires KBD hooks; execute.md routes to `kbd-apply`; grep for `opsx:apply` shows only "do not invoke bare" guidance.

---

### change-004-per-turn-position-reporter
**Addresses:** F2 · **Model class:** medium · **Agent:** claude-code

Make per-turn position reporting reliable, not stderr-and-voluntary.

Tasks:
- [ ] Document the split in orchestrator `SKILL.md`: **plain-text Progress Signals = the user-facing guarantee** (driver- and skill-emitted); the `*:*` stderr hook = user override/extension point only.
- [ ] Ship a documented Claude Code settings-hook recipe (`Stop` or `PostToolUse`) that injects `waypoint_chain(parent,phase,child) + i/n` into context each turn, so position survives even outside KBD skill turns. Place under `references/per-turn-position-hook.md`.
- [ ] Ensure `kbd-apply` (002) and each loop skill emit the plain-text signal — covered by 002 for tasks; verify assess/plan/execute/reflect already do (they do).

Acceptance: a documented mechanism exists that surfaces position to the user every turn without relying on the model voluntarily sourcing `hooks.sh`; docs no longer imply the stderr hook is user-visible.

---

### change-005-hooks-robustness
**Addresses:** F5, F7 · **Model class:** small · **Agent:** claude-code

Fix two live defects observed while firing hooks during assess/plan.

Tasks:
- [ ] F5: harden `shared/lib/memory-log.sh` so it does not emit `jq: parse error` when the memory endpoint is absent or input is empty (guard/validate before `jq`).
- [ ] F7: `shared/lib/hooks.sh` must `source` `waypoint.sh` defensively (or guard `chain_separator`/`waypoint_chain` with `command -v`) so sourcing `hooks.sh` alone does not throw `command not found`.
- [ ] Extend `shared/lib/tests/test-hooks.sh` to cover both: sourcing hooks.sh alone, and firing with no memory endpoint — both must be silent on stderr except the intended reporter line.

Acceptance: `bash shared/lib/tests/test-hooks.sh` passes; firing any hook with hooks.sh sourced alone produces no `command not found` and no `jq` error.

---

### change-006-child-loop-wrapping
**Addresses:** F6 · **Model class:** medium · **Agent:** claude-code

Ensure nested child loops use the same wrapped driver and full-chain reporting.

Tasks:
- [ ] Verify `/kbd-new-child` / `/kbd-next-child` children get their own `execution.md` driven by `kbd-apply` (not the old seam).
- [ ] Position reporter renders the full `parent › phase › child` chain (via `waypoint_chain`), not just the innermost name, for both outer and inner loops.
- [ ] Add `shared/lib/tests/test-child-apply.sh` (or extend existing child tests) asserting a child phase fires `task:*` through `kbd-apply` and reports the full chain.

Acceptance: a child phase executes through `kbd-apply`; position lines show the full outer+inner chain.

---

### change-007-speckit-adapter  ⟶ LAST (deferred capability)
**Addresses:** F4 · **Model class:** medium · **Agent:** claude-code

Add a thin second adapter behind `SpecBackend` — only after OpenSpec wrapping is proven.

Tasks:
- [ ] `references/spec-backend-interface.md`: add Spec Kit adapter mapping — `list_tasks` ← parse `specs/<feature>/tasks.md`; `mark_done` ← check the box; `implement` ← `/speckit.implement` per task; `verify` ← `/speckit.analyze`.
- [ ] Register `speckit` backend in `prompts/execute.md` backend registry + selection rules (detect `.specify/` or `specs/**/tasks.md`).
- [ ] Do **not** fork the loop — Spec Kit reuses `kbd-apply` via the adapter.

Acceptance: `speckit` is a selectable backend; `kbd-apply` drives a Spec Kit `tasks.md` without code duplication.

---

### change-008-verify-and-integration-test
**Addresses:** whole-phase verification · **Model class:** medium · **Agent:** claude-code

End-to-end proof the seam is fixed.

Tasks:
- [ ] Create a throwaway OpenSpec change with 2–3 tasks; run the wrapped execute (`kbd-execute` → `kbd-apply`); confirm: task hooks fire per task, `progress.json` increments per task, plain-text position line emitted per task, QA gate + archive run on completion.
- [ ] Run `npm run validate:strict` for any new/edited skill (`kbd-apply`).
- [ ] Run all `shared/lib/tests/*.sh`; all green.
- [ ] Tear down the throwaway change.

Acceptance: full assess→plan→execute(→apply per task)→archive observed with continuous position reporting; all skill validations and hook tests pass.

---

## Dependency / sequencing

```
001 (interface) ──▶ 002 (driver) ──▶ 003 (execute routes to driver)
                                   └▶ 004 (per-turn reporting)
005 (hooks robustness) ── independent, can land anytime (do early; it's cheap)
006 (child wrapping) ── after 002/003
007 (speckit) ── LAST, after 008 proves OpenSpec path
008 (verify) ── after 002/003/004/006
```

Recommended commit order: **005 → 001 → 002 → 003 → 004 → 006 → 008 → 007**.
(005 first as a cheap warm-up that also de-noises hook output for the rest.)

## Model routing summary

| Change | Class | Why |
|---|---|---|
| 001 | medium | one doc, concrete CLI mapping, no new abstraction in code |
| 002 | **frontier** | new abstraction, multi-state loop, cross-cutting (hooks+waypoint+progress+QA) |
| 003 | medium | bounded edits across 2 docs + boundary clarification |
| 004 | medium | doc + settings-hook recipe, one design marker (Stop vs PostToolUse) |
| 005 | small | two localized shell fixes + test |
| 006 | medium | one module boundary (child path) + test |
| 007 | medium | second adapter, bounded |
| 008 | medium | integration scripting + teardown |

## Next action

Proceed to **`/kbd-execute kbd-execute-spec-wrapping-and-nesting-2026-06-03`**,
which (per change-003's intent) will itself dogfood the corrected dispatch:
KBD owns the loop, `kbd-apply` drives tasks, position reports every task.
Start with change-005, then 001 → 002.
