---
type: Reference
id: deep-research-skill-landscape
title: "Deep Research Skill Landscape"
description: "Comprehensive survey of existing deep research agents and skills, including GPT Researcher, LangGraph ODR, MiroThinker, and other implementations in the ecosystem."
tags:
- deep-research
- skill-landscape
- research
sources:
- manual-backfill
timestamp: 2026-07-10T19:49:34.764950+00:00
created_at: 2026-07-10T19:49:34.764950+00:00
updated_at: 2026-07-10T19:49:34.764950+00:00
revision: 0
---
# Deep Research Skill Landscape

## Summary

Comprehensive survey of existing deep research agents and skills, including GPT Researcher, LangGraph ODR, MiroThinker, and other implementations in the ecosystem.

## Key Findings

- **GPT Researcher**: Python-based, multi-agent architecture with Tavily/Google search backends. Strong for rapid research but limited context windows.
- **LangGraph ODR**: LangChain's official deep research implementation. Graph-based state machine with human-in-the-loop checkpoints.
- **MiroThinker**: Reasoning-first approach with chain-of-thought verification. Good for complex analytical queries.
- **Common patterns**: All use web search → synthesize → verify loops. Most lack persistent knowledge storage between sessions.
- **Gap identified**: No universal cross-platform skill exists; each is tied to a specific framework or runtime.

## Full Report

[Read the complete research report](/docs/deep-research/research/deep-research-skill-landscape-report.md)

**Date:** 2026-07-03  
**Researcher:** Prometheus Research Agent  
**Lines:** ~450
