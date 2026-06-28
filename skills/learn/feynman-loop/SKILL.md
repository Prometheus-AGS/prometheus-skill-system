---
name: feynman-loop
description: The core Feynman learning cycle for prometheus-skill-pack. Maps Feynman's explain-grade-gap-relearn cycle to the PMPO lifecycle. Supports vertical recursion (child loops on gap concepts), horizontal escalation (novice→peer→skeptic audiences), recursion floor guards, and all three mastery closure criteria.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [learn, feynman, pmpo, loop, recursion, escalation, mastery]
---

# feynman-loop

## Overview

feynman-loop is the core Feynman learning cycle. It maps the Feynman Technique —
pick a concept, explain it plainly, identify gaps, re-learn, re-explain — onto
the PMPO lifecycle (Spec / Plan / Execute / Reflect). Each iteration produces a
graded explanation artifact, drives gap-targeted recursion, and gates closure on
three mastery criteria.

The loop is called by `/learn-plan` after a `curriculum.json` is produced, and
can also be invoked manually against any single concept.

## When to invoke

```
/feynman-loop --concept-id <id> --goal-id <id> --depth <N> [--audience novice|peer|skeptic]
```

| Argument | Required | Default | Description |
|---|---|---|---|
| `--concept-id` | yes | — | The concept to learn |
| `--goal-id` | yes | — | Parent learning goal |
| `--depth` | yes | — | Recursion depth (0 = top-level call) |
| `--audience` | no | `novice` | Target explanation level for this iteration |

## PMPO Mapping

The Feynman loop is a PMPO cycle instantiation:

| PMPO Phase | Feynman Step | What happens |
|------------|-------------|--------------|
| **Spec** | Pick concept + target depth | Load concept from learner model, set target depth and audience |
| **Plan** | Structure the explanation | Agent outlines: what to explain, what analogies to use, what the skeptic will challenge |
| **Execute** | Produce the explanation | Write a plain-language explanation, analogies, and a teach-the-skeptic pass |
| **Reflect** | Grade and identify gaps | Call `learn-grade`; identify gap concepts; decide: close loop or recurse |

## Flow

### Phase 1 — Spec

1. Load the concept state from the learner model:
   ```bash
   curl -s "${LEARNER_MODEL_URL:-http://localhost:7740}/api/v1/concepts/${CONCEPT_ID}"
   ```
2. Read `recursion_floor` from `~/.prometheus/learn/goals/<goal-id>/survey-result.json`
3. Set audience level from `--audience` (default: `novice` on the first iteration)
4. Log: `Starting feynman-loop — <concept-id> (depth <N>, audience <level>)`

### Phase 2 — Plan

Produce an explanation outline before writing the full explanation:

- **Core idea**: one sentence that captures the concept
- **Analogies**: 2–3 analogies appropriate for the audience level
  - `novice`: everyday comparisons, no assumed domain knowledge
  - `peer`: technical comparisons, domain vocabulary allowed
  - `skeptic`: steel-man the strongest objection, then refute it
- **Anticipated challenges**: for `peer` and `skeptic` audiences, list 2–3 things
  a knowledgeable reader would push back on

Present the outline to the user via `ui-surface` for early steering:
```bash
bash "${CLAUDE_PLUGIN_ROOT}/skills/learn/ui-surface/scripts/render.sh" \
  --type outline --content "${OUTLINE_JSON}"
```

### Phase 3 — Execute

Write the full explanation in plain language. The explanation must:

- Use the outline as the skeleton
- State the core idea first (no preamble, no "In this explanation I will…")
- Follow with analogies in order of complexity
- For `peer`/`skeptic`: end with a "Teach the skeptic" section that addresses
  anticipated objections and confirms they are resolved

Present the completed explanation to the user via `ui-surface`:
```bash
bash "${CLAUDE_PLUGIN_ROOT}/skills/learn/ui-surface/scripts/render.sh" \
  --type explanation --content "${EXPLANATION_JSON}"
```

Wait for the user to confirm, correct, or extend the explanation before grading.

### Phase 4 — Reflect

1. Call `learn-grade` with the final explanation text:
   ```
   /learn-grade \
     --concept-id "${CONCEPT_ID}" \
     --learner-id "${LEARNER_ID:-default}" \
     --corpus-path "${CORPUS_PATH}" \
     --explanation "${EXPLANATION_TEXT}" \
     --goal-id "${GOAL_ID}"
   ```

2. Receive the grade result (passed, overall_score, gaps, transfer_problems, grade_id)

3. Apply the recursion logic (see below) to any gaps

4. If all gaps are resolved or noted, present the transfer problems to the learner

5. Evaluate transfer problem responses (score >= 0.7 to pass)

