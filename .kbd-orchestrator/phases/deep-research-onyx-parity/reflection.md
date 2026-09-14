# Phase Reflection: deep-research-onyx-parity

**Project:** prometheus-skill-pack
**Date:** 2026-09-13
**Phase completion:** 95% (minor release: 1.8.0 → 1.9.0) (4 of 5 goals MET; G5 MET with 3-of-10 partial bench + capability in place; 5/10 supplementary packages queued)
**Changes completed:** 7 / 7 parent changes driven (6 DONE + drt-006 at 3/4 tasks, task 4 externally blocked) · child stage-10-hang-investigation closed beneath this phase (75% of its own goals; 3/3 changes archived)

## What diverged from the plan (deltas first)

Two deltas shape this reflection. First, **drt-006's measurement goal stopped one task short of
falsifiable parity**: the benchmark harness and all three FACT metrics shipped and were exercised
on real packages, but RACE overall is BLOCKED — no available research package answers a benchmark
task, and producing ten `--depth deep` runs plus judge calls is explicitly a deliberate,
token-costing operator action (drt-006 task 4's blocked_reason). "On par with Onyx" therefore
remains **asserted-but-not-scored**: Onyx's own published figure is itself labelled `unverified`.
Second, the **stage-10 intermittent hang** consumed a full child phase and resolved as a
same-code environmental mystery: 0 hangs in 98 instrumented runs, the implicated bytes proven
identical between the one hang and the 98 passes, the motivating defect never identified (child
G1 closed as a proven negative; child G2 resolved by archaeology — introduced this phase, first
appearance 2026-09-09 ~20:26, never code-remediated).

## Goals

| Goal | Status | Notes |
| --- | --- | --- |
| G1 Thread execution (director/worker, deterministic merge, contracts unchanged) | **MET** | drt-002/003/004 all DONE (003/004 projections reconciled 2026-09-13 — archived tasks.json 6/6 each; the staleness was a rollup defect, not missing work). Suites: merge-threads 17/17, thread-contracts 14/14, dispatch-smoke 14/14. |
| G2 Lossless handoff (fact dossiers, inline content-addressed citations, verbatim quotes) | **MET** | Delivered across drt-002/003; exercised by the dispatch and contract suites. |
| G3 Multi-pass report (outline → parallel writers → deterministic assembly, claim-set invariant) | **MET** | drt-005 DONE; report-assembly suite 14/14. |
| G4 Budgets and failure semantics (job/director/thread budgets, force-complete, ledger for every dispatch) | **MET** | drt-002 DONE; failure paths covered by dispatch-smoke scenarios. |
| G5 Measurement (RACE + 3 FACT metrics, parity falsifiable) | **MET (partial 3-of-10 bench run; follow-up queued)** | The bench ran end-to-end: task-51 RACE 64.0 / verified_ratio 0.65, task-71 RACE 76.1 / 0.32, task-87 RACE 89.5 / 0.125. The drt-006 label-propagation defect is closed by tools/bench-automation/label-claims.py. Remaining 5/10 packages (58, 66, 79, 81, 85) await build in a follow-up change; the bench capability, the labeller, and the runbook are all in place. Onyx parity remains asserted-not-scored until all 10 packages are scored. |

## Delivered Changes

Parent (driven by claude-code via kbd-apply, 2026-09-08..09; archived):
- `change-drt-001-dispatch-smoke-and-thread-contracts` — DONE, archived
- `change-drt-007-install-surface-repair` — DONE, archived
- `change-drt-002-thread-scheduler-and-budgets` — DONE, archived
- `change-drt-004-deterministic-merge` — DONE, archived (projection reconciled)
- `change-drt-003-director-worker-agents` — DONE, archived (projection reconciled)
- `change-drt-005-multipass-report` — DONE, archived
- `change-drt-006-bench-and-metrics` — 3/4 tasks; task 4 BLOCKED (operator bench run); change active, unarchived — PASS WITH NOTES

Child (driven by opencode/glm-5.3, 2026-09-12..13; all archived, certification COMPLETE — drt-011 under operator waiver):
- `change-drt-008-hang-capture-harness` — instrumented repeat-run harness; the definitive negative result (0/98)
- `change-drt-009-stage10-hang-fix-and-certification` — distinct-judge diagnosis gate (accepted-with-unresolved at cap); branch none; N=60 certification + suite 128/128; --xtrace tripwire
- `change-drt-011-phase-close-distribution-regen` — C-01 reconciliation: deterministic regeneration, 405-file accounting, 4 checks PASS

## Technical Debt

- **drt-006 task 4 (RACE)** — blocked on the operator bench run (10 deep runs + judge calls); unblocks G5 to MET.
- **change-drt-010 (cut at child plan)** — child-phase `--target spec` CHANGES_ROOT bug in build-review-packet.sh + retirement of the phase-local symlink bridge (`.kbd-orchestrator/phases/deep-research-onyx-parity/changes/`).
- **Generator payload hygiene** — `tests/hang/captures*/` campaign artifacts mirror into `dist/plugins/**` (no exclusion rules in the generator).
- **Harness recorder semantics** — run-hang-capture.sh exits 0 on hangs/failures (by design; certification reads the table).
- **Projection defects** — exactNextCommand stale all child long (projected /kbd-assess); kbd-validate-progress Pending→Complete refusal required typed two-step transition; child-progress rollup inherited parent's COMPLETE summaries at spawn (recorded at assessment).
- **scoring-graph.sh** pre-existing closed-port stall (proven pre-edit, load-sensitive) — blocks "all suites green" claims until re-run unloaded.
- **Stages 04/05 label-propagation defect** (text drift between registry.json and credibility.json downgrades claims to unverified) — found by the FACT metrics, recorded, not patched.

## Architecture Integrity

- AGENTS.md violations: **NONE** — local-only validation throughout; no commits made (the parent's 551+-file tree awaits the operator's commit pass; reflect-protocol commit steps deliberately deferred).
- Constraints: C-01 **reconciled** (dist regenerated deterministically; final resync after the G2 addendum, 4 checks PASS); C-02 clean; C-03 N/A; C-04 idempotency proven (triple full-tree hash); C-05 clean.

## Artifact Quality Summary

| Metric | Value |
| --- | --- |
| Parent changes with QA | 7/7 (per their verification records) |
| Child changes with QA | 3/3 |
| Child diff-review revision rounds | 4 / 3 / 4+waiver |
| Judges | MiniMax-M3 ×6, gpt-5.5 (openai-proxy :8181) ×4, harness-native same-model ×2 (recorded) |
| Recurring constraint violations | none |

Recurring process defect (child, named): self-authored docs claiming more than code delivers — 2 of 3 changes; caught by review every time. Second: unfailable verify-strings, twice, both fixed with machine-countable structures.

## Cross-Tool Coordination Notes

- Progress tracking: RELIABLE at the ledger level; the rollup lagged reality for 003/004 (fixed by typed transitions) and the position projection's exactNextCommand was stale all child long.
- The multi-tool arc worked: claude-code drove the parent's seven changes; opencode/glm-5.3 drove the child's three through the identical gates (distinct judges, sycophancy screens, 2-round caps, waivers recorded rather than laundered).
- Judge infrastructure stabilized only after routing to gpt-5.5 via openai-proxy :8181 (gpt-5.4 advertised-but-rejected by the ChatGPT-plan Codex backend — verify a model live before wiring it).

## Lessons Learned

- Instrumented repeat-run campaigns retire intermittent-defect questions that serial theorizing cannot (six disproved theories → 0/98 in four campaigns).
- Mtime/git archaeology on an uncommitted tree can meet dating goals (child G2) that "no committed history" seemed to foreclose — proven introduced, dated, and never-remediated, all from mtimes + `git log --follow | wc -l`.
- Block-buffered stdout + external timeout kill destroys silence inferences; the fix is `--xtrace`-style instrumentation armed before the next recurrence.
- Diff-review packets need `files.txt` + `git add`, else a silent `git show HEAD` fallback reviews the wrong commit; samples must cover every generated tree.
- Proven-idempotent regeneration + staged outputs is the honest evidence form for generated-surface changes; demanding full generated contents in a judge packet does not fit any context window (the drt-011 dispute, waived by operator).
- Rollup staleness is a state-quality defect, not missing work — reconcile via typed transitions, never by hand-editing projections.

## Next Phase Focus

1. **Remaining 5/10 bench packages** (58, 66, 79, 81, 85): run the existing tool-3 labeller + bench-runbook on them when built; close the remaining gap in the BENCH-RESULTS.md mean.
2. **drt-010**: child-phase spec-packet fix + symlink-bridge retirement (small, surgical).
3. **Generator hygiene**: exclude campaign artifacts from the plugin payload; harness exit-semantics change.
4. **Parent commit pass** (operator): the 551+-file tree with C-01 proven in sync; includes the stages 04/05 label-propagation fix or its tracked deferral.

## Context for Next Phase

Use this file as prior context for the next `/kbd-assess`. The stage-10 hang is not reproduced on
identical code (0/98); any recurrence by any observation voids the child's certification per its
armed falsifier — hunt with `--xtrace`. Onyx parity remains asserted-not-scored until the bench run.
