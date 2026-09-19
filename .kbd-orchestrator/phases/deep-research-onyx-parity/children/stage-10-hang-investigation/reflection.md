# Phase Reflection: deep-research-onyx-parity › stage-10-hang-investigation

**Project:** prometheus-skill-pack
**Date:** 2026-09-13
**Phase completion:** 75% (3 of 4 goals MET — G2 met by post-reflect archaeology; G1 closed as a proven negative)
**Changes completed:** 3 / 3 implemented · 2 archived · 1 certification-BLOCKED (drt-011)

## Post-reflect correction (2026-09-13, operator-challenged)

The operator challenged the NOT-MET grading. Re-investigation changed it: G2 is **MET** (timeline +
git archaeology: introduced this phase, first appearance 09-09 ~20:26, never code-remediated — the
same bytes later passed 98×; "incidentally fixed by drt-006" DISPROVEN — extractions predate the
hang). G1 remains NOT MET but is now a **closed negative** (artifact hunt exhausted, no surviving
trace, no discriminable suspect), not an open gap. Completion 50% → 75%. Evidence: HANG-CAPTURE.md
post-reflect addendum; dist resynced (4 checks PASS).

## What diverged from the plan (deltas first)

The plan was written around an intermittent ~1-in-3 stage-10 hang. **The defect
did not survive contact with instrumentation**: 0 hangs in 98 runs across four
campaigns (K=12, K=24 escalation, an exact-state probe reproducing the
original suite→full-run sequence, and the N=60 certification streak), under
machine load 50–493. Two goals presupposed a reproducible defect and are
therefore NOT MET on the evidence — the honest outcome, not a failure of
execution. Separately, drt-011's certification gate ended BLOCKED in a
structural dispute (judge demands ~700 KB of generated file contents inside
the review packet; the outputs are staged in git — 405 files, 24,618+/4,539−
— and three drift checks pass). Root causes and corrective actions below.

## Goals

