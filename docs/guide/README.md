# Prometheus Skill Pack guide

Use this guide with [the product README](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/README.md) for installation and
first workflows. Individual pages describe their source contracts and evidence
boundaries; a documented feature does not imply every host or harness is certified.
The pack supports standalone work, with optional services and a separately
installed Companion extension for connected controls.

## How this documentation is organized

The guide is built in layers. Read it top to bottom the first time; use it as a reference after that.

### Foundations — the *why* and the *what*

| # | Page | What it covers |
|---|------|----------------|
| 01 | [Introduction](01-introduction.md) | What the skill pack is, who it is for, the autonomy ladder, and the loop posture |
| 02 | [Metaprompting, PMPO, and KBD](02-metaprompting-pmpo-kbd.md) | The methodology: metaprompting, Prometheus Meta-Prompting Orchestration, Knowledge-Based Development, and the theory behind them |
| 03 | [Loop Architecture](03-loop-architecture.md) | The L0–L3 loop levels, nested loops, `loop.json`, the `loop-tick.sh` exit-code contract, feedback sources, escalation, and autonomy gates |
| 04 | [The Pipeline](04-four-layer-pipeline.md) | PMPO → OpenSpec → forge-rs, with C4 container diagrams |

### The substrate — what makes loops compound

| # | Page | What it covers |
|---|------|----------------|
| 05 | [The MCP Server Substrate](05-mcp-substrate.md) | MCP component roles, configured endpoints and research providers |
| 06 | [Memory and Karpathy-Pattern Learning](06-memory-and-learning.md) | The three-layer memory architecture, the self-learning engine, and the cross-session write-back sequence |
| 07 | [Sycophancy Correction](07-sycophancy-correction.md) | The eight patterns (S-01–S-08), modes, strictness, MCP tools, the reflection gate, and the reflection boundary |

### The catalog — every skill

| # | Page | What it covers |
|---|------|----------------|
| 08 | [Skills Overview](08-skills-overview.md) | The skills model, discovery, the AgentSkills.io standard, and the full category index |
| 09 | [Process & Orchestration Skills](09-process-skills.md) | iterative-evolver, the KBD orchestrator and its child skills, pmpo-elicit, pmpo-outer-loop, pmpo-skill-creator, native-agent, liter-llm-bridge, ideation-mindmap, kbd-evolve |
| 10 | [Learn Domain Skills](10-learn-skills.md) | The 12 learn skills (ui-surface, learn-goal, learn-survey, learn-plan, feynman-loop, learn-grade, learn-retain, learn-practice, learn-certify, learn-kb, learn-about-system, learn-harness), FSRS-6 spaced retrieval, KB adapters, and meta-learning for the Prometheus stack |
| 10 | [Language & Domain Skills](10-language-skills.md) | Rust, React, Flutter, Tauri, HTMX, TypeScript, Go, Python, architecture, testing, DevOps, document extraction, and the Flint SDK skills |
| 11 | [The Artifact Refiner](11-artifact-refiner.md) | The artifact-centric refinement engine and all fifteen of its commands |
| 12 | [The Native Agent Generator](12-native-agent-generator.md) | Generating complete Rust agents, the A2A/AG-UI/A2UI protocols, and agent networks |

### The engine room — tools, toolchain, lifecycle

| # | Page | What it covers |
|---|------|----------------|
| 13 | [Tools Reference](13-tools-reference.md) | forge-rs, prometheus-cli, prometheus-knowledge, liter-llm, surreal-memory-server, prometheus-rust-auditor, and prometheus-exec |
| 14 | [The Rust Toolchain & Dynamic Generation](14-rust-toolchain.md) | Why Rust, how the binaries are built, and how the pack generates new skills, CLIs, and MCP servers |
| 15 | [Hooks & Lifecycle](15-hooks-and-lifecycle.md) | Lifecycle observations, scoped learning, progress signals and protected-test integrity |
| 16 | [CLI & Scripts Reference](16-cli-and-scripts.md) | Every installer, validator, and runtime script, plus the npm script surface |
| 16a | [Spec Engines](spec-engines.md) | OpenSpec (default), Spec Kit, native-kbd: detection, pinning, the five-op adapter contract, and how to add or update an engine |

### Deployment — install, run, update, contribute

| # | Page | What it covers |
|---|------|----------------|
| 17 | [Platform Support](17-platform-support.md) | Per-tool support: Claude Code, OpenCode, Codex, Kimi Code, MiniMax, Cursor, Windsurf, Gemini CLI, Roo Code, Zed and Cline |
| 18 | [Plugins & Marketplace](18-plugins-and-marketplace.md) | Claude Code plugins and marketplace, OpenCode plugins, and the distribution model |
| 19 | [Installation](19-installation.md) | Toolchain install (Rust, Go, Node, Docker), the one-command install, and MCP services |
| 20 | [Updating](20-updating.md) | Keeping skills, tools, submodules, and MCP services current with recovery and evidence boundaries |
| 21 | [Contributing](21-contributing.md) | The open-source workflow, validation gates, submodules, and importing skills |

### Closing

| # | Page | What it covers |
|---|------|----------------|
| 22 | [Advantages & Impact](22-advantages-and-impact.md) | What changes about your development process, and why |
| 23 | [Glossary & Sources](23-glossary.md) | Every term defined, every external claim cited |

### UI and team workflows

| # | Page | What it covers |
|---|---|---|
| 24 | [Agent teams](24-agent-teams.md) | Create and adopt teams; role ownership and task lifecycle; messages, handoffs and cross-project requests; models, reasoning and scoped memory |
| 25 | [UI/UX routing](25-ui-ux-routing.md) | Context authority, selective skills, portable installation and completed-phase evidence |
| 26 | [Services, ownership and recovery](26-service-operations.md) | Repositories, optional Companion, memory/model services, installation and recovery |

### Operational

| Document | What it covers |
|---|---|
| [Production Readiness Report](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/production-readiness-report.md) | Evidence table separating artifact, disposable-runtime, installed-service, and external-deployment certification |
| [Deployment Modes](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/main/docs/deployment-modes.md) | Mode 0-3 capability matrix — which services are required for which features |
| [Dynamic Operations with Prometheus Exec](/docs/execution/overview-and-use-cases) | Choosing generated programs versus native agents, Tier P/W/R theory, APIs, receipts, examples, security, and platform evidence |

---


Design documents, old release receipts and future-work plans are retained as
history. They are not current installation instructions. Documentation changes
follow the same implementation-first, final local integration boundary as code.
