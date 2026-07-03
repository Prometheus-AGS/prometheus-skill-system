---
id: anti-sycophancy
title: Anti-Sycophancy
sidebar_label: Anti-Sycophancy
---

# Anti-Sycophancy in Learning

Sycophantic feedback — telling a learner they understood something when they didn't —
produces measurably worse learning outcomes. Prometheus blocks this architecturally.

## The problem

LLMs have a natural tendency to:

- Praise mediocre explanations ("Great job! You almost got it!")
- Understate gaps ("You just need to review the details")
- Affirm self-reported confidence ("If you feel confident, you probably are")
- Avoid saying "No" or "Wrong" directly

In a learning context, this sycophancy is actively harmful — it closes loops that
should stay open, and creates false confidence in unmastered material.

## The architectural solution

`learn-grade` is on the critical path of the sycophancy correction gate:

```
learner explanation
  → learn-grade (draft score)
    → sycophancy-correction S-02 check
      → rewrite if sycophantic
    → final score delivered
```

A grade that says "no gaps" when gaps are present is **rewritten** before delivery.
This is not optional guidance — it is enforced in the `learn-grade` skill script.

## Sycophancy patterns blocked

| Pattern | Example |
|---------|---------|
| False positive | "Score: 0.9 — only minor gaps!" when multiple misconceptions present |
| Gap minimization | "The explanation was clear" when key concept was omitted |
| Self-report affirmation | "Since you feel confident, you've mastered this" |
| Encouragement inflation | "Excellent explanation! Just a few edge cases missing..." |

## Behavioral rules

- **"Never tell the learner they did well when they did not"** — this is a hard operator invariant
- **Self-reported fluency NEVER closes a Feynman loop** — the 3-condition gate is the only path
- `learn-grade` MUST include concrete gaps if any exist

## For the Reflector gate

The same sycophancy-correction system that guards `learn-grade` also guards
agent reflections via the `reflector` SubagentStop hook. See [Sycophancy
Correction](/docs/guide/sycophancy-correction) for full details.
