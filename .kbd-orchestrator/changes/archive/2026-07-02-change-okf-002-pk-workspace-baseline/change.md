---
id: change-okf-002-pk-workspace-baseline
title: Clone prometheus-knowledge-rs; build/test baseline; diagnose pk ingest LLM failure
phase: phase-okf-llm-wiki-adoption
gaps: [Goal1, Goal5]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: pending
model_class: medium
scope:
  - ~/Projects/prometheus/prometheus-knowledge-rs (new sibling checkout)
---

# change-okf-002 — pk workspace baseline

## Context

No local working checkout of prometheus-knowledge-rs exists (only a read-only
cargo git checkout on /Volumes/my-passport). All format-layer changes
(003–006) require it. Separately, `pk ingest` fails locally with
"LLM error: failed to parse LLM response" — this blocks e2e verification.

## Tasks

- [ ] git clone https://github.com/Prometheus-AGS/prometheus-knowledge-rs.git
      into ~/Projects/prometheus/prometheus-knowledge-rs
- [ ] cargo build --workspace and cargo test --workspace baseline; record results
- [ ] Diagnose pk ingest LLM parse failure (timeboxed ~2h): fix if shallow,
      else document root cause + workaround and file follow-up
- [ ] Verify: pk --version from locally built binary; baseline test results recorded
