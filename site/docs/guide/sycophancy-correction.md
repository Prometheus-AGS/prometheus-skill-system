---
id: sycophancy-correction
title: Sycophancy Correction
sidebar_label: Sycophancy Correction
---

# Sycophancy Correction

See the full chapter:
[docs/guide/07-sycophancy-correction.md](https://github.com/prometheusags/prometheus-skill-pack/blob/main/docs/guide/07-sycophancy-correction.md)

## What it does

The sycophancy-correction MCP server detects and corrects sycophantic patterns in LLM
completions, agent reflections, and learning grades.

**Pedagogical sycophancy** — telling a learner they understood something when they didn't —
produces worse learning outcomes. This is blocked architecturally by routing `learn-grade`
through the sycophancy correction gate.

## The Reflector gate

When the `reflector` SubagentStop hook fires, `sycophancy-check-reflection.sh` validates
the reflection against the PMPO Reflect structure:

- **Delta** — what was planned vs. what was delivered
- **Root Cause** — why any delta occurred
- **Corrective Actions** — concrete steps for the next iteration

A reflection that summarizes success without naming gaps is rejected with actionable
feedback. After 2 consecutive rejections, the third attempt is accepted with a warning.
