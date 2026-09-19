---
id: feynman-loop
title: Feynman Loop
sidebar_label: Feynman Loop
---

# Feynman Loop

The `/feynman-loop` skill is the core of the Learn domain. It implements a structured
Explain → Gap-Find → Re-Study cycle for any concept.

## The 3-step loop

### 1. Explain

The learner explains the concept in their own words (to "a 12-year-old" or to a
non-expert audience). No notes, no lookup.

### 2. Gap-Find

`learn-grade` examines the explanation and identifies:

- Concepts that were omitted
- Misconceptions that were present
- Transfer problems not correctly solved

The grade routes through sycophancy-correction to prevent false "you got it!" feedback.

### 3. Re-Study

For each gap identified, `learn-plan` creates a targeted micro-study session.
The cycle then repeats from step 1.

## Mastery closure

A Feynman loop is closed when **all three** conditions hold:

1. `learn-grade` score ≥ 0.7 AND `misconceptions_absent == 1.0`
2. Two novel transfer problems solved at ≥ 0.7
3. Retention check via `learn-retain` at ≥ 24h interval after initial mastery

Self-reported fluency ("I feel like I understand this") never satisfies any condition.

## The artifact a closed loop writes

Closing a loop writes one artifact, and every learn skill reads it from the same
place:

```
<learn-home>/goals/<goal-id>/artifacts/<concept-id>/<artifact-id>.json
```

`<learn-home>` is `${PROMETHEUS_LEARN_HOME:-~/.prometheus/learn}`.
`learn-retain` globs `artifacts/<concept-id>/*.json` for the most recent
artifact (latest `closed_at`); `learn-certify` reads the same concept directory
for its evidence entries.

`write-artifact.sh` refuses an artifact that cannot say how it was checked. Two
blocks are required:

- **`verification`** — exactly one entry per transfer score, each with a label
  from the four-value vocabulary shared with deep-research
  (`verified | unverified | blocked | inferred`) and non-empty evidence.
- **`provenance`** — the grade file and corpus path the scores came from.

A self-reported score is `unverified` and never satisfies mastery criterion 2.

## What the learner model records

The `learner-model` binary carries the write paths the loop needs, so nothing a
later skill reads is a value no earlier skill wrote:

| Method | Written by | Read by |
|---|---|---|
| `add_observation` | `learn-grade`, `learn-practice` | mastery estimate |
| `add_gap` / `resolve_gap` | `learn-grade` | the next iteration's targets |
| `add_session` | `learn-practice` (open and close) | `learn-certify` practice-breadth gate |
| `set_certified` | `learn-certify` | `learn-certify` concept gate |

Mastery holds at the seeded prior until the fifth observation, then follows
`mastery_new = mastery_old + 0.3 × (score − mastery_old)`. Spaced-repetition
scheduling runs through FSRS, so `difficulty` is read and updated on every
review.

## Trigger phrases

- "teach me X"
- "explain this concept to me"
- "I want to really learn X"
- "feynman loop on X"
