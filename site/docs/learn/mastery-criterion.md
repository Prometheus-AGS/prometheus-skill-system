---
id: mastery-criterion
title: Mastery Criterion
sidebar_label: Mastery Criterion
---

# Mastery Criterion

The learning procedure requires three evidence conditions before recording mastery.
Scores depend on the corpus, grader and verification method; they are not an
independent measurement merely because a number was recorded.

## The 3 conditions

### Condition 1 — Grade gate

`learn-grade` produces:

- `overall_score` (0.0–1.0)
- `misconceptions_absent` (0.0 or 1.0)

Both must satisfy:

```
overall_score ≥ 0.7 AND misconceptions_absent == 1.0
```

A score of 0.8 with an unresolved misconception is **not mastery**.

### Condition 2 — Transfer gate

Two novel transfer problems (problems the learner has never seen before, in a
different context than the original learning) must be solved at ≥ 0.7 each.

### Condition 3 — Retention gate

A spaced retrieval check via `learn-retain` must pass at ≥ 24 hours after the
initial mastery claim.

## Why all three?

- **Grade gate** — measures explanation quality and misconception absence
- **Transfer gate** — measures ability to apply knowledge in new contexts (true understanding)
- **Retention gate** — measures durability (not just cramming)

## PFA mastery update

The learner model uses the **PFA (Performance Factors Analysis)** update rule:

```
mastery_new = mastery_old + 0.3 × (score - mastery_old)
```

Applied at ≥ 5 observations per concept.

## Evidence limits

The skill procedure requires sycophancy screening with the available checker or
its manual fallback. Persistence alone does not enforce that instruction or
establish a distinct grader. Retain the corpus, grader identity, timestamps and
transfer verification. See [Anti-Sycophancy](/docs/learn/anti-sycophancy).