| Goal | Status | Notes |
| --- | --- | --- |
| G1: identify the exact blocking command with a reproducible trace | **NOT MET — investigation CLOSED as a proven negative** | 0 hangs / 98 runs; no surviving artifact of the hung run exists (the 09-09 job checkpoints are 09-08 threaded-test jobs); same-code proof shows no discriminable suspect. The exact command is unidentifiable from any surviving evidence; the --xtrace tripwire converts any recurrence into exactly this evidence. |
| G2: pre-existing vs introduced, first appearance | **MET (post-reflect archaeology)** | Introduced, not pre-existing — PROVEN: every implicated file is new-in-tree this phase (0 prior commits); the hang cannot predate the code defining it. First appearance 2026-09-09 ~20:26 (dated from the assessment record). Not code-remediated: no chain-file edit exists between the hang and the 98 clean runs — the same bytes hung once and passed 98 times, so the discriminator is environmental state or an untriggered race. Graded NOT MET pre-addendum only because the archaeology (mtimes, git log, surviving-artifact hunt — all machine-checkable, recorded in HANG-CAPTURE.md's addendum) had not been run. |
| G3: fix so run-research completes a full deep run, proven by driver-contract full count | **MET (as restated)** | The single-run proof standard was restated at analyze (D1) into the only standard meaningful for an intermittent defect: N consecutive clean runs. Delivered: N=60/60 streak (Clopper-Pearson-sized) + full suite 128/128. No fix was applied because none was warranted — 98 consecutive healthy executions of the exact scenario that hung once on 2026-09-09. |
| G4: adversarial review of the diagnosis before any fix | **MET** | Distinct-judge review (MiniMax-M3 vs producer, verified-distinct, machine-checked) ran 2 rounds; accepted at the spec's 2-round cap with the CRITICAL carried verbatim as Unresolved. No fix preceded it — indeed no fix occurred at all (branch: none). |

## Delivered Changes

- `change-drt-008-hang-capture-harness` — instrumented repeat-run capture harness + HANG-CAPTURE.md evidence pack (by: opencode/glm-5.3, self-executed via kbd-apply; 4 review rounds, archived 2026-09-13)
- `change-drt-009-stage10-hang-fix-and-certification` — Goal-4 diagnosis gate (distinct-judge, accepted-with-unresolved), branch: none (no fix warranted), --xtrace diagnostic mode, N=60 certification + FIX-CERTIFICATION.md (by: opencode/glm-5.3; 3 review rounds, archived 2026-09-13)
- `change-drt-011-phase-close-distribution-regen` — C-01 reconciliation: generator ×2 byte-identical (full-tree hash cbc3725f…), check:distribution + validate:codex + check:skills-index PASS, 405-file manifest committed (by: opencode/glm-5.3; implementation COMPLETE, certification **BLOCKED** — packet-scale dispute, NOT archived)

## Technical Debt

- **drt-011 unarchived** with certification BLOCKED (dispute over review-form, not sync: outputs regenerated and staged). Unblock via a manifest-accepting review mode or an operator-signed waiver.
- **change-drt-010 (cut from plan)** still owed as a standalone post-child change: `build-review-packet.sh --target spec` child-phase CHANGES_ROOT bug + retirement of the phase-local symlink bridge at `.kbd-orchestrator/phases/deep-research-onyx-parity/changes/`.
- **Generator payload hygiene:** `tests/hang/captures*/` campaign artifacts mirror into `dist/plugins/**` (generator has no exclusion rules). Future change.
- **Harness recorder semantics:** `run-hang-capture.sh` exits 0 even when hangs/failures are recorded (by design; certification reads the table). Flagged in three review rounds; future harness change.
- **Runtime projection:** `exactNextCommand` projected `/kbd-assess` throughout the child's lifecycle (stale derivation; canonical state was correct). Plus a `kbd-validate-progress.sh` lookup error ("change not found") despite correct state — both recorded in the stage decision log.
- **Stale `.git/index.lock`** (46 h, from a crashed process) blocked `git add` until verified-and-removed — worth a guard in the packet builder or docs.

## Architecture Integrity

- AGENTS.md violations: **NONE** — no mutation guards added; all validation local (zero CI); no commits made (the reflect protocol's commit step is intentionally deferred to the operator per the no-commit-without-instruction rule); implementation-first followed (full harness built before campaigns; suites run as acceptance gates only).
- Constraints: C-01 **honored with one open certification dispute** (deferral contract followed; outputs regenerated deterministically; C-04 idempotency proven ×3 hashes); C-02 clean (scans); C-03 N/A; C-05 clean (bash 3.2, both shells, no mapfile/declare -A).

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Changes with QA | 3/3 |
| First-pass pass rate | 1/3 (drt-011; drt-008 and drt-009 each required revision rounds) |
| Diff-review revision rounds | 4 (drt-008) · 3 (drt-009) · 4+dispute (drt-011) |
| Judges used | MiniMax-M3 (verified-distinct, ×6 dispatches), gpt-5.5 via openai-proxy (verified-distinct, ×4), harness-native same-model fallback (×2, recorded honestly) |
| Constraint violations recurring | none across changes (C-01..C-05 all PASS/N-A per change) |

Recurring **process** findings (not constraint violations): claimed-but-unimplemented behavior appeared in 2 of 3 changes (perturbation marking; lstart defense) — self-authored docs promising more than code delivers is this producer's dominant defect class; verify-strings that cannot fail appeared twice and were fixed with machine-countable structures.

## Cross-Tool Coordination Notes

- Progress tracking: **RELIABLE** — canonical registration (prometheus kbd change/task) kept the ledger exact; runtime rollup updated project-wide counters (34/42 → 37/45) as changes completed. GAPS: the position projection's exactNextCommand stayed stale all phase; manual waypoint JSON edits were superseded by hook refreshes (lesson: only typed CLI + kbd-apply loops move projections durably).
- Handoff quality: CLEAR — the stage handoff chain (assess→analyze→spec→plan→execute→reflect) carried every warning, dispute, and gotcha forward; each stage consumed its predecessors' dispositions explicitly.
- Service layer: liter-llm wedged twice (LISTEN-but-000; completions timeouts under load) — fixed by launchctl kickstart; judge routing moved to gpt-5.5 via openai-proxy :8181 per operator direction (gpt-5.4 advertised-but-rejected by the ChatGPT-plan Codex backend; gpt-5.5 works, 400K context, zero timeouts after). surreal-memory intermittent timeouts (transient; in-repo decision log used as authoritative record).

## Lessons Learned

- Intermittent defects yield to instrumented repeat-run campaigns, not serial one-shot theories: six theories died before this phase; the campaign answered in 38 runs (and 98 total retired the question).
- Block-buffered stdout + external timeout kill destroys "zero output" inferences — a killed process's silence localizes nothing. Record the reinterpretation; arm `--xtrace`.
- Diff-review packets need `files.txt` **and** `git add` of the change's files, or the builder silently falls back to `git show HEAD` — reviewing the wrong commit. Samples must cover every generated tree (claude AND codex) or judges infer absence.
- Determinism proofs must hash every file type and label their baselines honestly ("steady state" ≠ "pre-change").
- Advertised ≠ supported on model proxies; verify a model with a live completion before wiring it as an infrastructure dependency.
- Review gates with 2-round caps + accept-with-unresolved keep honesty terminal instead of infinite; the pattern held across three artifact reviews and one diagnosis review.
- Judges re-flag what packets structurally cannot show (plan-mode excludes change specs; diff-mode samples ≠ full generated trees). Disposition by pointer-to-recorded-evidence works; a manifest-accepting review mode for generated-output changes is the real fix.

## Next Phase Focus

Close-out phase for this child's loose ends (or fold into the parent's close):
1. drt-011 unarchive decision — operator waiver or a manifest-accepting review mode (tooling change).
2. change-drt-010: child-phase packet CHANGES_ROOT fix + symlink-bridge retirement.
3. Generator payload hygiene (captures/ exclusion) + harness exit-semantics change.
Human review needed: the drt-011 waiver decision, and whether the parent phase's final commit pass happens before or after drt-010.

## Context for Next Phase

Use this file as prior context for the next `/kbd-assess` invocation. The
defect that motivated this child no longer reproduces (0/98); any future
hang, by any observation, voids the certification per the armed falsifier
(DIAGNOSIS.md) — hunt it with `--xtrace`, not theories.
