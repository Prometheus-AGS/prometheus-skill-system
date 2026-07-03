---
id: memory-and-learning
title: Memory & Learning
sidebar_label: Memory & Learning
---

# Memory & Learning

See the full chapter:
[docs/guide/06-memory-and-learning.md](https://github.com/prometheusags/prometheus-skill-pack/blob/main/docs/guide/06-memory-and-learning.md)

## Memory systems

Prometheus provides three memory tiers:

1. **File-based memory** (`~/.claude/projects/.../memory/`) — session-persistent, always available
2. **Cortex MCP** — structured project memory with search
3. **surreal-memory MCP** — knowledge graph + palace RAG + task streams

## Learning engine

The Learn domain implements the **Feynman-Spine** methodology:

```
learn-goal → learn-survey → learn-plan → feynman-loop → learn-grade → learn-retain
```

Mastery is measured objectively — self-reported fluency never closes a Feynman loop.
The loop requires:

1. `learn-grade` score ≥ 0.7 AND `misconceptions_absent == 1.0`
2. Two novel transfer problems solved at ≥ 0.7
3. Retention check via `learn-retain` at ≥ 24h interval
