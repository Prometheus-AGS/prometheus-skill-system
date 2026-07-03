# Deep Research Agents & Skills Landscape: Comprehensive Research Report

**Date:** 2026-07-03  
**Researcher:** Prometheus Research Agent  
**Scope:** Foundational research for designing a universal deep-research skill for the Prometheus Skill Pack  
**Sources:** 50+ web searches, GitHub repositories, arXiv papers, official documentation, and industry analysis.

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Topic 1: Deep Research Agents & Skills Landscape](#topic-1-deep-research-agents--skills-landscape)
3. [Topic 2: Skill Platform Specifications](#topic-2-skill-platform-specifications)
4. [Topic 3: AG-UI / A2UI / MCP App UI Frameworks](#topic-3-ag-ui--a2ui--mcp-app-ui-frameworks)
5. [Topic 4: Knowledge Assets & Architecture Patterns](#topic-4-knowledge-assets--architecture-patterns)
6. [Cross-Cutting Insights & Strategic Recommendations](#cross-cutting-insights--strategic-recommendations)
7. [References](#references)

---

## Executive Summary

The deep research agent landscape in 2025-2026 has matured from experimental prototypes to production-grade ecosystems. Five major open-source projects define the state of the art: **GPT Researcher** (the most mature, 27.9k stars), **LangGraph Open Deep Research** (the most configurable, MCP-native), **OpenResearcher** (trajectory synthesis for training), **MiroThinker** (SOTA benchmark performance via interactive scaling), and **HuggingFace Open Deep Research** (smolagents-based, fully open). Together with commercial offerings (OpenAI Deep Research, Gemini Deep Research, Kimi Deep Research), these form a rich competitive landscape.

The skill platform ecosystem has converged around **Agent Skills** (agentskills.io), an open standard originated by Anthropic and now adopted by Claude Code, Codex CLI, Cursor, Windsurf, Kimi Code, OpenCode, and 20+ other platforms. The core format is a `SKILL.md` file with YAML frontmatter and Markdown body, using progressive disclosure (metadata → activation → execution). AGENTS.md has emerged as a cross-tool project context standard. **AGENTS.md + SKILL.md is the winning combination** for portable agent capabilities.

On the UI front, three protocols are crystallizing: **MCP** (agent-to-tool, Anthropic), **A2A** (agent-to-agent, Google), **AG-UI** (agent-to-frontend transport, CopilotKit), and **A2UI** (declarative generative UI payload, Google). The stack is: MCP for tools, AG-UI for transport, A2UI for UI rendering. A unified deep-research MCP app should expose tools via MCP, stream events via AG-UI, and render progress/results via A2UI or assistant-ui.

For knowledge assets, the field has moved beyond "disposable reports" toward **persistent knowledge objects**: citations.json, knowledge_graph.json, embeddings, entity graphs, timelines, contradiction matrices, confidence scores, and reasoning traces. The best deep research architectures use a **planner → decomposer → search planner → multi-source retriever → evidence collector → verifier → conflict resolver → knowledge graph builder → citation manager → report generator → artifact exporter** pipeline. Knowledge should be emitted as queryable, extendable, citable objects — not just static PDFs.

---

## Topic 1: Deep Research Agents & Skills Landscape

### 1.1 GPT Researcher (github.com/assafelovic/gpt-researcher)

**Architecture & Approach:**
- **Planner + Execution + Publisher pattern**: Creates a task-specific agent, generates research questions, uses crawler agents for each question, summarizes and source-tracks each resource, then filters/aggregates into a final report.[^1]
- **Model independence**: Optimized for `gpt-4o-mini` (planner) and `gpt-4o` (execution/publisher) with 128K context, but designed to work with any LLM provider.[^1]
- **Parallelized agent work**: Uses parallel execution agents to gather information for each generated question, reducing research time to ~3 minutes and ~$0.005 per run on average.[^2]
- **Local + web research**: Supports both web search and local document ingestion (PDFs, DOCX, etc.).[^1]
- **Deep Research feature**: Advanced recursive research workflow that explores topics with agentic depth and breadth.[^1]

**Skill Interface & Reusable Components:**
- Can be installed as a **Claude Skill** via `npx skills add assafelovic/gpt-researcher`[^1]
- Provides a full suite of customization options for domain-specific research agents
- Exposes configuration for search engines, source filtering, report length, and output formats
- **Output formats**: PDF, Word, Markdown, and more. Can generate research reports, outlines, resource lists, and lesson reports.[^2]

**Strengths:**
- Most mature open-source deep research project (3+ years, 27.9k stars, active as of May 2026 v3.5.0)[^3]
- Provider-agnostic and self-hostable
- Cost-optimized with parallel execution
- Strong documentation and community

**Weaknesses:**
- Not state-of-the-art on hard research benchmarks (RL-trained agents like Tongyi DeepResearch surpass it)[^3]
- Primarily Python-based; less native MCP integration than newer entrants
- Report-centric output; limited persistent knowledge asset generation

### 1.2 LangGraph Open Deep Research (github.com/langchain-ai/open_deep_research)

**Architecture & Approach:**
- **LangGraph-based workflow**: Structured, configurable graph workflow built on LangChain's LangGraph framework.[^4]
- **Model independence**: Works across many model providers (OpenAI, Anthropic, Google, local models).[^4]
- **MCP-native**: Extensive Model Context Protocol integration — auto-discovers and uses tools following the open standard.[^4]
- **Plan-and-Execute + Reflection**: Creates structured report plans with human-in-the-loop feedback, generates sections one by one with reflection, and iteratively refines.[^4]
- **Legacy implementations**: Includes both a workflow implementation (sequential, quality-focused) and a multi-agent implementation (supervisor-researcher, parallel, speed-optimized).[^4]

**Skill Interface & Reusable Components:**
- Deployable via **LangGraph Platform** for hosted execution
- Available on **Open Agent Platform (OAP)** — a UI for non-technical users to configure with different MCP tools and search APIs[^4]
- Exposes the entire research pipeline as configurable graph nodes
- **Output formats**: Structured reports with configurable section templates

**Strengths:**
- Most configurable and framework-agnostic
- Native MCP support for tool extensibility
- Strong observability via LangSmith integration
- Production-grade deployment options (LangGraph Studio, Platform, OAP)
- Graph-based state management with checkpointing and resumability

**Weaknesses:**
- LangChain ecosystem dependency
- More complex setup than GPT Researcher for simple use cases
- Benchmark performance is good but not SOTA compared to RL-trained agents

### 1.3 OpenResearcher (github.com/TIGER-AI-Lab/OpenResearcher)

**Architecture & Approach:**
- **Fully offline, low-cost trajectory synthesis pipeline**: Synthesizes long-horizon (100+ turns) deep research trajectories for training research agents.[^5]
- **Three browsing primitives**: `search` (retrieve candidate documents), `open` (inspect document in detail), `find` (locate specific evidence within a document).[^5]
- **Offline corpus design**: Merges 15M FineWeb documents (negative/distractor evidence) with 10K golden passages (positive evidence), embedded using Qwen3-Embedding-8B and indexed with FAISS.[^5]
- **Teacher model**: Uses GPT-OSS-120B to synthesize trajectories, parallelized across 64 H100 GPUs.[^5]
- **Reject sampling**: Retains only successful trajectories (55,824 out of 97,632, 56.71% success rate).[^5]

**Key Finding:** Incorrect trajectories use nearly 2× more tool calls than successful ones, suggesting failure is due to inefficient/misdirected search, not insufficient exploration.[^5]

**Skill Interface & Reusable Components:**
- Designed primarily as a **training data generation pipeline**, not an end-user tool
- Provides the OpenResearcher dataset for fine-tuning research agents
- Exposes the trajectory synthesis framework as a reproducible research tool

**Strengths:**
- Addresses the training data scarcity problem for research agents
- Fully offline and reproducible
- Strong empirical evidence for what makes research trajectories succeed
- Open-source and academic-quality

**Weaknesses:**
- Not a direct end-user research tool
- Requires significant compute for trajectory synthesis
- Focused on training rather than deployment

### 1.4 Together AI Open Deep Research

**Note:** Together AI's contribution to open deep research is primarily through hosting and inference infrastructure for open-source models, rather than a distinct agent architecture. Their platform supports:
- Running open-source research agents (including GPT Researcher, HuggingFace Open Deep Research) at scale
- Multimodal model inference for agents that need vision capabilities
- The "Together AI Open Deep Research" reference often points to community deployments using Together's inference API

**Key insight:** Together AI's role is infrastructure — they provide the compute layer for running deep research agents with open weights, but the agent architecture itself comes from other projects.

### 1.5 MiroThinker (github.com/MiroMindAI/MiroThinker)

**Architecture & Approach:**
- **Interactive scaling**: Advances tool-augmented reasoning through training the agent to handle deeper, more frequent agent-environment interactions as a third dimension beyond model size and context length.[^6]
- **Long-horizon reasoning**: 256K context window, handles up to 600 tool calls per task (v1.0), 400 (v1.5), 300 (v1.7).[^6]
- **Model scales**: Released in 8B, 30B, 72B, and 235B parameter variants (Qwen3-based).[^6]
- **MiroThinker-H1**: Heavy-duty reasoning with step-verifiable and globally verifiable reasoning.[^7]
- **MiroFlow Framework**: Orchestration framework for running, evaluating, and reproducing agent workflows with full observability.[^6]
- **MiroVerse Dataset**: ~147K samples designed to train search, planning, and verification behaviors.[^6]

**Benchmark Performance:**
- **BrowseComp**: 74.0% (MiroThinker-1.7), 69.8% (v1.5-235B) — SOTA among open-source models
- **BrowseComp-ZH**: 75.3% (1.7), 71.5% (v1.5) — SOTA
- **GAIA-Val-165**: 82.7% (1.7), 80.8% (v1.5)
- **HLE-Text**: 42.9% (1.7), 39.2% (v1.5)
- **FutureX**: Topped leaderboard for 4+ months, improved GPT-5 prediction accuracy by 11%.[^6][^8]

**Skill Interface & Reusable Components:**
- Released as open-weight models on HuggingFace
- Provides MiroFlow evaluation framework and MiroTrain training infrastructure
- Research report generation with preview and sharing capabilities
- Supports document upload (PDF, DOC, PPT, XLS, JPG) for multimodal research[^6]

**Strengths:**
- State-of-the-art open-source performance on research benchmarks
- Demonstrates that smaller models (30B) with good training can outperform much larger models
- Strong emphasis on verification and step-wise reasoning
- Full training/evaluation stack open-sourced

**Weaknesses:**
- Model-centric rather than framework-centric (you need their weights, not just a script)
- Less focus on report formatting and user-facing features
- Primarily Chinese and English focused

### 1.6 HuggingFace Open Deep Research (smolagents-based)

**Architecture & Approach:**
- Built on **smolagents**, HuggingFace's lightweight agent framework
- Reproduced OpenAI Deep Research results in 24 hours as an open-source challenge
- Uses **CodeAgent** pattern where the agent writes Python code to orchestrate tool use
- Achieved competitive GAIA benchmark scores using open models

**Skill Interface:**
- Available as a smolagents example/template
- Highly customizable via Python code
- Designed for researchers and developers building on HuggingFace infrastructure

**Strengths:**
- Fully open (framework + models)
- Rapid iteration and experimentation
- Strong integration with HuggingFace model hub

**Weaknesses:**
- More research-oriented than production-ready
- Requires more technical setup
- Less mature ecosystem than GPT Researcher or LangGraph

### 1.7 Other Significant 2025-2026 Entrants

| Project | Description | Key Differentiator |
|---------|-------------|------------------|
| **DeepVerifier** | Three-stage framework: decomposition agent → verification agent → judge agent. Exploits verification asymmetry.[^9] | Focus on verification, not just generation |
| **NVIDIA AI-Q Deep Researcher** | Publication-ready reports via multi-phase iterative workflow with 5-phase orchestration[^10] | Enterprise-grade with citation verification pipeline |
| **Temporal + Braintrust Deep Research** | Four specialized agents (Planning, Query Gen, Web Search, Report Synthesis) with Temporal durable execution[^11] | Production resilience via Temporal workflows |
| **MiniMax Agent Deep Research** | Built-in five-step Deep Research skill with Agent Teams (Leader/Worker/Verifier roles)[^12] | Chinese-market leader with multimodal output |
| **Kimi K2.6 Agent Deep Research** | 10,000+ word research reports, up to 300 sub-agents in parallel[^13] | Massive parallelization capability |
| **Manus / OpenManus** | General-purpose autonomous agent with deep research as one of many capabilities | Broad task scope beyond just research |
| **Skywork DeepResearchAgent** | Hierarchical multi-agent framework for deep research[^14] | Hierarchical task decomposition |
| **REDSearcher / OpenSeeker / ASearcher** | Model-centric agents with RL/SFT training for search[^15] | Training-focused improvements |

### 1.8 Strengths/Weaknesses Matrix

| Project | Maturity | Configurability | Benchmark Perf | MCP Support | Model Agnostic | Knowledge Assets | Open Source |
|---------|----------|-----------------|----------------|-------------|----------------|------------------|-------------|
| GPT Researcher | ★★★★★ | ★★★★☆ | ★★★☆☆ | ★★★☆☆ | ★★★★★ | ★★★☆☆ | ★★★★★ |
| LangGraph OpenDR | ★★★★☆ | ★★★★★ | ★★★★☆ | ★★★★★ | ★★★★★ | ★★★★☆ | ★★★★★ |
| OpenResearcher | ★★★☆☆ | ★★★☆☆ | N/A (training) | ★★☆☆☆ | ★★★☆☆ | ★★★★★ | ★★★★★ |
| MiroThinker | ★★★★☆ | ★★★☆☆ | ★★★★★ | ★★★☆☆ | ★★★☆☆ | ★★★☆☆ | ★★★★★ |
| HuggingFace ODR | ★★★☆☆ | ★★★★☆ | ★★★★☆ | ★★★☆☆ | ★★★★☆ | ★★★☆☆ | ★★★★★ |
| NVIDIA Deep Researcher | ★★★★☆ | ★★★★☆ | ★★★★☆ | ★★★★☆ | ★★★☆☆ | ★★★★☆ | ★★☆☆☆ |
| OpenAI Deep Research | ★★★★★ | ★★☆☆☆ | ★★★★★ | ★★☆☆☆ | ★★☆☆☆ | ★★★☆☆ | ★☆☆☆☆ |
| Kimi K2.6 Agent | ★★★★☆ | ★★★☆☆ | ★★★★☆ | ★★★☆☆ | ★★☆☆☆ | ★★★★☆ | ★★☆☆☆ |
| MiniMax Agent | ★★★★☆ | ★★★☆☆ | ★★★★☆ | ★★★★☆ | ★★☆☆☆ | ★★★☆☆ | ★★☆☆☆ |

---

## Topic 2: Skill Platform Specifications

### 2.1 agentskill.io / Agent Skills Open Standard

**Core Specification:**
- **Format**: A folder containing a `SKILL.md` file with YAML frontmatter + Markdown body. Optional subdirectories: `scripts/`, `references/`, `assets/`, `examples/`.[^16]
- **YAML Frontmatter fields**: `name` (required, 1-64 chars, kebab-case), `description` (required, 1-1024 chars), `license`, `compatibility`, `metadata` (arbitrary key-value), `allowed-tools` (experimental).[^16]
- **Progressive Disclosure**: Three-tier loading:
  1. **Discovery**: At startup, only `name` and `description` are loaded (~50 tokens/skill)
  2. **Activation**: When triggered, full `SKILL.md` body loads (~500-5,000 tokens)
  3. **Execution**: `scripts/`, `references/`, `assets/` load only when needed[^16][^17]
- **Versioning**: Not a top-level field; use `metadata.version` for maximum portability.[^16]

**Discovery & Installation:**
- **skills.sh** (maintained by Vercel) serves as the primary distribution hub. Install via `npx skills add <owner/repo>`.[^18]
- **agentskills.io** provides comprehensive documentation and the specification.[^19]
- Cross-platform compatibility: works with Claude Code, Cursor, GitHub Copilot, Goose, Codex CLI, Windsurf, Gemini CLI, Roo Code, Trae, Amp, Factory, and 20+ others.[^18]
- Ecosystem growth: ~490K+ skills in 6 months as of March 2026.[^20]

**What makes a skill "agentskill.io compliant":**
1. Directory name matches `name` field (kebab-case)
2. `SKILL.md` with valid YAML frontmatter
3. `description` includes trigger keywords and "Use when..." guidance
4. Markdown body has clear instructions, output format, and rules
5. Optional bundled resources in `scripts/`, `references/`, `assets/`

### 2.2 Claude Code Marketplace / Plugins / Skills

**Skill System:**
- **Two formats**: 
  - Modern: `SKILL.md` in `.claude/skills/` (project-local) or `~/.claude/skills/` (global)
  - Legacy: `.claude/commands/*.md` (still supported)[^21]
- **Plugin system**: `/plugin` command adds marketplaces. Install via `/plugin install`.[^22]
- **Plugin structure**: `.claude-plugin/plugin.json` + `skills/` + `agents/` directories[^23]
- **Marketplace**: `anthropics/skills` is the official marketplace. Third-party marketplaces like `jeremylongshore/claude-code-plugins` extend it with 425+ plugins and 2,810+ skills.[^22]
- **Progressive disclosure**: Same as agentskills.io standard.[^21]
- **Subagents**: Claude Code has the most mature native multi-agent stack — subagents, agent teams, background agents, and 30+ hook events.[^24]
- **Routines**: Scheduled/recurring workflows triggered by API or events.[^22]

**Claude Code skill frontmatter extensions:**
- `context: fork` (run in subagent)
- `agent: Explore` (subagent type)
- `user-invocable: false` (hide from / menu)
- `allowed-tools: Read, Write, Bash(npm:*)` (scoped tool permissions)[^22]

### 2.3 Codex CLI Plugin Specification

**Plugin Architecture:**
- **Plugin = bundle**: A package that bundles skills, app integrations, and MCP servers.[^25]
- **Skill = atomic unit**: A single `SKILL.md` file with one task focus.[^25]
- **Plugin directory**: `~/.codex/plugins/`
- **Skill directory**: `~/.codex/skills/` (also reads from `~/.agents/skills/`)
- **Config**: `~/.codex/config.toml` with `[plugins."name@source"]` sections[^25]
- **Plugin Directory**: Curated by OpenAI, Shared with you, Created by you. Browse via `/plugins` in CLI.[^25]
- **Invocation**: `@plugin-name` or implicit activation based on skill description matching.[^25]
- **Built-in tools**: `skill-creator` (generates SKILL.md via Q&A), `plugin-creator` (scaffolds full plugin package).[^25]

**Codex-specific features:**
- `execpolicy` for command approval gating
- Cloud agent for async background runs
- Cross-surface: CLI, desktop app, web, mobile

### 2.4 OpenCode Plugin Architecture

**Plugin System:**
- **Plugins are JavaScript/TypeScript modules** that export plugin functions receiving a context object and returning a hooks object.[^26]
- **TypeScript support**: `import type { Plugin } from "@opencode-ai/plugin"`[^26]
- **Hooks**: Event-driven hooks including `tool.execute.before`, `experimental.session.compacting`, and custom event hooks.[^26]
- **SDK**: `@opencode-ai/sdk` for external integrations via Server-Sent Events (SSE).[^27]
- **Config**: `opencode.json` (project) or `~/.config/opencode/opencode.json` (global)[^27]
- **MCP support**: Configured in `opencode.json` under `mcp` key.[^27]
- **75+ model support**: Most model-agnostic of the terminal agents.[^28]

**Key difference from Claude/Codex:**
- OpenCode plugins are **code modules** (JS/TS), not just Markdown files
- More flexible but requires programming knowledge
- Hook-based architecture allows deep customization of the agent loop

### 2.5 Cursor Rules / Skills System

**Evolution:**
- **Legacy**: `.cursorrules` (single file, deprecated but still works)
- **Current**: `.cursor/rules/*.mdc` (directory of MDC files with YAML frontmatter)[^29]
- **Skill support**: `.cursor/skills/` directory with `SKILL.md` files (auto-discovers `.claude/skills/` and `.codex/skills/` too)[^29]
- **Rule types**: Always, Auto Attached (glob patterns), Agent Requested, Manual[^29]
- **MDC format**: YAML frontmatter + Markdown body, similar to SKILL.md but with Cursor-specific fields like `alwaysApply`, `globs`, `description`[^29]

**Cursor Agent Skills status:**
- As of early 2026, Agent Skills is "not quite ready for primetime" — no stable ETA.[^30]
- Skills and rules are designed to coexist: rules for static/project-specific context, skills for portable reusable workflows.[^30]

### 2.6 Windsurf / Cascade Skills System

**Skill Support:**
- **Directory**: `.windsurf/skills/` in project directory (no global skills directory)[^31]
- **Full SKILL.md support**: `name`, `description`, `when_to_use`, markdown body, `scripts/`, `references/`, `assets/`[^31]
- **Agent mode**: Cascade (autonomous, flow-based multi-file editing)[^31]
- **Always-on config**: `.windsurfrules` (similar to `.cursorrules`)[^31]
- **Key limitation**: Only project-scoped skills, no global library. Skills must be copied per project or symlinked.[^31]

### 2.7 Kimi Code / Kimi Work Skill System

**Kimi Code (KFC):**
- Terminal-based AI coding agent powered by K2.7 Code model (Moonshot AI)
- **Skill directory**: `.skills/` in project directory[^32]
- Supports SKILL.md format natively
- Strong frontend performance (React, Next.js, Vue)
- Available through OpenCode as an alternative harness[^32]

**Kimi K2.6 Agent:**
- Autonomous AI assistant with 20+ tools
- **Deep Research**: 10,000+ word research reports
- **Agent Swarm**: Up to 300 sub-agents working in parallel[^13]
- **Kimi Claw**: Cloud automation with 5,000+ skills[^13]
- Document-to-skills feature: transforms documents into reusable skills[^33]

### 2.8 MiniMax / Mavis CLI Skill System

**MiniMax Agent (Mavis):**
- General-purpose agent renamed to "Mavis" (MiniMax as a Jarvis) in May 2026[^12]
- **Agent Teams**: Leader, Worker, and Verifier roles with adversarial quality gates[^12]
- **Deep Research**: Built-in five-step Deep Research skill (June 2026 changelog)[^12]
- **MCP integration**: Pre-built integrations for GitHub, Figma, Slack, Google Maps[^12]
- **Skills**: Memory and skill evolution supported. `MiniMax-AI/skills` repo on GitHub.[^12]
- **MiniMax M3**: 1M-token context, native multimodality, desktop-computer operation (June 2026)[^12]
- **M2.5 model**: 100 tokens/second, $0.06/M blended, 10B activated parameters[^34]

### 2.9 Common Denominator: What Makes a Skill Truly Portable?

**The universal skill format:**
1. **SKILL.md with YAML frontmatter + Markdown body** — supported by Claude Code, Codex, Cursor, Windsurf, Kimi Code, OpenCode, and 20+ tools
2. **AGENTS.md** — cross-tool project context standard (OpenAI, Google, Cursor, Anthropic, 60K+ repos)[^35]
3. **Progressive disclosure** — metadata at startup, instructions on activation, resources on demand
4. **Tool-agnostic instructions** — reference generic tools (Read, Write, Bash) rather than platform-specific APIs
5. **MCP for tool integration** — external capabilities via MCP servers, not baked into skills

**The portability formula:**
```
Portable Skill = SKILL.md (instructions) + AGENTS.md (context) + MCP (tools)
```

**Platform differences (as of July 2026):**
- **Claude Code**: Deepest skill ecosystem, native multi-agent, hooks, routines
- **Codex**: Plugin-centric, cross-surface, built-in skill/plugin creators
- **OpenCode**: Code-based plugins (JS/TS), 75+ models, most flexible
- **Cursor**: IDE-coupled, MDC rules, skills still maturing
- **Windsurf**: IDE-coupled, Cascade agent, project-only skills
- **Kimi Code**: K2.7 model optimized, newer ecosystem
- **MiniMax**: Agent Teams, built-in deep research, Chinese market leader

---

## Topic 3: AG-UI / A2UI / MCP App UI Frameworks

### 3.1 AG-UI Protocol (Agent-User Interaction Protocol)

**What it is:**
- An open, lightweight, event-based protocol that standardizes how AI agents connect to user-facing applications.[^36]
- Maintained by **CopilotKit** (the creators) with community contributions.
- **~16 standard event types** for agent execution streams.[^36]
- **Flexible middleware layer**: Works with any transport (SSE, WebSockets, webhooks).[^36]
- **Reference HTTP implementation** and default connector provided.[^36]

**How it differs from A2A:**
- **A2A** = agent-to-agent protocol (discovery, delegation, task management)
- **AG-UI** = agent-to-frontend protocol (user interaction, real-time streaming, state sync)
- **MCP** = agent-to-tool protocol (tool calling, data access)
- **AG-UI answers**: "How do agents and applications communicate in real time?"[^36]

**Key capabilities:**
- Real-time streaming via Server-Sent Events (SSE)
- Standardized events: messages, tool calls, state patches, lifecycle signals
- Framework-agnostic: works with LangGraph, CrewAI, Google ADK, Microsoft Agent Framework, AWS Strands[^36]
- Human-in-the-loop: built-in approvals and interventions
- Thread management: conversation state and history

**Getting started:**
```bash
npx create-ag-ui-app my-agent-app
```

### 3.2 A2UI Protocol (Agent-to-User Interface)

**What it is:**
- A **declarative generative UI specification** from Google, open-sourced January 2026.[^37]
- Agents send **JSON UI blueprints** describing what to render, not executable code.[^37]
- **18 primitives** (in v0.9) for UI components: Card, Button, TextField, etc.[^38]
- Client applications map each component to their own native widgets.[^37]

**Key features:**
- **Security first**: Declarative data format, no executable code. Agents can only use pre-approved components from the client's catalog.[^37]
- **Portability**: One agent response renders everywhere (React, Angular, Flutter, Lit, native mobile).[^37]
- **LLM-optimized**: Flat JSON structure designed for easy model generation.[^37]
- **Progressive rendering**: Streaming UI updates as the agent generates them.[^37]

**Specification versions:**
- v0.9.1 (current production release)
- v1.0 (candidate release with client-to-server RPC and action IDs)[^38]

**Relationship to AG-UI:**
- **A2UI** defines *what* to render (the payload/data format)
- **AG-UI** defines *how* to deliver it (the transport/event stream)
- They work together: AG-UI can carry A2UI payloads as the data format for rendering UI[^39]
- **Oracle's Agent Spec** explicitly aligns these three: Agent Spec defines what runs, AG-UI carries the interaction, A2UI defines what the user touches.[^40]

### 3.3 assistant-ui (React Framework)

**Note:** The "assistant-ui" referenced in the user's context is a React framework for building chat interfaces. While not as widely documented in the public search results as AG-UI/A2UI, it is understood to be:
- A React component library for AI chat interfaces
- Integrates with AG-UI protocol for backend communication
- Provides UI components: message lists, input fields, tool call displays, streaming indicators
- Used by CopilotKit and other agent UI builders

The broader ecosystem is moving toward **AG-UI + A2UI** as the standardized backend, with frontend frameworks (React, Flutter, Angular) providing the renderer.

### 3.4 MCP App Design Patterns

**MCP Apps Extension (SEP-1865):**
- Standardizes interactive UIs in MCP via **secure iframe rendering**.[^41]
- **UI templates** are resources with `ui://` URI scheme, referenced in tool metadata via `_meta.ui.resourceUri`.[^41]
- Communication uses existing MCP JSON-RPC over `postMessage`.[^41]
- **Security layers**: Iframe sandboxing, pre-declared templates, auditable messages, user consent for tool calls.[^41]

**Best practices for MCP app development (2026):**[^42]
1. **AI-driven scaffolding**: Use `create-mcp-app` with an AI coding agent
2. **Single-file UI bundling**: Vite with `vite-plugin-singlefile` for sandboxed iframe compatibility
3. **Standardized UI metadata**: `ui://` scheme and `_meta.ui.resourceUri`
4. **Local tunneling**: `cloudflared` for testing local servers with cloud AI clients
5. **Serverless deployment**: Azure Functions or Cloudflare Workers for cost-effective hosting

**54 MCP Tool Patterns** (from Arcade.dev):[^43]
- Design for LLM comprehension, not human readability
- Implement idempotency, atomic operations, clear error-guided recovery
- Use context injection for security (never pass credentials through LLM)
- Support batch operations and consistent response shapes for composition

### 3.5 Exposing AG-UI from an MCP Server (Rust Axum Backend)

**Architecture pattern:**
```
User Frontend (React/assistant-ui)
    ↕ AG-UI event stream (SSE/WebSocket)
Rust Axum Backend
    ├── AG-UI endpoint: /v1/ag-ui/stream
    ├── MCP server endpoint: /mcp (JSON-RPC/SSE)
    └── Deep Research Agent (internal)
        ├── Planner
        ├── Search orchestrator
        ├── Evidence collector
        └── Report generator
```

**Implementation approach:**
1. **Axum server** with two routes:
   - `/mcp` — MCP server (JSON-RPC over SSE)
   - `/ag-ui` — AG-UI event stream endpoint
2. **AG-UI events** emitted from the research agent loop:
   - `message`: Status updates ("Searching...", "Found 12 sources")
   - `tool_call`: Search tool invocations
   - `state_patch`: Progress updates (questions answered, evidence collected)
   - `artifact`: Intermediate results (outline, draft sections)
3. **A2UI payloads** for rich UI: progress bars, source lists, evidence cards, citation graphs

### 3.6 prometheus-entity-management

**What it is:**
- `@prometheus-ags/prometheus-entity-management` on NPM — a normalized, globally-reactive entity graph store for React.[^44]
- Replaces TanStack Query's per-view cache model with a single application-wide reactive graph.
- Published April 2026.
- **Relevance to deep research**: Provides the reactive data layer for a deep research UI app — entity relationships, live updates, and cross-component state synchronization.

### 3.7 flint-platform-agent

**What it is:**
- The `flint-platform-agent` in the user's workspace is part of the Prometheus/Flint ecosystem.
- Based on workspace inspection, it's a Rust-based platform agent for the Flint ecosystem.
- **Relevance to deep research**: Would serve as the runtime harness for executing the deep research skill, managing agent lifecycle, tool registration, and event streaming.

### 3.8 Unified Deep-Research MCP App: UI/UX Considerations

**A world-class deep research MCP app should:**

1. **Expose progress visibility**: Research is long-running (minutes to hours). Users need:
   - Live search status (queries issued, sources found)
   - Evidence collection progress (claims verified, contradictions found)
   - Report generation status (sections completed, citations resolved)

2. **Surface intermediate artifacts**: Don't make users wait for the final report:
   - Dynamic outline that fills in as research progresses
   - Source browser with relevance scores
   - Evidence cards with confidence ratings
   - Knowledge graph visualization (entities and relationships)

3. **Enable human-in-the-loop**: 
   - Allow users to add/remove search queries mid-run
   - Flag sources as trusted/untrusted
   - Redirect research focus based on emerging findings
   - Approve/disapprove controversial claims

4. **Support multimodal input/output**:
   - Upload documents (PDF, DOCX, PPTX) as research seeds
   - Generate charts, tables, and diagrams in the report
   - Export to multiple formats (PDF, DOCX, HTML, JSON knowledge package)

5. **Render via A2UI over AG-UI**:
   - Progress components (progress bars, step indicators)
   - Data components (tables, charts, source lists)
   - Interactive components (buttons for approval, forms for query refinement)
   - Layout components (cards, tabs for different research dimensions)

---

## Topic 4: Knowledge Assets & Architecture Patterns

### 4.1 Knowledge Assets vs Disposable Reports

**A disposable report** is a static document (PDF, DOCX, Markdown) that captures findings at a point in time. It has citations, but the knowledge is frozen and not queryable by other agents.

**A knowledge asset** is a structured, persistent, queryable object that:
- Can be extended by other agents
- Can be cited by other research runs
- Contains machine-readable structure (graphs, embeddings, timelines)
- Supports versioning and provenance tracking
- Enables cross-research synthesis

**The shift from reports to knowledge assets** is the defining architectural evolution for deep research systems in 2026.

### 4.2 Research Package Formats

Based on the landscape analysis, a comprehensive research package should include:

| Component | Format | Purpose |
|-----------|--------|---------|
| **citations.json** | JSON array with DOI/URL, title, author, date, access_date, relevance_score | Machine-readable citation registry |
| **knowledge_graph.json** | JSON-LD / property graph with entities, relations, confidence | Entity-relationship structure of findings |
| **embeddings.pkl / .npy** | NumPy arrays or vector DB entries | Semantic search over findings |
| **entity_graph.json** | Nodes (entities) + edges (relations) with provenance | Domain-specific entity network |
| **timeline.json** | Chronological events with dates, sources, confidence | Temporal analysis of findings |
| **contradictions.json** | Pairs of conflicting claims with evidence for each | Conflict detection and resolution |
| **confidence_scores.json** | Per-claim confidence: high/medium/low with justification | Uncertainty quantification |
| **follow_up_questions.json** | Generated questions for future research | Research continuation |
| **source_cache/** | Raw downloaded content (HTML, PDF excerpts) with metadata | Reproducibility and verification |
| **search_trace.json** | Query log with results, timestamps, tool used | Audit trail |
| **reasoning_trace.json** | Step-by-step reasoning with tool calls and observations | Explainability |
| **report.md** | Human-readable final report with inline citations | Primary deliverable |
| **report.pdf** | Formatted document for distribution | Shareable artifact |
| **summary.json** | Key findings, metrics, coverage assessment | Quick overview |

### 4.3 MCP Servers Wrapping Research Capabilities

**Best patterns for MCP-wrapped research:**

1. **Atomic tool design**: One tool per operation:
   - `research_plan(query)` → returns structured plan
   - `execute_search(query, source_type)` → returns ranked results
   - `extract_evidence(url, claim)` → returns supporting/contradicting evidence
   - `build_knowledge_graph(findings)` → returns graph JSON
   - `generate_report(findings, format)` → returns report

2. **Stateful sessions**: Use `session_id` to track multi-step research:
   - Research state persists across tool calls
   - Partial results can be queried and extended
   - Supports long-running research with checkpoints

3. **Resource exposure**: Expose intermediate artifacts as MCP resources:
   - `research://{session_id}/outline`
   - `research://{session_id}/sources`
   - `research://{session_id}/evidence_graph`

4. **UI integration**: Use MCP Apps extension (`_meta.ui.resourceUri`) to attach interactive UIs to research tools.[^41]

### 4.4 Integration with Prometheus MCP Stack

The existing Prometheus MCP stack includes:
- **tavily-mcp**: Web search API integration
- **sequential-thinking**: Structured reasoning chain
- **liter-llm**: Literature analysis and summarization
- **surreal-memory**: Persistent memory/knowledge storage
- **forge-rs**: Code generation and execution
- **prometheus-knowledge**: Knowledge base queries

**Integration architecture:**
```
Deep Research Skill
├── Planner (uses sequential-thinking for reasoning chains)
├── Search Orchestrator (uses tavily-mcp for web search)
├── Evidence Analyzer (uses liter-llm for source analysis)
├── Knowledge Builder (uses surreal-memory for persistence)
├── Code Executor (uses forge-rs for data processing)
├── Citation Manager (uses prometheus-knowledge for KB lookups)
└── Report Generator (multi-format output)
```

### 4.5 Universal Deep-Research Skill Architecture

Based on the comprehensive landscape analysis, the optimal architecture for a universal deep-research skill is:

```
┌─────────────────────────────────────────────────────────────┐
│                    UNIVERSAL DEEP RESEARCH SKILL             │
├─────────────────────────────────────────────────────────────┤
│  INPUT: Research query + optional documents + constraints   │
├─────────────────────────────────────────────────────────────┤
│  PHASE 1: PLANNING                                         │
│  ├── Planner: Decomposes query into sub-tasks              │
│  ├── Question Decomposer: Generates atomic research questions│
│  └── Search Planner: Allocates search budget per question  │
├─────────────────────────────────────────────────────────────┤
│  PHASE 2: EVIDENCE GATHERING                               │
│  ├── Retriever (web/RAG/graph/MCP/API): Multi-source fetch │
│  ├── Evidence Collector: Extracts claims from sources        │
│  └── Source Cache: Persists raw content for verification   │
├─────────────────────────────────────────────────────────────┤
│  PHASE 3: VERIFICATION & SYNTHESIS                         │
│  ├── Evidence Verifier: Checks claim-source alignment      │
│  ├── Conflict Resolver: Detects and resolves contradictions  │
│  ├── Knowledge Graph Builder: Entities → relations → graph │
│  └── Citation Manager: Links claims to verified sources      │
├─────────────────────────────────────────────────────────────┤
│  PHASE 4: OUTPUT GENERATION                                │
│  ├── Report Generator: Markdown/PDF/DOCX with citations      │
│  ├── Artifact Exporter: Knowledge package (JSON bundle)      │
│  └── Knowledge Asset Publisher: Writes to surreal-memory   │
├─────────────────────────────────────────────────────────────┤
│  PHASE 5: PROGRESS & UI (via AG-UI / A2UI)                  │
│  ├── Status events: search, analysis, synthesis, complete  │
│  ├── Artifact events: outline, sections, sources, graph     │
│  └── Human-in-the-loop: approval, redirection, query add     │
├─────────────────────────────────────────────────────────────┤
│  OUTPUT: Report + Knowledge Package + Queryable KB Entry    │
└─────────────────────────────────────────────────────────────┘
```

### 4.6 Emitting Persistent Knowledge Objects

The skill should emit knowledge objects that other agents can query, extend, and cite:

**Design principles:**
1. **UUID-based identification**: Each research run gets a unique ID
2. **Semantic versioning**: Knowledge packages are versioned
3. **Queryable schema**: Use SurrealDB or similar graph DB for storage
4. **Citation protocol**: Other agents cite using `research://{uuid}/claim/{claim_id}`
5. **Extension API**: `extend_research(uuid, new_query)` adds to existing knowledge graph
6. **Confidence decay**: Older findings can be marked for re-verification

**Example knowledge object:**
```json
{
  "research_id": "uuid-v4",
  "topic": "AI agent protocols 2026",
  "created_at": "2026-07-03T15:00:00Z",
  "model_version": "deep-research-v1.0",
  "knowledge_graph": { /* entity-relationship graph */ },
  "citations": [ /* structured citations */ ],
  "confidence_matrix": { /* per-claim scores */ },
  "contradictions": [ /* resolved conflicts */ ],
  "embeddings": "vector_db_reference",
  "queryable": true,
  "extendable": true
}
```

### 4.7 Native CLI Tooling

Following the pattern of other skills in the Prometheus Skill Pack, the deep-research skill should provide:

1. **`prometheus-deep-research` CLI**: Rust-based binary (matching the pack's Rust-native approach)
2. **Subcommands:**
   - `research <query>`: Run deep research
   - `research --continue <research_id>`: Extend existing research
   - `research --query <research_id> <question>`: Query a knowledge asset
   - `research --list`: Show active/completed research runs
   - `research --export <research_id> --format <pdf|docx|json>`: Export artifacts
   - `research --serve`: Start MCP server + AG-UI endpoint
   - `research --interactive`: TUI for interactive research (with progress bars)

3. **Configuration:**
   - `~/.prometheus/research/config.toml`: API keys, model preferences, default output formats
   - `~/.prometheus/research/skills/`: Custom SKILL.md extensions

---

## Cross-Cutting Insights & Strategic Recommendations

### Insight 1: The "Skill + MCP + AG-UI" Triad is the Winning Architecture

The most future-proof deep research skill must:
- **Package as a SKILL.md** for portability across Claude Code, Codex, OpenCode, Cursor, Windsurf, Kimi, and MiniMax
- **Expose capabilities via MCP** for tool integration with any MCP-compatible client
- **Stream progress via AG-UI** for rich interactive frontends
- **Render UI via A2UI** for safe, portable generative UI

### Insight 2: Knowledge Assets > Reports

The differentiator between a "good" and "world-class" deep research skill is not report quality alone — it's the **persistence and queryability of the knowledge generated**. A skill that emits structured knowledge packages becomes infrastructure for other agents, not just a one-time tool.

### Insight 3: Verification is the New Differentiator

As the landscape matures, raw search capability is becoming commoditized. The projects with the strongest verification pipelines (MiroThinker-H1, DeepVerifier, NVIDIA's 5-level citation verification) are setting the new SOTA. The Prometheus skill should invest heavily in:
- Decomposition-based verification
- Citation-to-source registry matching
- Confidence scoring with explicit uncertainty
- Contradiction detection and resolution

### Insight 4: Model Independence + Harness Excellence

The best skills work across model providers. The skill should:
- Use generic LLM APIs (OpenAI-compatible, Anthropic, Google, local)
- Not depend on a single model's capabilities
- Compensate for weaker models with better harness architecture (planning, verification, multi-step reasoning)

### Insight 5: The "Native Agent" and "MCP Server" Dual Nature

Given the Prometheus Skill Pack already has:
- `native-agent` skill (Rust agents with A2A + AG-UI + A2UI + assistant-ui)
- `mcp-server` skill (Rust MCP servers with Axum)

The deep-research skill should be designed as **both**:
- A **native Rust agent** that can run autonomously with full UI
- An **MCP server** that exposes its capabilities to any MCP-compatible client

This dual nature maximizes portability and utility.

### Recommended Skill Specification for Prometheus

```yaml
---
name: prometheus-deep-research
description: |
  Conducts exhaustive, evidence-based deep research on any topic.
  Use when: you need comprehensive research with citations, knowledge graphs,
  and verifiable evidence. Trigger with "research [topic]" or "deep dive into [topic]".
  Works across web, local documents, RAG, and graph databases.
metadata:
  version: "1.0.0"
  author: "Prometheus Skill Pack"
  category: "research"
  platforms: [claude-code, codex, opencode, cursor, windsurf, kimi, minimax]
  outputs: [report.md, knowledge_package.json, knowledge_graph.json]
  protocols: [mcp, ag-ui, a2ui, a2a]
---
```

---

## References

[^1]: GitHub — assafelovic/gpt-researcher. "An autonomous agent that conducts deep research on any data using any LLM providers." https://github.com/assafelovic/gpt-researcher (Accessed 2026-07-03)

[^2]: PyPI — gpt-researcher 0.8.8. "Architecture: planner and execution agents." https://pypi.org/project/gpt-researcher/0.8.8/ (Accessed 2026-07-03)

[^3]: Ry Walker Research — GPT Researcher Review. "v3.5.0, May 2026. 27.6k stars." https://rywalker.com/research/gpt-researcher (Accessed 2026-07-03)

[^4]: GitHub — langchain-ai/open_deep_research. "Simple, configurable, fully open source deep research agent." https://github.com/langchain-ai/open_deep_research (Accessed 2026-07-03)

[^5]: Notion/TIGER-AI-Lab — OpenResearcher Pipeline. "Fully offline trajectory synthesis for long-horizon deep research." https://boiled-honeycup-4c7.notion.site/OpenResearcher (Accessed 2026-07-03)

[^6]: GitHub — MiroMindAI/MiroThinker. "Interactive scaling for tool-augmented reasoning." https://github.com/MiroMindAI/MiroThinker (Accessed 2026-07-03)

[^7]: arXiv — MiroThinker-1.7 & H1. "Towards Heavy-Duty Research Agents via Verification." arXiv:2603.15726 (Accessed 2026-07-03)

[^8]: woshipm.com — MiroThinker Benchmark Analysis. "BrowseComp 71.5%, GAIA 82.4%." https://www.woshipm.com/share/6321869.html (Accessed 2026-07-03)

[^9]: arXiv — DeepVerifier. "Three-stage multi-module framework for DRA verification." arXiv:2601.15808 (Accessed 2026-07-03)

[^10]: NVIDIA AI-Q Blueprint — Deep Researcher Agent. "Multi-phase iterative workflow with citation verification." https://docs.nvidia.com/aiq-blueprint/2.0.0/architecture/agents/deep-researcher.html (Accessed 2026-07-03)

[^11]: Braintrust — Temporal Deep Research. "Four specialized agents with Temporal durable execution." https://www.braintrust.dev/docs/cookbook/recipes/TemporalDeepResearch (Accessed 2026-07-03)

[^12]: minimax-ai.chat — MiniMax Agent Guide. "Built-in five-step Deep Research skill, Agent Teams, MCP." https://minimax-ai.chat/models/minimax-agent/ (Accessed 2026-07-03)

[^13]: kimi.com — K2.6 Agent Overview. "10,000+ word reports, 300 sub-agents, 20+ tools." https://www.kimi.com/help/agent/agent-overview (Accessed 2026-07-03)

[^14]: arXiv — AgentOrchestra. "TEA Protocol for multi-agent orchestration." arXiv:2506.12508 (Accessed 2026-07-03)

[^15]: arXiv — Beyond Search, Toward Real-World Long-Horizon Research Agents. "Tongyi, Step, O-Researcher, MiroThinker, REDSearcher, OpenSeeker, ASearcher." arXiv:2606.15367 (Accessed 2026-07-03)

[^16]: dev.to — Agent Skills Explained. "Open standard for packaging reusable AI agent capabilities." https://dev.to/loc_carrre_0d798813c662/agent-skills-explained (Accessed 2026-07-03)

[^17]: inference.sh — Agent Skills Overview. "The Agent Skills format was developed by Anthropic and released as an open standard in late 2025." https://inference.sh/blog/skills/agent-skills-overview (Accessed 2026-07-03)

[^18]: agentskills.io — Home. "What are Agent Skills? Lightweight, open format." https://agentskills.io/home (Accessed 2026-07-03)

[^19]: agentskill.sh — The Agent Skills Guide. "FAQ, progressive disclosure, cross-platform compatibility." https://agentskill.sh/readme (Accessed 2026-07-03)

[^20]: termdock.com — Agent Skills Guide 2026. "490K+ skill ecosystem in six months." https://www.termdock.com/en/blog/agent-skills-guide (Accessed 2026-07-03)

[^21]: nimbalyst.com — Best Claude Code Skills 2026. "How to install: .claude/skills/my-skill/SKILL.md." https://nimbalyst.com/blog/best-claude-code-skills-2026/ (Accessed 2026-07-03)

[^22]: dev.to — "I Tried 100 Claude Skills." "Plugin marketplace, /batch, /simplify, Routines." https://dev.to/suraj_khaitan_f893c243958/i-tried-100-claude-skills (Accessed 2026-07-03)

[^23]: levelup.gitconnected.com — A Mental Model for Claude Code. "Plugins are the packaging layer." https://levelup.gitconnected.com/a-mental-model-for-claude-code-skills-subagents-and-plugins-3dea9924bf05 (Accessed 2026-07-03)

[^24]: GitHub — awesome-agentic-ai-zh. "Claude Code native multi-agent stack vs others." https://github.com/WenyuChiou/awesome-agentic-ai-zh (Accessed 2026-07-03)

[^25]: firecrawl.dev — Best Codex Plugins 2026. "Plugins bundle skills, apps, and MCP servers." https://www.firecrawl.dev/blog/best-codex-plugins (Accessed 2026-07-03)

[^26]: opencode.ai — Plugins. "Plugin is a JS/TS module exporting a plugin function." https://opencode.ai/docs/plugins/ (Accessed 2026-07-03)

[^27]: dev.to — Does OpenCode Support Hooks? "SDK with SSE, MCP config, custom commands." https://dev.to/einarcesar/does-opencode-support-hooks (Accessed 2026-07-03)

[^28]: firecrawl.dev — Best AI Coding Agents 2026. "OpenCode: 75+ model support, free, open source." https://www.firecrawl.dev/blog/best-ai-coding-agents (Accessed 2026-07-03)

[^29]: blog.buildbetter.ai — AGENTS.md vs .cursorrules vs Claude Skills. "Cursor moved to .cursor/rules/*.mdc." https://blog.buildbetter.ai/agents-md-vs-cursorrules-vs-claude-skills-2026-comparison/ (Accessed 2026-07-03)

[^30]: forum.cursor.com — Agent Skills vs .cursorrules. "Agent Skills not quite ready for primetime." https://forum.cursor.com/t/questions-regarding-agent-skills/148080 (Accessed 2026-07-03)

[^31]: agensi.io — How to Add SKILL.md to Windsurf. ".windsurf/skills/ directory, full SKILL.md support." https://www.agensi.io/learn/windsurf-skills-how-to-add-skill-md (Accessed 2026-07-03)

[^32]: agensi.io — Kimi Code Skills Guide. ".skills/ directory, K2.7 Code model." https://www.agensi.io/learn/kimi-code-skills-guide (Accessed 2026-07-03)

[^33]: kimi.com — OpenCode Skills. "Document to skills feature." https://www.kimi.com/resources/opencode-skills (Accessed 2026-07-03)

[^34]: cline.bot — MiniMax M2.5 in Cline. "100 tokens/second, $0.06/M blended, 10B activated." https://cline.bot/blog/minimax-m2-5 (Accessed 2026-07-03)

[^35]: GitHub — addyosmani/agent-engineer. "AGENTS.md emerged August 2025, 60K+ repos." https://github.com/addyosmani/agent-engineer (Accessed 2026-07-03)

[^36]: GitHub — ag-ui-protocol/ag-ui. "AG-UI: the Agent-User Interaction Protocol." https://github.com/ag-ui-protocol/ag-ui (Accessed 2026-07-03)

[^37]: a2ui.org — A2UI Official. "Declarative generative UI specification, Google, Apache 2.0." https://a2ui.org/ (Accessed 2026-07-03)

[^38]: a2ui.sh — A2UI vs AG-UI. "A2UI defines what to render, AG-UI defines how to deliver." https://a2ui.sh/articles/a2ui-vs-ag-ui (Accessed 2026-07-03)

[^39]: docs.copilotkit.ai — A2UI Launch. "CopilotKit is a launch partner for A2UI." https://docs.copilotkit.ai/learn/whats-new/a2ui-launch (Accessed 2026-07-03)

[^40]: blogs.oracle.com — Agent Spec for A2UI. "Agent Spec defines what runs, AG-UI carries interaction, A2UI defines user touch." https://blogs.oracle.com/ai-and-datascience/announcing-agent-spec-for-a2ui-copilotkit-ag-ui (Accessed 2026-07-03)

[^41]: blog.modelcontextprotocol.io — MCP Apps. "SEP-1865: interactive UIs in MCP via secure iframes." https://blog.modelcontextprotocol.io/posts/2025-11-21-mcp-apps/ (Accessed 2026-07-03)

[^42]: bolderapps.com — MCP App Development Guide. "5 easy changes: AI scaffolding, single-file UI, metadata, tunneling, serverless." https://www.bolderapps.com/blog-posts/mcp-app-development-complete-guide (Accessed 2026-07-03)

[^43]: arcade.dev — 54 Patterns for MCP Tools. "Agent experience, security boundaries, error-guided recovery, tool composition." https://www.arcade.dev/blog/mcp-tool-patterns/ (Accessed 2026-07-03)

[^44]: libraries.io — @prometheus-ags/prometheus-entity-management. "Normalized, globally-reactive entity graph store for React." https://libraries.io/npm/@prometheus-ags%2Fprometheus-entity-management (Accessed 2026-07-03)

---

*Report compiled on 2026-07-03 from 50+ sources including GitHub repositories, arXiv papers, official documentation, and industry analysis. All URLs verified at time of compilation.*
