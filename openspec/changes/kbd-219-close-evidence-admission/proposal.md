# kbd-219-close-evidence-admission

## Why

Issue #219 (reproduced 2026-11-12): `prometheus kbd phase close --disposition pass
--evidence-file <f>` closed a contract-bound phase for evidence naming another phase and
project, with no `sourceDigests` and no `attempted`/`observed`/`evidence` fields. The shell
evaluator `kbd-phase-close-eval.sh` refuses the same evidence. The CLI adapter dropped those
facts before the runtime saw them, so the canonical admission could not check them and the two
supported routes disagreed.

## What changes

- The CLI adapter passes the evidence facts through: `phase`, `projectId`, `sourceDigests`, and
  each scenario entry's `attempted`, `observed` and `evidence`.
- The canonical close admission refuses a new PASS closure, before any event is appended, when
  the evidence phase is not the phase being closed, a supplied project is not this project,
  `sourceDigests` is empty or names a file whose current sha256 differs, or a scenario entry
  owned by a bound scenario (`S` or `S-<sub>`) has an empty observation field. Reason codes are
  the evaluator's: `evidence-phase-mismatch`, `evidence-project-mismatch`,
  `evidence-source-digests-missing`, `evidence-stale-sources`, `empty-evidence:<id>`.
- Newly admitted disposition events carry `admissionVersion: 2`, stamped by the runtime, never
  by the caller. The reducer applies the pure fact checks only to events at that version.
  Events without it (written before this change) replay with their recorded semantics.
- Source-file hashing happens once, at command admission on a host with the project tree. The
  reducer never reads files, so replay cannot be changed by later edits. A filesystem-free
  replica cannot admit a PASS closure.

- The same admission step reads the bound contract snapshot
  (`.kbd-orchestrator/phases/<id>/adopted/<digest>.json`, verified against the bound digest) and
  refuses a PASS closure whose metered consumption is above a ceiling (`budget-exceeded`), whose
  enforced dimension has no metered value (`budget-unmeasured`), or whose snapshot is missing or
  altered (`snapshot-invalid`). Exactly at a ceiling closes. The cumulative high-water rule is
  unchanged. Operator decision 2026-11-13: direct close enforces the ceilings the evaluator does.

## Out of scope

- #184/#192/#196 scope, raw assertion or provenance strengthening, a new ledger or controller.
