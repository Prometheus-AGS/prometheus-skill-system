---
id: loop-architecture
title: Loop Architecture
sidebar_label: Loop Architecture
---

# Loop Architecture

See the full chapter:
[docs/guide/03-loop-architecture.md](https://github.com/prometheusags/prometheus-skill-pack/blob/main/docs/guide/03-loop-architecture.md)

## Overview

Prometheus uses nested control loops:

```
L3: Strategic / Evolution loop (pmpo-evolver)
  └─ L2: KBD lifecycle loop (kbd-assess → kbd-reflect)
       └─ L1: Change execution loop (per-change in kbd-execute)
```

**L3** drives long-term strategic evolution across KBD cycles.

**L2** is the KBD lifecycle — produces assessment, plan, execution, reflection artifacts.

**L1** is the per-change loop inside `kbd-execute` — each change is a discrete unit of work
with its own QA gate (artifact-refiner).
