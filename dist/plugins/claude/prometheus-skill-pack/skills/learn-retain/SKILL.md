---
name: learn-retain
description: Spaced repetition review skill for the Feynman learning loop. Reads the FSRS due queue from the learner-model crate, surfaces review prompts via ui-surface, grades retention via learn-grade at a 0.6 threshold, and updates FSRSCard state. Prevents knowledge decay after feynman-loop and learn-practice.
version: '1.0.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [learn, retain, spaced-repetition, fsrs, review, memory]
---

# learn-retain

Spaced repetition review skill for the Feynman learning loop. Works with
`learner-model`, `ui-surface`, and `learn-grade` to run FSRS-scheduled review
sessions that keep acquired knowledge above the retention threshold.

## When to invoke

Run on any session start when due cards exist, OR when `feynman-loop` records
`retention_scheduled: true` in an artifact.

```
/learn-retain <goal-id> [--concept-id <id>] [--max-cards N]
```

- Without `--concept-id`: reviews ALL due concepts for the goal.
- Without `--max-cards`: defaults to 5 cards per session.

## Flow

### 1. Load due queue

Call `learner-model load` for the learner. Filter concepts where
`fsrs_card.due <= now`. Sort by `fsrs_card.due` ascending (most overdue first).

### 2. Cap the session

Take up to `--max-cards` (default 5) concepts from the sorted due list.

### 3. For each due concept

**a. Load artifact**

Load the most recent Feynman artifact for the concept:
```
artifacts/<concept-id>-*.json
```

**b. Surface review prompt via ui-surface**

Do NOT show the original explanation. Show only the concept label:

```
Re-explain [concept] in your own words. You have 3 minutes.
```

**c. Collect the user's explanation**

Wait for the user's free-recall response.

**d. Grade via learn-grade**

```bash
/learn-grade --corpus-path <goal-artifact-corpus-path> \
             --response "<user explanation>"
```

**e. Apply FSRS rating**

Map the numeric grade to an FSRS `Rating` enum value:

| Grade range | Rating |
|-------------|--------|
| ≥ 0.8       | `Rating::Easy` |
| 0.6–0.79    | `Rating::Good` |
| 0.4–0.59    | `Rating::Hard` |
| < 0.4       | `Rating::Again` |

`Rating::Again` schedules an immediate re-review (next due: 1 day).

**f. Update FSRSCard**

Send the scored review to the learner-model binary. The `review` method records
the observation and advances the persisted FSRS card atomically:

```bash
jq -nc \
  --arg learner_id "$GOAL_ID" \
  --arg concept_id "$CONCEPT_ID" \
  --arg rating "$RATING" \
  --arg timestamp "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --argjson score "$RETENTION_SCORE" \
  '{method:"review",params:{
    learner_id:$learner_id,
    concept_id:$concept_id,
    score:$score,
    rating:$rating,
    timestamp:$timestamp,
    source_skill:"learn-retain"
  }}' | learner-model
```

The learner-model `fsrs.rs` stub's `next_review()` function consumes the
`Rating` and returns an updated `FSRSCard` with new `due`, `stability`,
`reps`, and `lapses` values. These are persisted to the learner model store.

### 4. Report session results

Surface a session summary via `ui-surface` (see Session summary format below).

## FSRS integration

The learner-model crate's `fsrs.rs` contains a simplified FSRS-6 stub. The
`next_review()` function takes a `Rating` enum and returns an updated
`FSRSCard`. `learn-retain` calls this by sending JSON-RPC commands to the
learner-model binary — there is no direct Rust dependency in this skill.

### Rating enum variants

```
Rating::Again  — concept forgotten; reschedule immediately (1 day)
Rating::Hard   — recalled with significant effort
Rating::Good   — recalled with moderate effort
Rating::Easy   — recalled effortlessly
```

## Retention threshold and mastery closure

The retention check that closes the mastery criterion from `feynman-loop`
requires two conditions:

1. At least 24 hours have elapsed since the `feynman-loop` artifact was written.
2. Retention grade ≥ 0.6 on this review.

When a concept passes retention at ≥ 0.6 for the first time after the
`feynman-loop` artifact was created, update the artifact:

```json
{
  "retention_passed": true,
  "retention_grade": <score>,
  "retention_date": "<ISO-8601 now>"
}
```

This signals to `feynman-loop` that the mastery cycle is complete for this
concept.

## Review prompt formats by tier

Both tiers must NOT show the original explanation — free recall only.

**Tier 0** (minimal UI):
```
Re-explain [concept] in your own words:
```

**Tier 1** (structured prompt with time nudge):
```
Concept: [concept label]

Re-explain this concept in your own words. You have 3 minutes.
Do not look up the original explanation.
```

Show the concept label in both tiers. Never show the original explanation
during the recall prompt.

## Session summary format

```
Retention session complete:
  Concepts reviewed: N
  Passed (>=0.6):    M
  Needs re-review:   K (Rating::Again — scheduled in 1 day)
  Next session due:  <date of earliest next due card>
```

Surface this summary via `ui-surface` at the end of every session.

## Dependencies

| Dependency | Role |
|---|---|
| `substrate/learner-model/` | FSRS card store; `load`, `add_observation` commands |
| `skills/learn/learn-grade/` | Grades the free-recall explanation (0.0–1.0) |
| `skills/learn/ui-surface/` | Surfaces prompts and the session summary |
| `skills/learn/feynman-loop/` | Produces the artifacts this skill checks for retention |

## Error handling

- If `learner-model load` returns no due cards, emit:
  `"No cards due for <goal-id>. Next review: <earliest due date>."` and stop.
- If the learner-model binary is unreachable, surface the error via `ui-surface`
  and halt — do not silently skip card updates.
- If `learn-grade` returns an error, treat the grade as 0.0 and apply
  `Rating::Again`. Log the grader error in the session summary.
