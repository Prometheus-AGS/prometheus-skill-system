---
id: learn-survey
title: /learn-survey
sidebar_label: learn-survey
---

# /learn-survey

Diagnostic placement for the learning arc. Determines what the learner already
knows and sets the recursion floor for the concept DAG.

## What it does

1. Asks diagnostic questions at the conceptual level
2. Maps answers to the concept graph
3. Identifies prerequisite gaps
4. Sets the starting point for `learn-plan`

## Recursion floor

The recursion floor is the lowest prerequisite the learner should be assumed to
know. For example, if the learner wants to learn Rust async and already knows
threads, the floor is "threads" — the survey won't go back to "what is concurrency."

## Invocation

`learn-survey` is typically invoked automatically by `learn-goal`. It can also
be run standalone:

```
/learn-survey "Rust async and await"
```
