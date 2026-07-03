---
id: learn-retain
title: /learn-retain
sidebar_label: learn-retain
---

# /learn-retain

Spaced retrieval using the FSRS-6 scheduler. Surfaces concepts at optimal
intervals for long-term retention.

## How it works

1. The `learner-model` substrate crate tracks FSRS-6 cards per concept
2. `learn-retain` asks which concepts are due for review
3. Presents a retrieval prompt (explain it again, solve a transfer problem)
4. Updates FSRS-6 state based on performance

## Schedule

FSRS-6 calculates the optimal interval between reviews based on:

- Initial memory strength
- Forgetting curve parameters
- Historical performance

## Mastery closure gate

`learn-retain` provides the third mastery condition:

> Retention check at ≥ 24h interval after initial mastery claim

The 24h gap ensures the learner isn't passing on short-term working memory.

## Usage

```
/learn-retain                     # review all due concepts
/learn-retain "Rust borrow checker"   # targeted review
```
