# Assessment — phase-learn-grader-validation

_Assessed: 2026-07-16_

## Project identity

`prometheus-skill-pack` — enterprise skill collection. `.kbd-orchestrator/project.json`
is absent (noted, non-blocking — same as prior phases). Change backend:
OpenSpec (`openspec/` present at project root).

## Executive summary

`learn-grade` (`skills/learn/learn-grade/SKILL.md`, 257 lines, v1.0.0) is a
**prose-executed grading protocol**, not a deterministic script. Only the
final write step (`write-grade.sh`) is real shell code; grading itself
(corpus load, dimension scoring, gap detection, transfer-problem
generation) is a 9-step instruction set an agent follows. This is the
single most important fact for this phase's design: **"run learn-grade
against the dataset" means invoking an agent once per eval item**, not
calling a CLI.

The four "integration test" changes from `phase-learn-feynman`
(`change-learn-021` through `change-learn-024`, all marked `DONE` in that
phase's `progress.json`) have **no artifacts on disk anywhere in the
repo** — no test files, no feature files, no scripts matching their
titles ("basic flow", "full loop", "KB", "meta-skills"). This is a
material assessment finding: G-05 ("wire into the existing learn-domain
integration test suite") has no existing suite to wire into. The suite
either never shipped, or shipped and was later removed without a
corresponding KBD update. Either way, **G-05 must build a new
minimal test harness, not extend one.**

## Existing surfaces (what's already here)

| Path | State | Notes |
|------|-------|-------|
| `skills/learn/learn-grade/SKILL.md` | Present, v1.0.0, 257 lines | Full 9-step grading protocol, 4-dimension rubric, pass criterion, schema |
| `skills/learn/learn-grade/scripts/write-grade.sh` | Present | Only real script — writes a grade JSON to `~/.prometheus/learn/goals/<goal-id>/grades/`. Does NOT grade. |
| `skills/learn/learn-grade/references/` | **ABSENT** | Target dir for G-01 (eval dataset) and G-06 (EVAL-RESULTS.md) does not exist yet |
| `shared/scripts/content-grounding.sh` | Present | Produces the corpus JSON shape `learn-grade` expects: `{concept_id, sources: [{source_ref, source_type, confidence, is_misconception, content_summary, key_points?, misconceptions?}]}` |
| Integration tests (`change-learn-021..024`) | **NOT FOUND ON DISK** | No test files, feature files, or scripts anywhere under `skills/learn/`, `substrate/learner-model/`, `substrate/storage-provider/`, or repo root matching these titles |
| `sycophancy-correction` MCP server | Present (used elsewhere in repo) | `learn-grade` step 5 references it as optional ("if available... otherwise apply manually") |

## Grading rubric (exact, from SKILL.md)

| Dimension | Key | Scoring |
|---|---|---|
| Completeness | `completeness` | 0.0–1.0 continuous |
| Accuracy | `accuracy` | 0.0–1.0 continuous; single factual error caps it below 0.7 |
| Clarity | `clarity` | 0.0–1.0 continuous |
| Misconceptions absent | `misconceptions_absent` | **Binary** — 1.0 if none of `sources[].misconceptions` appear in the explanation, else 0.0. No partial credit. |

`overall_score = mean(all four)`. `pass_threshold = 0.7`.

This has a direct design consequence for **G-03** (precision/recall):
`misconceptions_absent` is a binary classification (misconception
present/absent) — precision/recall in the traditional sense applies
cleanly here. `completeness`/`accuracy`/`clarity` are continuous scores,
where **score correlation** (Pearson/Spearman vs human gold-standard) is
the right metric, not precision/recall. The goals.md already reflects
this ("score correlation" language in G-03) — no gap, just confirming
the design is internally consistent.

## Gap analysis vs G-01 through G-06

### G-01 — Grader evaluation dataset
**Status: NOT MET.** `skills/learn/learn-grade/references/eval-dataset/`
does not exist. No prior eval explanations, annotated or otherwise, exist
anywhere in the repo.
- **Action:** Build 20+ explanations × 3+ domains. Each item needs: the
  explanation text, a matching corpus JSON (content-grounding.sh shape),
  and expert-annotated ground truth (misconceptions present/absent,
  gold-standard scores per dimension).
- **Open question:** who authors "expert" ground truth in an automated
  KBD phase? See Open Questions below.

### G-02 — Run learn-grade against the dataset
**Status: NOT MET — and requires a design decision.** Since grading is
agent-executed prose (not a script), "running" the grader for 20+ items
means 20+ agent invocations following the SKILL.md protocol verbatim, with
the corpus and explanation as inputs. This is naturally parallelizable
(each grading is independent) and is a strong fit for `Workflow`-style
fan-out if the user wants exhaustive coverage, or sequential `Agent` calls
for a lighter run.
- **Action:** Write a harness script that, for each eval item, packages
  `{concept_id, corpus_path, explanation}` and either (a) invokes an agent
  per item, or (b) documents the manual protocol precisely enough that
  each grading is reproducible and auditable.

### G-03 — Compute precision/recall metrics
**Status: NOT MET (no dataset, no results yet).** Design is sound per the
rubric analysis above: binary precision/recall for `misconceptions_absent`
gap-detection; Pearson/Spearman correlation for the three continuous
dimensions vs their gold-standard counterparts.
- **Action:** Write `scripts/compute-eval-metrics.py` (or `.sh` with `jq`
  aggregation) once G-02 results exist.

### G-04 — Tune the grader on failure modes
**Status: NOT MET (depends on G-03 results).** No changes needed yet —
this is inherently a second-pass activity once real failure data exists.

### G-05 — Grader regression test
**Status: NOT MET, and scope is larger than goals.md implied.** The
referenced "existing learn-domain integration test suite" does not exist.
This phase must:
1. Build a minimal test harness (not "extend" one)
2. Decide the harness's invocation model given G-02's agent-executed
   nature — likely a script that replays the *recorded* grades from the
   initial G-02 run and checks for score drift, rather than re-invoking
   an LLM agent on every CI run (cost + determinism concerns)
- **Open question:** should regression testing re-run the LLM grader
  live (expensive, non-deterministic) or snapshot-compare against last
  known-good grades (cheap, catches prompt/schema regressions but not
  live model drift)? Recommend the latter as the CI-safe default, with
  the former as an optional manual re-validation script.

### G-06 — Document findings
**Status: NOT MET (depends on all prior goals).** Straightforward once
G-01 through G-05 produce real numbers.

## Open questions

1. **Ground-truth authorship.** "Expert-authored" annotations for 20+
   explanations need a human-in-the-loop step, or the user acting as the
   domain expert during dataset construction, or Claude producing
   candidate annotations that the user reviews/corrects. Recommend:
   Claude drafts explanations + candidate misconceptions per domain,
   user (or a stronger-model second pass) reviews and corrects before
   the dataset is treated as ground truth. Flag clearly in plan.md that
   this is NOT fully automatable without human review if "expert"
   fidelity is to mean anything.

2. **Live-agent vs snapshot regression testing (G-05).** See above —
   recommend snapshot-compare as the CI-safe default.

3. **Domain selection for the 3+ eval domains.** goals.md suggests
   "STEM, humanities, technical/programming" as examples. Recommend
   picking domains where the existing meta-grounding corpus (from
   `change-learn-016`, "Meta-grounding corpus for KBD + skill pack") can
   partially seed content, reducing the corpus-construction burden. Needs
   verification during /kbd-plan whether that meta-corpus exists on disk
   and in what shape.

4. **Cost/scope of G-02's agent invocations.** 20+ real grading passes
   through an LLM agent is a real time/token cost. Recommend batching via
   the `Agent` tool or a `Workflow` fan-out rather than 20+ sequential
   manual turns.

## Recommended focus for /kbd-plan

**Change ordering (proposed for /kbd-plan to refine):**

1. Verify meta-grounding corpus availability (quick check, informs #2)
2. Build eval dataset: explanations + corpora + draft ground truth (G-01)
3. Human/second-pass review of ground truth annotations (G-01 completion)
4. Harness + run: invoke learn-grade against all dataset items, capture
   results (G-02)
5. Compute metrics: precision/recall (misconceptions) + correlation
   (other 3 dims) (G-03)
6. Tune grader based on failure analysis; re-run affected items (G-04)
7. Build snapshot-based regression test + wire into CI or a runnable
   script (G-05)
8. Write EVAL-RESULTS.md, update confidence assessment (G-06)

Estimated 6-8 changes. Analyze stage optional — this phase is mostly
content construction + measurement, not library adoption; recommend
skipping `/kbd-analyze` unless the user wants research into existing
LLM-eval-harness patterns (e.g., promptfoo, DeepEval) as prior art for
G-05's harness design.
