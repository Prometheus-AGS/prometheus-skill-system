---
name: learn-survey
description: Diagnostic placement skill for the Feynman learning loop. Generates dynamic survey items from the teaching corpus, probes current knowledge state and misconceptions, sets the recursion floor, and seeds the learner model via the learner-model crate.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [learn, survey, diagnostic, placement, learner-model, misconceptions]
---

# learn-survey

Diagnostic placement for the Feynman learning loop. Assess what the learner
already knows, detect misconceptions, set the recursion floor, and seed the
learner model before any curriculum or Feynman loop begins.

## When to invoke

Run after `/learn-goal` succeeds and a goal artifact exists. Invoke as:

```
/learn-survey <goal-id>
```

The `goal-id` references the artifact at
`~/.prometheus/learn/goals/<goal-id>/goal.json`.

## Flow

### 1. Load goal artifact

Read `~/.prometheus/learn/goals/<goal-id>/goal.json`. Confirm the goal has
status `ready` and a `corpus_path` field. If either is missing, abort with a
clear error.

### 2. Load corpus

Read the corpus JSON from `goal.corpus_path`. The corpus contains a `sources`
array of concept entries with `concept_id`, `title`, and content fields.

### 3. Generate diagnostic items

Generate exactly 11 items from the corpus sources — never invented, always
grounded in corpus content:

| Type | Count | Form |
|------|-------|------|
| `conceptual` | 5 | Open explain/define questions on key concepts |
| `procedural` | 3 | How-would-you / apply-this questions |
| `misconception_probe` | 3 | Present a common wrong statement; ask true / false / not-sure |

Each item must reference a `concept_id` from the corpus. Assign unique
`item_id` values (`item-001` … `item-011`).

### 4. Present survey

Render items via `ui-surface` (Tier 1 preferred). Present one at a time or
grouped by type. Wait for the learner's response to each item before advancing.
Do not reveal correct answers during the survey.

### 5. Score responses

Score each response using the rubric below. Do not round-trip to the learner
during scoring.

**Conceptual and procedural items:**

| Score | Criterion |
|-------|-----------|
| 1.0 | Correct with accurate reasoning or demonstration |
| 0.5 | Partially correct — right idea, missing precision |
| 0.0 | Absent, wrong, or "I don't know" |

**Misconception probes:**

| Score | Criterion |
|-------|-----------|
| 1.0 | Correctly identifies the statement as a misconception |
| 0.0 | Accepts the statement as true, or answers "not sure" |

### 6. Detect recursion floor

The recursion floor is the set of concepts the learner already owns. A
concept enters the floor when:

- Its aggregated survey score is ≥ 0.7, AND
- No misconception probe for that concept scored 0.0

Floor concepts are included in the learner model with high initial mastery
but are never the target of a new Feynman loop. The Feynman loop will not
recurse into them.

Aggregate the score for each concept across all items that reference it.
When a concept has multiple items, use the mean of their scores as the
estimated mastery prior.

### 7. Compute mastery priors

For each concept in the corpus, produce a `mastery_priors` entry:

```json
{
  "concept_id": "string",
  "estimated_mastery_prior": 0.0,
  "confidence": 0.7,
  "basis": "survey_response"
}
```

Concepts with no survey items get `estimated_mastery_prior: 0.0` and
`confidence: 0.3` (low confidence, uninformed prior).

### 8. Write survey result

Call `scripts/write-survey-result.sh` to persist the result:

```bash
bash skills/learn/learn-survey/scripts/write-survey-result.sh \
  --goal-id "<goal-id>" \
  --result-json '<survey-result-json>'
```

The script writes to `~/.prometheus/learn/goals/<goal-id>/survey-result.json`.

### 9. Seed learner model

After writing the survey result, convert it to the learner-model seed schema and
send the `seed_from_survey` JSON-RPC method. `GOAL_PATH` points to the matching
`goal.json` and supplies the subject when it is not repeated in the survey:

```bash
SEED_JSON=$(jq --slurpfile goal "$GOAL_PATH" '{
  schema_version: "1.0.0",
  learner_id: (.learner_id // .goal_id),
  subject: (.subject // $goal[0].subject),
  surveyed_at: .surveyed_at,
  mastery_priors: .mastery_priors,
  recursion_floor: (.recursion_floor // []),
  misconceptions_detected: [
    .misconceptions_detected[]? |
    if type == "object" then . else {
      concept_id: .,
      wrong_model: "Detected during learn-survey",
      source_evidence: "survey-result.json"
    } end
  ]
}' "$SURVEY_PATH")

jq -nc --argjson seed "$SEED_JSON" \
  '{method:"seed_from_survey",params:{seed:$seed}}' | learner-model
```

If the `learner-model` binary is not on PATH, emit a warning and skip this
step — the survey result file is still written and the handoff proceeds.

## Survey result schema

```json
{
  "goal_id": "string",
  "surveyed_at": "ISO datetime",
  "items_presented": 11,
  "item_responses": [
    {
      "item_id": "item-001",
      "item_type": "conceptual|procedural|misconception_probe",
      "concept_id": "string",
      "response": "string",
      "score": 0.0
    }
  ],
  "mastery_priors": [
    {
      "concept_id": "string",
      "estimated_mastery_prior": 0.0,
      "confidence": 0.7,
      "basis": "survey_response"
    }
  ],
  "recursion_floor": ["concept-id-1", "concept-id-2"],
  "misconceptions_detected": ["string"]
}
```

## Recursion floor rule

The recursion floor prevents the Feynman loop from recursing into concepts
the learner already owns. A concept is on the floor if:

- Its survey score is ≥ 0.7, AND
- No misconceptions were detected for it in the misconception probes

Floor concepts are included in the learner model with high initial mastery
but are never the target of a new Feynman loop.

## Honesty rule

Never tell the learner they did well when they did not. Survey results go
through the same anti-sycophancy principle as `learn-grade`: if scores are
low, say so plainly with specific gaps identified. Do not soften or hedge
a low-scoring result with compliments.

Examples of banned phrasings:
- "Great effort!" when all scores are 0.0
- "You have a solid foundation" when the recursion floor is empty
- "That's mostly right" when the score is 0.0

State the result plainly: which concepts were assessed, what was demonstrated,
what was absent.

## Handoff

After writing the survey result, print:

```
Survey complete: <N> concepts assessed, <M> on recursion floor
Misconceptions detected: <list or "none">
Next: /learn-plan <goal-id>
```

Replace `<N>` with the total concept count from the corpus, `<M>` with the
count of concepts on the recursion floor, and the misconceptions list with
the `misconceptions_detected` array values (comma-separated) or "none".

## Error handling

| Condition | Action |
|-----------|--------|
| `goal.json` missing | Abort: "Goal not found: <goal-id>. Run /learn-goal first." |
| `corpus_path` absent or unreadable | Abort: "Corpus not found at <path>. Re-run /learn-goal." |
| Corpus has fewer than 4 concepts | Proceed but note the limited diagnostic coverage in handoff |
| `write-survey-result.sh` fails | Surface the error; do not proceed to seeding |
| `learner-model` binary absent | Warn and skip seeding; survey result is still written |
