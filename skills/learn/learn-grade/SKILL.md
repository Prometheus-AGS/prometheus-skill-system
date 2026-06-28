---
name: learn-grade
description: External, source-grounded, anti-sycophantic grader for the Feynman learning loop. Checks explanations against the teaching corpus, identifies gaps, generates novel transfer problems, and updates the learner model. Routes through sycophancy-correction to prevent pedagogical flattery.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [learn, grade, assessment, feynman, sycophancy, corpus-grounded, transfer]
---

# learn-grade

## Overview

learn-grade is the shared, corpus-grounded grader for all assessment skills in the
Feynman learning loop. It grades a learner's explanation against the teaching corpus
on four dimensions, enforces anti-sycophancy via `sycophancy-correction`, identifies
specific gaps, and generates novel transfer problems drawn from the corpus rather than
the learner's own words.

The grader is called as a sub-step by feynman-loop, learn-practice, learn-retain, and
learn-certify. It is not typically invoked directly by the user — though it may be.

## When to invoke

- **Internally** — called by feynman-loop, learn-practice, learn-retain, learn-certify
  after the learner submits an explanation or answer.
- **Manually** — invoke `/learn-grade` directly to assess any freeform explanation of
  a concept against the corpus without running the full feynman-loop.

## Interface

```
/learn-grade \
  --concept-id <id> \
  --learner-id <id> \
  --corpus-path <path> \
  --explanation "<text>" \
  [--goal-id <id>]
```

| Argument | Required | Description |
|---|---|---|
| `--concept-id` | yes | Identifier for the concept being graded |
| `--learner-id` | yes | Learner identifier for model update |
| `--corpus-path` | yes | Absolute path to the corpus JSON produced by content-grounding.sh |
| `--explanation` | yes | The learner's explanation text |
| `--goal-id` | no | Goal context for grade file placement; defaults to `default` |

## Grading flow

### Step 1 — Load corpus

Read the corpus JSON from `--corpus-path`. The expected shape matches the output of
`shared/scripts/content-grounding.sh`:

```json
{
  "concept_id": "string",
  "sources": [
    {
      "source_ref": "string",
      "content_summary": "string",
      "key_points": ["string"],
      "misconceptions": ["string"]
    }
  ]
}
```

### Step 2 — Semantic search

Find the 3 most relevant corpus entries for the concept being graded. Match by:
1. Exact `concept_id` lookup in `sources[].concept_id` where present
2. Keyword overlap between `content_summary` and `--concept-id`
3. Key-point keyword matching against the explanation

### Step 3 — Grade on four dimensions

Score the explanation on a 0–1 scale for each dimension:

| Dimension | Key | Description |
|---|---|---|
| Completeness | `completeness` | Does the explanation cover all key aspects present in the corpus? |
| Accuracy | `accuracy` | Is the explanation factually correct vs the corpus? Flag specific errors. |
| Clarity | `clarity` | Is it understandable to the stated target level? |
| Misconceptions absent | `misconceptions_absent` | Does it contain any known misconceptions from the corpus? Score 1.0 means none detected. |

For **accuracy**: identify each factual discrepancy against corpus key points. A single
factual error caps accuracy below 0.7 regardless of how much else is correct. Always
name the specific error — never describe it vaguely.

For **misconceptions_absent**: if any misconception from `sources[].misconceptions` appears
in the explanation, score 0.0. Otherwise score 1.0. There is no partial credit on this
dimension — misconceptions must be absent entirely to pass.

### Step 4 — Compute overall score

```
overall_score = mean(completeness, accuracy, clarity, misconceptions_absent)
```

### Step 5 — Anti-sycophancy check

Before writing the grade, apply the S-02 sycophancy-correction pattern to the draft
grade narrative:

> "Does this grade say 'no gaps' or 'excellent explanation' when gaps are present?
> Does it soft-pedal a factual error to protect the learner's feelings?"

If the draft grade is sycophantic:
- Remove any praise that follows a gap identification
- Lead with errors and gaps — not with compliments
- Rewrite hedging language ("minor issue", "just a small point") to direct language
  ("factual error", "missing coverage of X")

