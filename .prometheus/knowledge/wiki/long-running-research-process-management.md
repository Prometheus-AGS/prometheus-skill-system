---
type: Reference
id: long-running-research-process-management
title: "Long-Running Research Process Management"
description: "Research on managing long-running research processes: checkpointing, drift detection, progressive summarization, and the Karpathy Loop for iterative refinement."
tags:
- research
- process-management
- background-job
- daemon
sources:
- manual-backfill
timestamp: 2026-07-10T19:49:34.768544+00:00
created_at: 2026-07-10T19:49:34.768544+00:00
updated_at: 2026-07-10T19:49:34.768544+00:00
revision: 0
---
# Long-Running Research Process Management

## Summary

Research on managing long-running research processes: checkpointing, drift detection, progressive summarization, and the Karpathy Loop for iterative refinement.

## Key Findings

- **LangGraph-Style Checkpointing**: Store state after each pipeline stage. Resume from any checkpoint.
- **KBD Integration**: `current-waypoint.json` extended for research tracking (current stage, thread status, checkpoint hash).
- **Progressive Summarization**: 5 layers — raw, chunked, summarized, synthesized, final. Each layer is a checkpoint.
- **Karpathy Loop**: 
  - Micro-frequency: Per-iteration reflection during pipeline execution
  - Macro-frequency: Per-session reflection at completion (what worked, what didn't, what to change)
- **Drift Detection**: Compare current findings against original research plan. Alert when scope creep or topic divergence occurs.
- **Human-in-the-Loop**: Checkpoints at natural decision points (after Search, after Verify, before Export).
- **Time-to-Result**: Research jobs may run hours. User gets streaming updates via AG-UI, not just final report.
- **Resource Management**: Automatic pause/resume based on API rate limits and compute budget.

## Full Report

[Read the complete research report](/docs/deep-research/research/long-running-research-process-management-report.md)

**Date:** 2026-07-03  
**Researcher:** Prometheus Research Agent  
**Lines:** ~680
