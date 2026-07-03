---
id: learn-plan
title: /learn-plan
sidebar_label: learn-plan
---

# /learn-plan

Builds a concept DAG (directed acyclic graph) and ordered curriculum from the
learning goal and survey results.

## Output

- Concept list with prerequisites
- Ordered curriculum (dependencies resolved)
- Estimated time per concept
- Target mastery criteria per concept

## Example

For "Rust async":

```
1. Futures trait             (prereq: none)
2. Poll + Waker              (prereq: Futures)
3. async/await syntax        (prereq: Poll)
4. Tokio runtime             (prereq: async/await)
5. Spawning + join handles   (prereq: Tokio)
6. Channels + select!        (prereq: spawn)
```

## Usage

```
/learn-plan "Rust async and await"
```

Or via the learning arc — `learn-plan` is called automatically after `learn-survey`.
