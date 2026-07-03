---
id: mastery-criterion
title: Mastery Criterion
sidebar_label: Mastery Criterion
---

# Mastery Criterion

Mastery is an objective measurement, not a feeling. Prometheus uses a 3-condition
gate that must all be satisfied before a concept is considered mastered.

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

## Sycophancy protection

`learn-grade` is on the critical path of the anti-sycophancy gate:

1. Draft grade is produced
2. Grade routes through sycophancy-correction S-02 check
3. A grade claiming "no gaps" when gaps exist is **rewritten before delivery**

This is enforced architecturally, not as optional guidance.
