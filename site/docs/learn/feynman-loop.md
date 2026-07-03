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

## Trigger phrases

- "teach me X"
- "explain this concept to me"
- "I want to really learn X"
- "feynman loop on X"
