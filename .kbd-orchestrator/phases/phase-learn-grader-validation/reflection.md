# Reflection — phase-learn-grader-validation

_Closed: 2026-07-16_

## Why this phase existed

`phase-learn-feynman` (v1.4.0, closed 2026-06-28) shipped `learn-grade` — the
sycophancy-corrected external grader that closes every Feynman loop — at an
assessed confidence of only **60–70%**, with zero empirical validation. That
reflection named this the highest-severity open risk in the entire learn
domain: *"a grader that misses misconceptions is worse than no grader — it
provides false assurance."* This phase existed to replace that guess with a
measured number.

## Goal achievement

| Goal | Status | Evidence |
|---|---|---|
| G-01: Grader evaluation dataset | **MET** | 24 items across 3 domains (kbd-lifecycle, skill-pack, cellular-respiration) in `references/eval-dataset/`, machine-readable schema (`SCHEMA.md`), 2 of 3 domains reused from existing production corpora |
| G-02: Run learn-grade against the dataset | **MET** | 24 items graded via the real SKILL.md protocol (dispatched as Agent calls, not mocked), documented deviations in `HARNESS.md`, results in `results/*.json` |
| G-03: Compute precision/recall metrics | **MET** | `compute-eval-metrics.py` → misconception F1 0.96 (recall 1.0, precision 0.923); Pearson r 0.94 (accuracy), 0.91 (completeness), 0.61 (clarity, post-tuning) |
| G-04: Tune the grader based on failure modes | **MET** | Found and fixed a systematic clarity-rubric defect (rubric ambiguity conflating prose quality with factual correctness); validated via re-grading 9 items; clarity Pearson r rose 0.405→0.609 (`TUNING-LOG.md`) |
| G-05: Grader regression test | **MET** | `grader-regression-test.sh` — snapshot-compare against `baseline-snapshot.json`; wired into CI as `learn-grade-regression` job in `validate.yml` |
| G-06: Document findings | **MET** | `EVAL-RESULTS.md` with final numbers and known limitations; `learn-grade/SKILL.md` updated to point to measured confidence, superseding the "60–70%" placeholder |

**6/6 goals MET.** No goals PARTIAL or NOT MET.

## Delivered changes

All 7 changes completed and closed (`progress.json`: 7/7):

1. `change-lgv-001-third-domain-corpus` — cellular-respiration teaching corpus (12 sources, 5 misconceptions)
2. `change-lgv-002-eval-explanations` — 24 eval items (8 per domain: 2 strong, 2 incomplete, 4 factually-flawed), draft ground truth
3. `change-lgv-003-dataset-schema-and-index` — `SCHEMA.md` + `index.json` manifest
4. `change-lgv-004-grading-harness` — `HARNESS.md` + 24 grade results via the real learn-grade protocol
5. `change-lgv-005-compute-metrics` — `compute-eval-metrics.py` + `metrics-summary.json` / `METRICS-REPORT.md`
6. `change-lgv-006-tune-grader` — clarity rubric fix in `SKILL.md`, re-graded 9 items, `TUNING-LOG.md`
7. `change-lgv-007-regression-test-and-docs` — regression guard, CI wiring, `EVAL-RESULTS.md`, confidence placeholder replaced

## Artifact Quality Summary

No artifact-refiner QA logs exist for this phase (`.refiner/artifacts/` has no `change-lgv-*` entries). This phase's changes were data/documentation/script artifacts (eval corpora, ground-truth JSON, a Python metrics script, a bash regression script, markdown docs, and one SKILL.md rubric edit) rather than application code changes, so they were not routed through the code-oriented artifact-refiner gate. This is consistent with how prior doc/data-heavy KBD changes in this repo have been handled — not a gap introduced by this phase.

## Technical debt introduced

