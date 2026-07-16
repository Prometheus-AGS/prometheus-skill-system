# Goals — phase-learn-grader-validation

## Context

`phase-learn-feynman` (v1.4.0, closed 2026-06-28) shipped `learn-grade` —
the sycophancy-corrected external grader that closes each Feynman loop —
at only 60–70% assessed confidence. That reflection named this the
**highest-severity open risk** in the entire learn domain: "a grader that
misses misconceptions is worse than no grader — it provides false
assurance."

No empirical validation dataset exists. The grader has never been tested
against explanations with known, expert-labeled gaps. This phase builds
that dataset and measures actual precision/recall.

## Goals

- [ ] **G-01: Grader evaluation dataset** — assemble 20+ Feynman
  explanations spanning at least 3 subject domains (e.g., one STEM
  topic, one humanities topic, one technical/programming topic). Each
  explanation gets expert-authored ground-truth annotations: which
  misconceptions are present, which are absent, and a gold-standard
  score. Store as `skills/learn/learn-grade/references/eval-dataset/`
  with a machine-readable schema (JSON or YAML per explanation).

- [ ] **G-02: Run `learn-grade` against the dataset** — script that
  feeds each explanation through the actual `learn-grade` skill/script
  path (not a mock), captures the grader's score + misconception list,
  and diffs against ground truth.

- [ ] **G-03: Compute precision/recall metrics** — for misconception
  detection: precision (of flagged misconceptions, how many are real),
  recall (of real misconceptions, how many were flagged), and overall
  score correlation (Pearson/Spearman) between grader score and
  human gold-standard score.

- [ ] **G-04: Tune the grader based on failure modes** — for any
  systematic failure pattern found in G-03 (e.g., misses subtle
  misconceptions, over-flags correct-but-unusual phrasing), adjust the
  `learn-grade` system prompt / grading rubric and re-run G-02 to
  confirm improvement.

- [ ] **G-05: Grader regression test** — add the eval dataset run as a
  repeatable test (shell script or integration test) so future changes
  to `learn-grade` can be checked against the same ground truth before
  shipping. Wire into the existing learn-domain integration test suite
  (`change-learn-021` through `change-learn-024` established the
  pattern).

- [ ] **G-06: Document findings** — write up final precision/recall
  numbers, confidence level achieved, and any remaining known failure
  modes in `skills/learn/learn-grade/references/EVAL-RESULTS.md`.
  Update the learn domain's confidence assessment in project memory /
  docs from "60–70%" to the measured number.

## Non-goals

- **Full FSRS-6 implementation** — separate technical debt item from
  the same reflection; not in scope here.
- **Sycophancy-correction S-02 changes** — the sycophancy gate that
  routes `learn-grade` output is out of scope; this phase measures the
  grader's substantive accuracy, not the sycophancy-correction layer.
- **New grading rubric dimensions** — this phase validates and tunes
  the *existing* rubric; it does not redesign what the grader measures.