Use the `sycophancy-correction` MCP server if available. Otherwise apply the check
manually using the prose rules above.

### Step 6 — Identify gaps

For each dimension scoring below 0.7, produce a GapRecord:

```json
{
  "dimension": "completeness|accuracy|clarity|misconception",
  "description": "What is missing or wrong — specific, not vague",
  "corpus_ref": "source_ref value from the relevant corpus source"
}
```

Gaps are returned to the calling skill to target the next Feynman iteration.

### Step 7 — Generate transfer problems

From the corpus (NOT from the learner's explanation), generate exactly 2 novel
transfer problems. A transfer problem tests whether the learner can apply the concept
in a new context they have not already encountered.

Rules:
- Use concepts, examples, and contexts from `sources[].key_points` only
- Do not repeat phrasing from the learner's explanation
- Each problem must require genuine application, not recall

### Step 8 — Update learner model

Call the learner-model substrate's `add_observation` endpoint:

```bash
curl -s -X POST "${LEARNER_MODEL_URL}/api/v1/observations" \
  -H "Content-Type: application/json" \
  -d "{
    \"learner_id\": \"${LEARNER_ID}\",
    \"concept_id\": \"${CONCEPT_ID}\",
    \"score\": ${OVERALL_SCORE},
    \"source_skill\": \"learn-grade\",
    \"observed_at\": \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"
  }"
```

`LEARNER_MODEL_URL` defaults to `http://localhost:7740` (the substrate/learner-model
default port). When the learner-model substrate is unavailable, log a warning and
continue — the grade file is the primary output.

### Step 9 — Emit grade result

Write the grade JSON using `scripts/write-grade.sh`:

```bash
bash "${CLAUDE_PLUGIN_ROOT}/skills/learn/learn-grade/scripts/write-grade.sh" \
  --goal-id "${GOAL_ID}" \
  --grade-json "${GRADE_JSON}"
```

## Grade result schema

```json
{
  "grade_id": "string",
  "goal_id": "string",
  "concept_id": "string",
  "learner_id": "string",
  "graded_at": "ISO datetime",
  "explanation_excerpt": "first 200 chars of the explanation",
  "scores": {
    "completeness": 0.0,
    "accuracy": 0.0,
    "clarity": 0.0,
    "misconceptions_absent": 1.0
  },
  "overall_score": 0.0,
  "gaps": [
    {
      "dimension": "completeness|accuracy|clarity|misconception",
      "description": "string",
      "corpus_ref": "source_ref from corpus"
    }
  ],
  "transfer_problems": ["string", "string"],
  "passed": false,
  "pass_threshold": 0.7
}
```

`grade_id` is generated as `grade-<concept_id>-<unix_timestamp>`.

## Pass criterion

An explanation **passes** if both conditions hold:

1. `overall_score >= 0.7`
2. `misconceptions_absent == 1.0` (no misconceptions detected)

When the explanation does not pass, the `gaps` array is returned to the calling skill
to focus the next Feynman iteration on those specific concepts. The calling skill
must not suppress or paraphrase the gaps — it must surface them verbatim to the
learner.

## Anti-sycophancy mandate

The grader must surface gaps even when the overall score is high. A specific gap in
accuracy is always reported, even if the rest of the explanation is excellent.

Never structure feedback as: "Great explanation, a few minor notes…"

Always structure feedback as: "Factual error in X: <specific error>. Also missing
coverage of Y per corpus_ref Z."

Pedagogical sycophancy — making the learner feel good at the cost of accurate feedback
— produces worse learning outcomes. The grade is for the learner's benefit, not for
the author's comfort. This is enforced structurally via the S-02 sycophancy-correction
check on every draft grade.

## Output to calling skill

learn-grade returns to the calling skill:

```json
{
  "passed": true|false,
  "overall_score": 0.0,
  "gaps": [...],
  "transfer_problems": ["...", "..."],
  "grade_id": "string",
  "grade_path": "absolute path to grade file"
}
```

## Directory layout

```
skills/learn/learn-grade/
├── SKILL.md          — this file
└── scripts/
    └── write-grade.sh — writes grade JSON to ~/.prometheus/learn/goals/<goal-id>/grades/
```