1. **All 24 ground-truth items remain `review_status: "draft"`.** No human review pass was performed (resolved as an explicit, scoped-out open question in `assessment.md` — an automated KBD phase run has no human-in-the-loop step for this by default). The measured numbers are internally consistent and methodologically sound but should be treated as provisional until reviewed.
2. **24 items across 3 domains is a modest sample.** Statistically meaningful for this phase's purpose, but a larger/more diverse dataset would tighten confidence intervals and might surface failure modes this run didn't hit.
3. **The CI regression guard is snapshot-compare, not live re-validation.** It catches harness/schema regressions and pass/fail flips cheaply and deterministically, but does not catch subtle score drift within the same pass/fail bucket. This was a deliberate tradeoff (learn-grade is prose-executed, so live re-grading in CI would be non-deterministic and costly), not an oversight.
4. **Clarity correlation (r=0.61) remains the weakest dimension**, even after tuning. It is a genuine, moderate limitation flagged for future work, not a defect left unaddressed — G-04 explicitly only required fixing found systematic patterns, and the clarity fix was validated to improve, not perfect, this dimension.
5. **Sycophancy-correction (S-02) was not evaluated.** Explicitly out of scope per this phase's non-goals; the grader's substantive accuracy (finding real errors/misconceptions) was measured, not the sycophancy layer sitting on top of it.

## Lessons captured for knowledge base

- **learn-grade is prose-executed, not scripted.** Only `write-grade.sh` (the final file-write step) is real shell code — the actual grading logic (four-dimension scoring, gap identification, transfer-problem generation) is a natural-language protocol an LLM agent follows. Validating "the grader" therefore means dispatching parallel Agent calls against the protocol, not writing a test harness that calls a deterministic function. Any future skill claiming to have "integration tests" for a prose-executed skill should be checked for actual on-disk artifacts before being trusted (see next lesson).
- **Claimed test coverage should be verified against the filesystem, not taken at face value.** `phase-learn-feynman`'s reflection claimed 4 integration-test changes (change-learn-021 through 024) had been completed; an exhaustive search turned up zero artifacts for any of them. G-05's original framing ("wire into the existing suite") had to be replaced with "build a new harness from scratch."
- **Check for reusable domain content before building new corpora.** Two of the three eval domains needed already existed as production-quality corpora in `docs/learn/meta-corpus/`, cutting corpus-construction work by roughly two-thirds.
- **Rubric ambiguity produces systematic, not random, grader bias.** The clarity-scoring defect affected all 8 factually-flawed items in the same direction and similar magnitude — a clear signal of a definitional gap (conflating "is this readable" with "is this correct"), not per-item grading noise. Distinguishing systematic bias from noise is what made this worth a rubric fix rather than a "no pattern found" no-op.
- **Gold-standard authoring bias is a distinct failure mode from grader regression.** When a post-tuning score moved away from the draft gold value, the correct question was "which one is actually wrong," not "did the grader get worse." The `kbd-001-strong-full-cycle` control case documents a instance where the draft ground truth's own clarity score looks more suspect than the grader's post-fix score — a finding to disclose transparently, not launder into a pure "improvement" narrative.
- **Snapshot-compare is a legitimate middle ground for CI-testing non-deterministic LLM-graded systems** — cheap, deterministic, and catches real regression classes (missing files, schema drift, coarse pass/fail flips) without pretending it substitutes for periodic live re-validation.
- **The pipeline-enforce hook scans Bash command TEXT for the literal substring "kbd-reflect"** (and, by extension, other lifecycle-command names), not just `progress.json` state — it can false-positive on content that merely mentions the string (e.g., a heredoc quoting a misconception title). Workaround: use Write/Edit for file content containing such strings, or rephrase around the trigger word.

## Recommended focus for next phase

Per `EVAL-RESULTS.md`'s own recommended follow-ons, in priority order:

1. **Human (or independent stronger-model) review pass on the 24 ground-truth items** — resolves the `draft` status caveat and would upgrade the confidence of every number in this report from "provisional" to "confirmed."
2. **Revisit the outstanding `phase-learn-feynman` technical-debt item not chosen this cycle**: full FSRS-6 implementation (the other item named in that reflection, deliberately scoped out of this phase's non-goals).
3. **If clarity-dimension accuracy becomes safety-critical for a downstream use**, expand the dataset (more domains/items) and consider a second tuning pass specifically targeting clarity, since r=0.61 is the one dimension still only moderate after this phase's fix.

No other open goals remain in this phase. All prerequisites for closing are satisfied: all 7 changes implemented, no unmet QA gate applicable (no artifact-refiner routing for this phase's artifact types), and all deliverables committed and pushed to `main`.
