---
type: Reference
id: skill-platform-specifications
title: "Skill Platform Specifications"
description: "Cross-platform analysis of skill/agent platforms: agentskills.io, Claude Code, Codex, OpenCode, Cursor, Windsurf, Kimi, MiniMax, and their skill packaging formats."
tags:
- skills
- platforms
- agentskills
- packaging
sources:
- manual-backfill
timestamp: 2026-07-10T19:49:34.769779+00:00
created_at: 2026-07-10T19:49:34.769779+00:00
updated_at: 2026-07-10T19:49:34.769779+00:00
revision: 0
---
# Skill Platform Specifications

## Summary

Cross-platform analysis of skill/agent platforms: agentskills.io, Claude Code, Codex, OpenCode, Cursor, Windsurf, Kimi, MiniMax, and their skill packaging formats.

## Key Findings

- **agentskills.io**: Emerging standard for skill metadata. JSON-based manifest with tool definitions.
- **Claude Code**: Rich tool use API, supports custom MCP servers, strong reasoning capabilities.
- **Codex**: OpenAI's agent framework. JSON schema for tool definitions, conversation threading.
- **OpenCode**: Open-source alternative with plugin architecture. Skills as YAML + Python modules.
- **Cursor**: Editor-integrated agents. Skills defined via `.cursorrules` + custom commands.
- **Windsurf**: Cascade agent with tool integration. Skill manifests in TOML.
- **Kimi**: Chinese LLM with agent mode. Skill packs as JSON configurations.
- **MiniMax**: API-first approach. Skills as structured prompt + tool bindings.
- **Commonality**: All support some form of tool definition + system prompt + execution context.
- **Gap identified**: No cross-platform skill format; each ecosystem is siloed.

## Full Report

[Read the complete research report](/docs/deep-research/research/skill-platform-specifications-report.md)

**Date:** 2026-07-03  
**Researcher:** Prometheus Research Agent  
**Lines:** ~500
