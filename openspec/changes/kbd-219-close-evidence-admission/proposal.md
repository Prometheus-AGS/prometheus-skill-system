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

## Out of scope

- Budget ceilings stay where they are (evaluator reads them from the bound snapshot; the runtime
  keeps the cumulative high-water rule).
- #184/#192/#196 scope, raw assertion or provenance strengthening, a new ledger or controller.
