---
name: learn-practice
description: Deliberate practice skill for the Feynman learning loop. Generates difficulty-gated, interleaved problem sets across derivation, implementation, and transfer modes. Grades via learn-grade, updates the learner model, and prevents illusion-of-knowing by requiring demonstrated performance rather than felt fluency.
version: '1.1.0'
license: MIT
metadata:
  author: prometheus-skill-pack
  category: learn
  tags: [learn, practice, deliberate-practice, interleaving, transfer, mastery]
---

# learn-practice

Deliberate practice skill for the Feynman learning loop. Generates difficulty-gated,
interleaved problem sets and grades responses via `learn-grade`. Mastery advances
through demonstrated performance, not felt fluency.

## When to invoke

Invoked after `feynman-loop` closes a concept loop, OR directly by the user for
practice on a specific concept:

```
/learn-practice --concept-id <id> --goal-id <id> [--type derivation|implementation|transfer] [--problems N]
```

Without `--type`, the skill rotates through all three types (interleaving).
Without `--problems`, defaults to 3 problems.

## Problem types

### Derivation

Work through a logical chain from first principles.

Example: "Starting from the definition of a derivative, derive the power rule step-by-step."

- Problems generated from corpus definitions and theorems
- Graded on logical completeness and correctness of each step

### Implementation

Apply the concept in a concrete, worked context.

Example: "Implement a binary search algorithm for a sorted list of integers."

- Problems generated from corpus examples and reference implementations
- Graded on correctness, edge case handling, and clarity

### Transfer

Apply the concept in a novel context the corpus did not explicitly cover.

Example: "The concept of gradient descent was explained for neural networks. Apply the
same optimization principle to a different problem: scheduling tasks to minimize total
latency."

- Transfer problems always come from `learn-grade`'s novel problem generator — NOT
  from the learner's own explanation
- Graded on analogical reasoning and correct application

## Difficulty gating

| Mastery range | Available types |
|---|---|
| < 0.6 | `derivation` only — build from definitions |
| 0.6 – 0.8 | `derivation` and `implementation` interleaved |
| > 0.8 | All three types interleaved, including `transfer` |

If the user requests a higher difficulty than their mastery level allows, show a
brief warning and offer to proceed anyway or suggest building mastery first:

```
Warning: your current mastery for "<concept>" is 0.55, below the 0.6 threshold
for implementation problems. Proceed anyway? [yes / build mastery first]
```

## Interleaving schedule

When rotating through problem types, use a mixed schedule — NOT blocked (not all
derivations then all implementations).

| Session length | Schedule |
|---|---|
| 3 problems | derivation → transfer → implementation |
| 6 problems | derivation → implementation → transfer → derivation → transfer → implementation |
| N problems | Continue the rotation pattern, wrapping as needed |

Blocked practice (all of one type then all of another) is explicitly prohibited
because it produces illusion-of-knowing.

## Flow

