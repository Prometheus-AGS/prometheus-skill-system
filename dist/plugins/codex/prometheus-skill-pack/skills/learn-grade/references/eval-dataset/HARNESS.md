# Grading Harness — Eval Invocation Protocol

`learn-grade` is prose-executed, not a callable script (confirmed in
`phase-learn-grader-validation`'s assessment.md). This document specifies
exactly how each of the 24 eval items in `index.json` is graded so the
process is reproducible and auditable, and exactly where the eval
invocation deviates from the live Feynman-loop invocation in
`learn-grade/SKILL.md`.

## Corpus shape

learn-grade reads `sources[].key_points[]` and `sources[].misconceptions[]`
(the shape `shared/scripts/content-grounding-kb.sh` emits, `schema_version`
1.1.0, change-rah-008). The three eval corpora are authored at schema 1.0.0:
every source carries `is_misconception` and `content_summary`, none carries
`misconceptions[]`, and only some carry an authored `key_points[]` (7 of 12 in
`cellular-respiration-corpus.json`; 0 of 18 and 0 of 16 in the two meta-corpora).
Before grading an item, normalize its corpus through the one grounding script
rather than re-deriving the fields by hand — normalization derives the missing
arrays and keeps the authored `key_points[]` verbatim, so the eval grades
against exactly what the live loop would see:

```bash
bash shared/scripts/content-grounding-kb.sh \
  --normalize <item.corpus_path> \
  --output /tmp/eval-corpus/<item_id>.json
```

and pass the normalized file as `corpus_path`. The eval corpus files
themselves are not rewritten; the harness normalizes at invocation time.

## Per-item invocation

For each item in `index.json`, an agent is given:

```
Grade this explanation using the learn-grade protocol
(skills/learn/learn-grade/SKILL.md, Steps 1-7 only — see deviations below).

concept_id: <domain>  (e.g. "kbd-lifecycle")
corpus_path: <item.corpus_path>
explanation: <item's explanation_text, verbatim>

Follow the SKILL.md's Step 3 four-dimension rubric exactly:
- completeness (0-1, continuous)
- accuracy (0-1, continuous; single factual error caps it below 0.7)
- clarity (0-1, continuous)
- misconceptions_absent (binary 0.0 or 1.0 — 1.0 only if NO entry of
  corpus sources[].misconceptions[] appears in the explanation)

Apply Step 5's anti-sycophancy check to the draft grade narrative.
Produce Step 6's gaps array for any dimension scoring below 0.7.
Produce Step 7's exactly-2 transfer problems from corpus key_points only.

Return ONLY the grade result JSON matching learn-grade/SKILL.md's
"Grade result schema" section, with grade_id = "eval-<item_id>",
goal_id = "eval", learner_id = "eval-harness", concept_id = <domain>.
```

## Deviations from the live SKILL.md protocol (and why)

| Step | Live behavior | Eval behavior | Why |
|---|---|---|---|
| Step 2 (semantic search, top-3) | Narrows the corpus to the 3 most relevant sources before grading | **Skipped — full corpus is passed.** Each eval corpus has only 12-18 sources total (small enough that no narrowing is needed), and skipping this step removes a second, un-measured judgment call from the harness. Narrowing to top-3 would risk excluding the exact misconception source_ref an item is designed to test. | Keeps the eval focused on measuring dimensions 3-7 rather also measuring semantic-search recall, which is out of scope for this phase. |
| Step 8 (learner-model observation RPC) | Sends `add_observation` to the `learner-model` JSON-RPC binary | **Skipped entirely.** No real `learner_id` exists in an eval context; writing synthetic eval grades would pollute the learner model with fake observations. | Eval items are not real learners; writing to the learner-model substrate would be a side effect with no correct undo. |
| Step 9 (write-grade.sh to `~/.prometheus/learn/goals/`) | Writes the grade JSON to the real learner's goal-scoped grade store | **Redirected — written to `references/eval-dataset/results/<item_id>.json` instead.** Same JSON schema, different destination. | Eval results must not be written into a real learner's grade history; they belong with the eval dataset itself for change-lgv-005's metrics computation. |

Steps 1, 3, 4, 5, 6, 7 run **exactly as specified** in `learn-grade/SKILL.md`
— no rubric changes, no threshold changes. This is the entire point of
the eval: measure the grader's real behavior on Steps 3-7, unmodified.

## Batching

24 items are independent — no item's grading depends on another's
result. Run via parallel `Agent` invocations (or a `Workflow`
`pipeline()`/`parallel()` fan-out) rather than 24 sequential manual
turns, per the assessment's cost/batching recommendation.

## Output contract

One file per item: `references/eval-dataset/results/<item_id>.json`,
matching `learn-grade/SKILL.md`'s grade result schema:

```json
{
  "grade_id": "eval-<item_id>",
  "goal_id": "eval",
  "concept_id": "<domain>",
  "learner_id": "eval-harness",
  "graded_at": "ISO datetime",
  "explanation_excerpt": "first 200 chars",
  "scores": {"completeness": 0.0, "accuracy": 0.0, "clarity": 0.0, "misconceptions_absent": 1.0},
  "overall_score": 0.0,
  "gaps": [{"dimension": "...", "description": "...", "corpus_ref": "..."}],
  "transfer_problems": ["...", "..."],
  "passed": false,
  "pass_threshold": 0.7
}
```

change-lgv-005 reads every file in `results/` and diffs `scores` against
each item's `ground_truth.scores` from its `explanations/*.json` file.