6. If mastery closure criteria are all met, write the artifact and emit the handoff

## Recursion logic (vertical)

When `learn-grade` returns gaps, for each gap concept:

```
if concept_id in recursion_floor:
    log "[feynman-loop] concept is on recursion floor — not recursing"
    record gap in learner model as "noted for review"
    continue

if current_depth >= 3:
    log "[feynman-loop] max recursion depth reached — gap noted for review"
    record gap in learner model as "noted for review"
    continue

# Spawn child loop
/feynman-loop \
  --concept-id <gap.concept_id> \
  --goal-id "${GOAL_ID}" \
  --depth $((current_depth + 1)) \
  --audience novice
```

After all child loops complete:
- Re-run Phase 3 (Execute) for the parent concept, incorporating what was learned
  in child loops
- Re-run Phase 4 (Reflect) with the updated explanation

### Recursion floor guard

Check this before any recursion:

```bash
SURVEY_PATH="${HOME}/.prometheus/learn/goals/${GOAL_ID}/survey-result.json"
FLOOR=$(jq -r '.recursion_floor[]?' "$SURVEY_PATH" 2>/dev/null)

if echo "$FLOOR" | grep -qx "${GAP_CONCEPT_ID}"; then
  echo "[feynman-loop] concept ${GAP_CONCEPT_ID} is on recursion floor — not recursing"
fi
```

## Horizontal escalation

After a concept loop closes at the `novice` audience level, the plan may specify
escalation to `peer` and `skeptic`:

1. Re-invoke feynman-loop with `--audience peer` (same concept, same depth)
2. After `peer` closes, re-invoke with `--audience skeptic`
3. Each escalation is a full Spec / Plan / Execute / Reflect cycle
4. Escalation is optional — the curriculum plan controls which levels are required
   for the target proficiency rating

Horizontal escalation does NOT increase `--depth`. It runs at the same depth as
the original call.

## Mastery closure criteria

A concept loop closes ONLY when all three criteria are met:

| # | Criterion | How it is checked |
|---|---|---|
| 1 | **Grade passes** | `overall_score >= 0.7` AND `misconceptions_absent == 1.0` from `learn-grade` |
| 2 | **Transfer problems solved** | User solves both novel transfer problems with score >= 0.7 each |
| 3 | **Retention scheduled** | Loop records `retention_scheduled: true`; `learn-retain` performs the check at >= 24h |

Self-reported fluency ("I feel like I understand this") never closes a loop.

When criteria 1 and 2 are met, the loop closes and marks retention as scheduled.
The actual retention check is deferred to `/learn-retain`.

## Feynman artifact

After a loop closes, write the artifact using `scripts/write-artifact.sh`:

```bash
bash "${CLAUDE_PLUGIN_ROOT}/skills/learn/feynman-loop/scripts/write-artifact.sh" \
  --goal-id "${GOAL_ID}" \
  --artifact-json "${ARTIFACT_JSON}"
```

Artifact schema written to
`~/.prometheus/learn/goals/<goal-id>/artifacts/<artifact-id>.json`:

```json
{
  "artifact_id": "string",
  "goal_id": "string",
  "concept_id": "string",
  "depth": 0,
  "audience": "novice|peer|skeptic",
  "explanation_text": "string",
  "grade_id": "string",
  "overall_score": 0.0,
  "transfer_scores": [0.0, 0.0],
  "retention_scheduled": true,
  "child_loops": ["artifact_id"],
  "closed_at": "ISO datetime"
}
```

`artifact_id` is generated as `artifact-<concept-id>-<audience>-<depth>-<unix-timestamp>`.

## Handoff

When all required audience levels for a concept close:

```
Concept mastered: <concept-id> (depth <N>, <audience> level)
Transfer problems: <score1>, <score2>
Retention check scheduled: yes
Next: /feynman-loop --concept-id <next-concept> or /learn-certify <goal-id>
```

The calling skill or user decides whether to proceed to the next concept in the
curriculum or invoke `/learn-certify` to close the goal.

## Error handling

| Condition | Behavior |
|---|---|
| Learner model unreachable | Log warning, continue; use empty concept state |
| Corpus path missing | Abort with error: `corpus not found at <path>` |
| `survey-result.json` missing | Treat recursion floor as empty; log warning |
| `ui-surface` unavailable | Print explanation directly to stdout |
| `learn-grade` fails | Retry once; on second failure, surface error to user |

## Directory layout

```
skills/learn/feynman-loop/
├── SKILL.md              — this file
└── scripts/
    └── write-artifact.sh — writes artifact JSON to ~/.prometheus/learn/goals/<goal-id>/artifacts/
```