1. **Load concept state** — call `learner-model` `load`, find the concept by
   `concept_id`. Extract current mastery score and corpus entries. Then open
   the session with `add_session` (`session_type: "practice"`, see "Learner
   model update"); keep the returned `session_id` for the closing call.

2. **Select problem type** — from `--type` flag or the interleaving schedule
   for the current problem index.

3. **Check mastery gate** — if the selected type exceeds the mastery threshold,
   display the warning and wait for user confirmation before continuing.

4. **Generate problem** — from the corpus (NOT from the learner's explanation).
   Use the concept's corpus entries as source material. For transfer problems,
   delegate problem generation to `learn-grade`.

5. **Present problem** — via `ui-surface`. Show:
   - Problem number (e.g., "Problem 1 of 3")
   - Problem type label
   - The problem statement
   - A prompt for the learner's response

6. **Collect response** — wait for the learner's full answer before proceeding.

7. **Grade via learn-grade** — with the appropriate grading mode:
   - `derivation`: check logical chain completeness and step-by-step correctness
   - `implementation`: check correctness, edge cases, and clarity
   - `transfer`: check analogical reasoning and correct application

8. **Show grade feedback** — display the score and `learn-grade` feedback before
   moving to the next problem. Do not skip to the next problem silently.

9. **Update learner model** — call `add_observation` with the score and
   `source_skill: "learn-practice"` after each problem.

10. **Repeat** for remaining problems in the session.

11. **Session summary** — via `ui-surface` after all problems are complete.
    Close the session with a second `add_session` carrying the same
    `session_id` and an `ended_at`; the learner model replaces the open record
    rather than adding a duplicate.

## Session summary format

```
Practice session complete:
  Concept: <label>
  Problems: N attempted, M passed (score >= 0.7)
  Types: derivation (X), implementation (Y), transfer (Z)
  Mastery estimate: <before> -> <after>
```

"Passed" means a score of 0.7 or higher from `learn-grade`.

## Illusion-of-knowing prevention

The skill explicitly avoids:

- Asking "does this make sense?" — yes/no is not evidence of understanding
- Accepting "I think I get it" as a passing response
- Generating problems from the learner's explanation text — only the corpus is used
- Blocking practice as complete when the learner says they understand without
  demonstrating it

If the learner provides a non-answer ("I understand this concept"), respond:

```
Understanding is shown through doing. Here is the problem — please work through it.
```

## Grading thresholds

| Score | Outcome |
|---|---|
| >= 0.7 | Pass — counts toward mastery advancement |
| 0.4 – 0.69 | Partial — feedback provided, does not count as pass |
| < 0.4 | Fail — detailed corrective feedback shown before next problem |

## Learner model update

After each problem, call `add_observation` on the concept node through the
learner-model JSON-RPC binary:

`$LEARNER_ID` is the learner's DID, the same key learn-grade and learn-certify
use. It is not the goal id: one learner model spans every goal, and a session
stored under a different key is invisible to learn-certify's gates.

```bash
jq -nc \
  --arg learner_id "$LEARNER_ID" \
  --arg concept_id "$CONCEPT_ID" \
  --argjson score "$SCORE" \
  '{method:"add_observation",params:{
    learner_id:$learner_id,
    concept_id:$concept_id,
    score:$score,
    source_skill:"learn-practice"
  }}' | learner-model
```

The learner model aggregates these observations to update the mastery estimate.
The updated mastery is reflected in the session summary.

The session itself is recorded with `add_session`, once when it opens and once
when it closes. learn-certify's practice-breadth gate counts these records
(`session_type: "practice"`, `concepts_touched` containing the concept), so a
practice session that never calls `add_session` does not count toward
certification:

```bash
# open — session_id is generated and returned; an error reply yields no id
REPLY=$(jq -nc \
  --arg learner_id "$LEARNER_ID" \
  --arg concept_id "$CONCEPT_ID" \
  '{method:"add_session",params:{
    learner_id:$learner_id,
    session_type:"practice",
    skills_called:["learn-practice","learn-grade"],
    concepts_touched:[$concept_id]
  }}' | learner-model 2>/dev/null) || REPLY='{"error":"learner-model unavailable"}'
SESSION_ID=$(printf '%s' "$REPLY" | jq -r 'select(.error == null) | .session_id // empty')
[ -n "$SESSION_ID" ] || echo "warning: session not recorded: $REPLY" >&2

# close — same id, ended_at set; replaces the open record and keeps its started_at
[ -n "$SESSION_ID" ] && jq -nc \
  --arg learner_id "$LEARNER_ID" \
  --arg concept_id "$CONCEPT_ID" \
  --arg session_id "$SESSION_ID" \
  --arg ended_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  '{method:"add_session",params:{
    learner_id:$learner_id,
    session_id:$session_id,
    session_type:"practice",
    skills_called:["learn-practice","learn-grade"],
    concepts_touched:[$concept_id],
    ended_at:$ended_at
  }}' | learner-model
```

When the open call fails (binary absent, `error` reply), `SESSION_ID` is empty:
the closing call is skipped, the warning is shown in the session summary, and
the session does not count toward learn-certify's practice-breadth gate. Never
pass a placeholder id; a made-up id would create a phantom session. A closing
call that omits `started_at` keeps the opening call's timestamp; pass one
explicitly only to override it.

## Handoff

After a practice session, present the next options:

```
Next options:
  /learn-practice --concept-id <id>   (more practice on this concept)
  /feynman-loop --concept-id <next>   (move to next concept)
  /learn-retain <goal-id>             (review due retention cards)
```

If the session mastery rose above 0.8 for the first time, surface:

```
Mastery reached 0.8+ — transfer problems are now unlocked for this concept.
```
