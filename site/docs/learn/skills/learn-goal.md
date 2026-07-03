---
id: learn-goal
title: /learn-goal
sidebar_label: learn-goal
---

# /learn-goal

Start a learning session. The skill elicits what you want to learn, validates
feasibility, and routes into the full learning arc.

## Trigger phrases

- "I want to learn X"
- "teach me X"
- "help me understand X"
- `/learn-goal "I want to master Y"`

## Example

```
/learn-goal "I want to understand how Rust's borrow checker works"
```

## What happens

1. Elicits the learning goal (what, why, depth)
2. Runs `learn-survey` to diagnose current knowledge level
3. Builds a concept DAG via `learn-plan`
4. Starts `feynman-loop` on the first concept
5. Tracks progress in the learner model (FSRS-6)

## Options

```
/learn-goal "topic" --kb local:/path/to/docs    # ground in custom KB
/learn-goal "topic" --depth deep                # full mastery track
/learn-goal "topic" --depth overview            # survey-level understanding
```
