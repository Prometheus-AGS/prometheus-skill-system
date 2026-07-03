---
id: learn-grade
title: /learn-grade
sidebar_label: learn-grade
---

# /learn-grade

Sycophancy-corrected external grader for Feynman explanations.

## What it grades

Given a learner's explanation of a concept, `learn-grade` produces:

- `overall_score` (0.0–1.0)
- `misconceptions_absent` (0.0 or 1.0)
- `gap_list` — specific concepts that were omitted or wrong
- `transfer_score` — how well the explanation generalizes

## Sycophancy correction

Every draft grade routes through sycophancy-correction S-02 before delivery.
A grade claiming "no gaps" when gaps are present is rewritten.

This is enforced architecturally — `learn-grade` cannot deliver a sycophantic
grade even if the underlying LLM produces one.

## Mastery gate

`learn-grade` is one of the three gates for mastery closure:

```
overall_score ≥ 0.7 AND misconceptions_absent == 1.0
```

Both conditions must hold. A high score with a present misconception is NOT mastery.

## Usage

`learn-grade` is invoked automatically inside `feynman-loop` after the learner
provides an explanation. It can also be run standalone:

```
/learn-grade "The borrow checker ensures there is only one mutable reference..."
```
