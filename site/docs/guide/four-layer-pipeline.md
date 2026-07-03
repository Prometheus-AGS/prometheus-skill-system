---
id: four-layer-pipeline
title: Four-Layer Pipeline
sidebar_label: Four-Layer Pipeline
---

# Four-Layer Pipeline

See the full chapter:
[docs/guide/04-four-layer-pipeline.md](https://github.com/prometheusags/prometheus-skill-pack/blob/main/docs/guide/04-four-layer-pipeline.md)

## Layers

| Layer | Location | Purpose |
|-------|----------|---------|
| A — Substrate | `substrate/` | Rust crates: persistence, CRDT, P2P, UI |
| B — UI primitive | `skills/learn/ui-surface` | Cross-harness rendering |
| C — Skills | `skills/` | 50+ agent skills |
| D — KB adapters | `shared/scripts/` | Privacy-safe KB integration |

The four layers allow Prometheus skills to run identically across all supported harnesses
while adapting their UI rendering to each harness's capability level.
