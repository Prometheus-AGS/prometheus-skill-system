# Prometheus Deep Research Skill — Master Specification

## Overview

This document is the master specification for the **Prometheus Deep Research Skill** — a universal, cross-platform deep-research capability designed for the Prometheus Skill Pack. It synthesizes findings from 8 parallel research investigations into a unified architectural blueprint.

**Version:** 1.0.0-draft  
**Date:** 2026-07-03  
**Status:** Specification (pre-implementation)  

## Quick Links

- **Full Specification:** [`/docs/deep-research/index.md`](/docs/deep-research/index.md) — 1,685 lines, 22 sections, 96 sub-sections
- **Research Reports:**
  - [Deep Research Skill Landscape](/docs/deep-research/research/deep-research-skill-landscape-report.md) — GPT Researcher, LangGraph, MiroThinker, etc.
  - [Skill Platform Specifications](/docs/deep-research/research/skill-platform-specifications-report.md) — agentskills.io, Claude, Codex, OpenCode, Cursor, Kimi, MiniMax
  - [AG-UI / A2UI / MCP App UI Frameworks](/docs/deep-research/research/ag-ui-a2ui-mcp-app-ui-frameworks-research.md) — Agent-User interaction protocols
  - [Knowledge Assets & Architecture](/docs/deep-research/research/knowledge-assets-architecture-report.md) — Knowledge graph, vector DB, embedding strategies
  - [Deep Research + Feynman Integration](/docs/deep-research/research/deep-research-feynman-integration-patterns.md) — Learning loop integration patterns
  - [Google OKF LLM Wiki Standards](/docs/deep-research/research/google-okf-llm-wiki-standards-report.md) — Open Knowledge Format v0.1
  - [Threaded/Concurrent Research](/docs/deep-research/research/threaded-concurrent-research-per-thread-context.md) — Per-thread context isolation
  - [Long-Running Research Process Management](/docs/deep-research/research/long-running-research-process-management-report.md) — Checkpointing, drift detection

## Architecture at a Glance

- **Dual-Nature Delivery:** Portable `SKILL.md` skill + native Rust MCP server (Axum, stdio + SSE)
- **10-Stage Pipeline:** Planner → Search → Retrieve → Collect → Verify → Resolve → Graph → Cite → Report → Export
- **Output Format:** `.research` package (knowledge graph, citations, embeddings, contradictions, confidence scores, audit trails)
- **UI Paradigm:** AG-UI streaming + A2UI generative artifacts + MCP App embedded UIs
- **CLI Tool:** `prometheus-research` with 11 subcommands
- **Storage:** surreal-memory-server (localhost:23001) for unified graph + vector + document + relational + time-travel

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Rust for MCP server | Performance, memory safety, tokio async ecosystem |
| Dual delivery (skill + server) | Portable fallback + native performance |
| `.research` package format | OKF v0.1 base + Prometheus extensions (research_id, confidence, verification_status) |
| surreal-memory backend | Already in stack, supports all required storage modes |
| Feynman loop integration | Deep research as learning primitive; research outputs auto-generate curriculum DAGs |
| Threaded concurrency | Per-thread context isolation via surreal-memory; deterministic merge stage |
| LangGraph-style checkpointing | Long-running process recovery; progressive summarization (5 layers) |

## Integration Points

- **Feynman Skills:** `feynman-loop`, `learn-plan`, `learn-survey`, `learn-kb`, `learn-grade`, `learn-practice`, `learn-retain`, `learn-certify`
- **MCP Stack:** `tavily-mcp` (search), `sequential-thinking` (reasoning), `liter-llm` (literature), `surreal-memory` (storage), `forge-rs` (build), `prometheus-knowledge` (KB), `sycophancy-correction` (bias)
- **Karpathy Wiki Hooks:** `UserPromptSubmit` → `pk focus`, `Stop` → `forge reflect` + `pk ingest`
- **Platforms:** Claude Code, Codex, OpenCode, Cursor, Windsurf, Kimi, MiniMax, Roo, Amp, Gemini

## Next Steps (Post-Specification)

1. Scaffold `tools/prometheus-research/` Rust workspace using `native-agent` skill template
2. Implement MCP server core (Axum, stdio + SSE transports)
3. Build 10-stage pipeline as async tokio tasks with surreal-memory checkpointing
4. Create `.research` package serializer/deserializer with OKF v0.1 compliance
5. Develop `prometheus-research` CLI with all 11 subcommands
6. Integrate with AG-UI / A2UI for streaming + generative artifact UIs
7. Wire Feynman loop hooks at Report → Export stage boundary

## References

- 67 cross-references throughout the full specification
- 8 research reports totaling ~6,285 lines of raw investigation
- Google OKF v0.1 (released 2026-06-12, Apache 2.0)
- Karpathy LLM Wiki pattern (content-addressed, append-only, index+log)
