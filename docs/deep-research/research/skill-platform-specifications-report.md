# Skill Platform Specifications: Cross-Platform Agent Skills Landscape

## Research Report
**Date:** 2026-07-03  
**Topic:** Skill Platform Specifications (agentskill.io, Claude, Codex, OpenCode, Cursor, Windsurf, Kimi, MiniMax)  
**Purpose:** Foundational research for designing a universal "deep-research" skill for the Prometheus Skill Pack

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [agentskills.io: The Open Standard](#agentskillsio-the-open-standard)
3. [Claude Code (Anthropic)](#claude-code-anthropic)
4. [OpenAI Codex CLI](#openai-codex-cli)
5. [OpenCode](#opencode)
6. [Cursor](#cursor)
7. [Windsurf / Cascade](#windsurf--cascade)
8. [Kimi Code / Kimi Work](#kimi-code--kimi-work)
9. [MiniMax / Mavis CLI](#minimax--mavis-cli)
10. [Other Platforms (Gemini CLI, Roo Code, Amp, etc.)](#other-platforms)
11. [Cross-Platform Comparison Matrix](#cross-platform-comparison-matrix)
12. [Common Denominator: What Makes a Skill Truly Portable](#common-denominator-what-makes-a-skill-truly-portable)
13. [Recommendations for Prometheus Skill Pack](#recommendations-for-prometheus-skill-pack)
14. [Sources & Citations](#sources--citations)

---

## Executive Summary

The agent skills ecosystem has converged on a **single open standard** originally developed by Anthropic and released as the Agent Skills specification at [agentskills.io](https://agentskills.io). As of mid-2026, virtually every major AI coding agent platform supports this format, though each implements it with platform-specific directory conventions, discovery mechanisms, and additional proprietary extensions.

A skill, at its simplest, is a **folder containing a `SKILL.md` file** with YAML frontmatter (`name`, `description`) and Markdown instructions. The format supports progressive disclosure (metadata at startup → full instructions on activation → scripts/references on demand), making it efficient for agents to maintain large skill libraries without context window bloat.

The key strategic insight for the Prometheus Skill Pack: **the SKILL.md format is the common denominator**, but true portability requires awareness of each platform's directory conventions, invocation patterns, and any supplemental metadata files.

---

## agentskills.io: The Open Standard

### Specification Overview

The Agent Skills specification is maintained at [agentskills.io](https://agentskills.io) and on GitHub at `github.com/agentskills/agentskills`. It defines a lightweight, filesystem-based format for packaging reusable AI agent capabilities.

**Core principle:** A skill is a folder with a `SKILL.md` file. No SDK, no API, no compilation — just Markdown.

### Directory Structure

```
skill-name/
├── SKILL.md          # Required: metadata + instructions
├── scripts/          # Optional: executable code
├── references/       # Optional: documentation
├── assets/           # Optional: templates, resources
└── ...               # Any additional files or directories
```

### SKILL.md Frontmatter Fields

| Field | Required | Constraints | Purpose |
|-------|----------|-------------|---------|
| `name` | Yes | 1-64 chars, lowercase alphanumeric + hyphens, no leading/trailing/double hyphens, must match parent directory | Unique identifier |
| `description` | Yes | 1-1024 chars | Describes what the skill does AND when to use it (critical for discovery) |
| `license` | No | String or file reference | License information |
| `compatibility` | No | Max 500 chars | Environment requirements (platform, system packages, network access) |
| `metadata` | No | Arbitrary key-value mapping | Custom properties (version, author, tags, etc.) |
| `allowed-tools` | No | Space-delimited string | Pre-approved tools (experimental; platform support varies) |

**Minimal valid SKILL.md:**
```yaml
---
name: explain
description: Explain complex topics in simple terms. Use when the user asks for clarification, simplification, or an analogy.
---

# Explain

Break down the topic into plain language. Use analogies when helpful.
```

### Progressive Disclosure Architecture

The specification's key innovation is **three-tier loading** that keeps context efficient:

| Tier | When Loaded | Token Cost | Content |
|------|-------------|------------|---------|
| **Discovery** | At startup | ~100 tokens per skill | `name` and `description` only |
| **Activation** | When skill is triggered | <5000 tokens recommended | Full `SKILL.md` body |
| **Execution** | When referenced | Variable | Scripts, references, assets (on demand) |

This means an agent can carry 50+ skills for only ~5000 tokens of metadata overhead, loading full instructions only for skills actually used.

### Discovery & Installation

**Primary distribution platforms:**
- **skills.sh** — The main distribution hub; `npx skills add <owner/repo>` installs across compatible platforms
- **agentskill.sh** — Curated skill directory with security scanning and ratings
- **Agensi** — Marketplace with cross-compatibility verification
- **GitHub** — Direct repository hosting; many orgs publish official skills (e.g., `smartcontractkit/chainlink-agent-skills`, `google/skills`)

**Validation tools:**
- `skills-ref validate ./my-skill` — Official reference library for checking frontmatter and structure

### Sources
- [agentskills.io/specification](https://agentskills.io/specification) — Complete specification
- [agentskills.io/home](https://agentskills.io/home) — Overview and ecosystem
- [agentskill.sh/readme](https://agentskill.sh/readme) — FAQ and guides
- [github.com/agentskills/agentskills](https://github.com/agentskills/agentskills) — Specification source
- [github.com/microsoft/hve-core/issues/671](https://github.com/microsoft/hve-core/issues/671) — VS Code alignment with spec
- [github.com/JacobPEvans/ai-assistant-instructions/issues/425](https://github.com/JacobPEvans/ai-assistant-instructions/issues/425) — Skill alignment work

---

## Claude Code (Anthropic)

### Skill System

Claude Code was the **originator** of the Agent Skills format and remains the reference implementation. It supports both filesystem-based custom skills and an API-based skill system.

**Skill directories:**
- `~/.claude/skills/` — User-level (personal, across all projects)
- `.claude/skills/` — Project-level (shared via git with team)

### Invoking Skills

- **Automatic discovery**: Claude reads skill descriptions at session startup and activates matching skills automatically
- **Manual invocation**: `/skill-name` slash command
- **Parameter invocation**: `/skill-name param1 param2`
- **Disable auto-invocation**: `disable-model-invocation: true` in frontmatter

### Claude-Specific Features

- **Subagent context forking**: Skills can influence how Claude spawns subagents (unique to Claude Code)
- **Plan Mode**: Separates exploration from execution, letting Claude plan before acting
- **Plugin system**: `/plugin marketplace add <owner/repo>` and `/plugin install <plugin>` for third-party plugins
- **Built-in skills**: Anthropic provides pre-built skills for PowerPoint (`pptx`), Excel (`xlsx`), Word (`docx`), and PDF (`pdf`) document processing
- **Skill Creator**: Built-in `skill-creator` skill that guides users through creating new skills interactively
- **Organization skills**: Team/Enterprise admins can distribute approved skills organization-wide

### Claude API Integration

Skills in the Claude API require three beta headers:
- `code-execution-2025-08-25`
- `skills-2025-10-02`
- `files-api-2025-04-14`

```python
response = client.beta.messages.create(
    model="claude-sonnet-4-5-20250514",
    betas=["code-execution-2025-08-25", "skills-2025-10-02", "files-api-2025-04-14"],
    container={"skills": [{"type": "anthropic", "skill_id": "xlsx", "version": "latest"}]},
    # ...
)
```

Custom skills can be uploaded via `/v1/skills` endpoints and shared organization-wide.

### Plugin System (Claude Code Plugins)

Claude Code has a plugin marketplace with manifest files in `.claude-plugin/`:
- `plugin.json` — Plugin identity and bundled components
- `marketplace.json` — Marketplace metadata
- Plugins can bundle skills, MCP servers, and hooks

**Example install flow:**
```bash
/plugin marketplace add openai/codex-plugin-cc
/plugin install codex@openai-codex
/reload-plugins
```

### Sources
- [Claude Code Docs](https://docs.anthropic.com/claude-code) — Official documentation
- [github.com/agamm/claude-code-owasp](https://github.com/agamm/claude-code-owasp) — Example skill
- [Claude Code December 2025 - January 2026 Updates](https://www.vincirufus.com/en/posts/claude-code-december-january-updates/) — Plan Mode, subagents, native plugins
- [mcpmarket.com](https://mcpmarket.com/tools/skills/ftc-decode-2025-2026-reference) — Skill marketplace examples

---

## OpenAI Codex CLI

### Skill System

Codex adopted the Agent Skills open standard in late 2025. It supports the same `SKILL.md` format as Claude Code, with Codex-specific extensions.

**Skill directories (hierarchical scan):**
1. `$CWD/.agents/skills` — Current directory
2. `$REPO_ROOT/.agents/skills` — Repository root
3. `$HOME/.agents/skills` — User-level
4. `/etc/codex/skills` — System/admin level

### Activation

- **Explicit**: `/skills` command or `$` mention of skill name
- **Implicit**: Codex autonomously selects skills when tasks match descriptions
- **Configuration**: Skills can be enabled/disabled in `~/.codex/config.toml`

```toml
[[skills.config]]
path = "/path/to/skill/SKILL.md"
enabled = false
```

### Codex-Specific Extensions

- **`agents/openai.yaml`**: Optional UI metadata file per skill controlling icons, branding, and MCP tool dependencies
- **`openai.yaml`**: Declares how the skill appears in the Codex UI
- **Skills can be disabled with `disable-model-invocation: true`** (similar to Cursor)

### Codex Plugin System (Enterprise, March 2026)

OpenAI launched an enterprise plugin system in March 2026 that allows organizations to package workflows, app integrations, and MCP server configurations into installable bundles.

**Plugin structure:**
```
my-plugin/
├── .codex-plugin/
│   └── plugin.json          # Required: manifest
├── skills/                  # Bundled skills
│   └── my-skill/
│       └── SKILL.md
├── .app.json                # App/connector mappings
├── .mcp.json                # MCP server config
├── hooks/                   # Lifecycle hooks
│   └── hooks.json
└── assets/                  # Icons, logos
```

**Plugin manifest (`plugin.json`) fields:**
- `name`, `version`, `description` — Identity
- `author`, `homepage`, `repository`, `license`, `keywords` — Publisher metadata
- `skills`, `mcpServers`, `apps`, `hooks` — Pointers to bundled components
- `interface` — Install-surface metadata (displayName, shortDescription, category, brandColor, icons, screenshots)

**Governance layer:**
- Organizations define plugin catalogs (`marketplace.json`) with installation policies: `INSTALLED_BY_DEFAULT`, `AVAILABLE`, `NOT_AVAILABLE`
- Authentication behavior configurable at policy level

**Installation:**
```bash
codex plugin marketplace add <owner/repo>
codex plugin install <plugin>
```

**Built-in `$plugin-creator` skill**: Scaffolds the required `.codex-plugin/plugin.json` and generates marketplace entries.

**Cross-platform note:** OpenAI shipped an official `codex-plugin-cc` plugin that embeds Codex inside Claude Code, demonstrating the convergence of plugin ecosystems.

### Sources
- [OpenAI Codex Plugin Docs](https://developers.openai.com/codex/plugins/build) — Build plugins
- [Azalio: OpenAI adds plugin system to Codex](https://www.azalio.io/openai-adds-plugin-system-to-codex-to-help-enterprises-govern-ai-coding-agents/) — Enterprise plugin announcement
- [InfoWorld: OpenAI adds plugin system](https://www.infoworld.com/article/4151214/openai-adds-plugin-system-to-codex-to-help-enterprises-govern-ai-coding-agents.html) — Enterprise governance
- [Smart Scope Blog](https://smartscope.blog/en/blog/codex-plugin-cc-openai-claude-code-2026/) — codex-plugin-cc analysis
- [github.com/openai/codex-plugin-cc](https://github.com/openai/codex-plugin-cc) — Official Claude Code plugin
- [github.com/daloopa/daloopa-plugin-codex](https://github.com/daloopa/daloopa-plugin-codex) — Example Codex plugin
- [github.com/tim-osterhus/codex-remotion-plugin](https://github.com/tim-osterhus/codex-remotion-plugin) — Example plugin structure

---

## OpenCode

### Plugin Architecture (NOT Skill-Based)

**Critical distinction:** OpenCode does NOT use the `SKILL.md` Agent Skills format. Instead, it has a **JavaScript/TypeScript plugin system** based on `@opencode-ai/plugin` and `@opencode-ai/sdk`.

### Plugin Structure

An OpenCode plugin is a **JS/TS module** that exports an async function receiving a context object and returning lifecycle hooks:

```typescript
import type { Plugin } from "@opencode-ai/plugin"

export const MyPlugin: Plugin = async ({ project, client, $, directory, worktree }) => {
  return {
    // Hook implementations
    "tool.execute.before": async (input, output) => { /* ... */ },
    "session.created": async (input, output) => { /* ... */ },
  }
}
```

### Plugin Loading

**Two ways to load plugins:**
1. **Local files**: Place JS/TS files in:
   - `.opencode/plugins/` — Project-level
   - `~/.config/opencode/plugins/` — Global
2. **npm packages**: Specify in `opencode.json`:
   ```json
   {
     "$schema": "https://opencode.ai/config.json",
     "plugin": ["opencode-helicone-session", "@my-org/custom-plugin"]
   }
   ```

**Loading order:**
1. Global config (`~/.config/opencode/opencode.json`)
2. Project config (`opencode.json`)
3. Global plugin directory (`~/.config/opencode/plugins/`)
4. Project plugin directory (`.opencode/plugins/`)

### Plugin Capabilities

OpenCode plugins are **far more powerful** than SKILL.md skills:

| Capability | SKILL.md Skill | OpenCode Plugin |
|------------|--------------|-----------------|
| Runs | N/A (passive instructions) | Inside OpenCode process |
| Communication | N/A | Direct function calls |
| Can add tools | No | Yes (AI-callable tools) |
| Can hook events | No | Yes (lifecycle hooks) |
| Can modify behavior | No | Yes (params, headers, env) |
| Can use npm deps | No | Yes (via package.json) |
| Cross-platform | Yes (portable) | OpenCode only |

**Available hooks:**
- Command: `command.executed`
- File: `file.edited`, `file.watcher.updated`
- Installation: `installation.updated`
- LSP: `lsp.client.diagnostics`, `lsp.updated`
- Message: `message.*` (part.removed, part.updated, removed, updated)
- Permission: `permission.asked`, `permission.replied`
- Server: `server.connected`
- Session: `session.*` (created, compacted, deleted, diff, error, idle, status, updated)
- Tool: `tool.execute.before`, `tool.execute.after`
- TUI: `tui.prompt.append`, `tui.command.execute`, `tui.toast.show`

**Custom tools example:**
```typescript
import { type Plugin, tool } from "@opencode-ai/plugin"

export const CustomToolsPlugin: Plugin = async (ctx) => {
  return {
    tool: {
      mytool: tool({
        description: "This is a custom tool",
        args: { foo: tool.schema.string() },
        async execute(args, context) {
          return `Hello ${args.foo} from ${context.directory}`
        },
      }),
    },
  }
}
```

### OpenCode SDK vs Plugin

| Aspect | `@opencode-ai/plugin` | `@opencode-ai/sdk` |
|--------|----------------------|-------------------|
| Runs | Inside OpenCode process | External process |
| Communication | Direct function calls | HTTP/SSE/WebSocket |
| Can add tools | Yes | No |
| Can hook events | Yes | No |
| Can modify behavior | Yes | No |
| Use case | Extend OpenCode | Automate OpenCode |

### OpenCode + Agent Skills (External Compatibility)

OpenCode itself does NOT natively load `SKILL.md` files. However:
- The **OpenCode server** exposes an HTTP API (Hono backend, Bun runtime)
- Third-party tools can bridge OpenCode with SKILL.md skills via the SDK
- The VTEX skills project, for example, exports to OpenCode via `opencode-skills.tar.gz` that creates `SKILL.md` directories in `~/.config/opencode/skills/`

### Sources
- [opencode.ai/docs/plugins/](https://opencode.ai/docs/plugins/) — Official plugin docs
- [github.com/awesome-opencode/awesome-opencode](https://github.com/awesome-opencode/awesome-opencode) — Plugin ecosystem
- [cefboud.com: How Coding Agents Actually Work](https://cefboud.com/posts/coding-agents-internals-opencode-deepdive/) — Architecture deep dive
- [github.com/agnusdei1207/opencode-orchestrator](https://github.com/agnusdei1207/opencode-orchestrator) — Orchestrator plugin example
- [github.com/pai4451/opencode-telemetry-plugin](https://github.com/pai4451/opencode-telemetry-plugin) — Telemetry plugin example
- [lobehub.com skills](https://lobehub.com/skills/fkxxyz-cclover-skills-opencode-plugin-development) — OpenCode plugin development skill

---

## Cursor

### Dual System: `.cursorrules` vs SKILL.md

Cursor has **two distinct customization systems**:

| Feature | `.cursorrules` | `SKILL.md` Skills |
|---------|---------------|-------------------|
| Format | Plain text file | YAML frontmatter + Markdown |
| Location | Project root | `.cursor/skills/` directory |
| Activation | Always loaded | On-demand by description match |
| Scope | Entire project | Specific tasks |
| Multiple files | One per project | Unlimited per project |
| Cross-agent | Cursor only | 20+ agents (Claude Code, Codex, OpenCode, etc.) |
| Marketplace | None | Agensi, GitHub |
| Supporting files | None | Scripts, references, assets |

**Best practice:** Use `.cursorrules` for always-on project context (coding conventions, tech stack, style preferences). Use `SKILL.md` skills for on-demand workflows (code review, test generation, deployment).

### Cursor Rules (.cursorrules / .cursor/rules/)

- **Legacy**: `.cursorrules` file in project root (plain text, always loaded)
- **Modern**: `.cursor/rules/` directory with `.mdc` rule files
  - Rules can have `alwaysApply: true/false` and `globs` patterns for file-specific activation
  - Three activation modes: **Always**, **Glob** (file pattern), **Manual/Description-based**
- **Migration tool**: `/migrate-to-skills` built-in command (Cursor 2.4+) converts dynamic rules to SKILL.md skills

### SKILL.md Skills in Cursor

**Skill directories:**
- `.cursor/skills/` — Project-level
- `~/.cursor/skills/` — User-level
- `.claude/skills/` and `.codex/skills/` — Cross-platform compatibility (Cursor also scans these)

**Invocation:**
- **Automatic**: Cursor detects and applies based on context
- **Manual**: `/skill-name` in Agent chat
- **Special field**: `disable-model-invocation: true` — makes skill slash-command only

**Cursor 2.4+ includes a `/migrate-to-skills` skill** that converts existing dynamic rules and slash commands to standard SKILL.md format.

### Sources
- [agensi.io: Cursor Rules vs SKILL.md](https://www.agensi.io/learn/cursor-rules-vs-skill-md-complete-guide) — Complete comparison
- [promptspace.in: Cursor AI Skills Marketplace](https://www.promptspace.in/blog/cursor-ai-skills-marketplace-skill-md) — Using SKILL.md with Cursor
- [github.com/hutchic/.cursor](https://github.com/hutchic/.cursor/blob/main/docs/cursor-skills.md) — Migration guide
- [github.com/Mindrally/skills](https://github.com/Mindrally/skills) — 240+ Claude Code skills converted from Cursor rules

---

## Windsurf / Cascade

### Skill System

Windsurf (acquired by Cognition AI in December 2025) added Agent Skills support in **January 2026** (Wave 13 release). The system follows the agentskills.io standard.

**Skill directories:**
- `.windsurf/skills/` — Workspace-level (project-specific)
- Global skills via Customizations panel in Cascade

**Skill structure:**
```
.windsurf/skills/deploy-to-production/
├── SKILL.md
├── deployment-checklist.md
├── rollback-procedure.md
└── config-template.yaml
```

**SKILL.md format:** Same as agentskills.io standard with `name` and `description` frontmatter. The `name` field is used for display and @-mentions. The `description` helps Cascade decide when to automatically invoke the skill.

**Invocation:**
- **Automatic**: Cascade uses progressive disclosure (description matching)
- **Manual**: @-mention the skill name in prompts

### Cascade Features Beyond Skills

- **Cascade modes**: Write Mode (full write access) vs Chat Mode (read-only)
- **Workflows**: Saved as markdown files in `.windsurf/workflows/`, invoked via `/workflow-name` slash commands
- **Memories**: Auto-generated and user-created persistent context across sessions
- **Rules**: `global_rules.md` (cross-workspace), `.windsurf/rules/` (workspace-level), with Always/Glob/Manual activation modes
- **MCP Integration**: Native MCP support with marketplace, @-mention triggering, stdio/HTTP/SSE transports
- **Cascade Hooks**: Execute custom shell commands at key workflow points (Enterprise)
- **Simultaneous Cascades**: Multiple parallel Cascade sessions with Git worktrees
- **Fast Context**: `Cmd+Enter` / `Ctrl+Enter` for 20x faster code retrieval
- **Skills + Workflows + Memories + MCP** = full agentic ecosystem within the IDE

### Key Differences from Cursor

| Feature | Windsurf | Cursor |
|---------|----------|--------|
| AI Agent | Cascade (agentic, flow-aware) | Cursor Agent (CLI-based) |
| Rules | `.windsurf/rules/` + `global_rules.md` | `.cursor/rules` + `CLAUDE.md` |
| Workflows | `.windsurf/workflows/` (slash commands) | N/A |
| Memories | Auto-generated + user-created | Codebase indexing + Project Rules |
| Skills | `.windsurf/skills/` (bundled folders) | `.cursor/skills/` |
| Terminal | Dedicated zsh shell + Turbo Mode | Standard terminal |
| Live Preview | Built-in | Extension-based |
| MCP | Native with marketplace | Native |

### Sources
- [byteiota.com: Windsurf Cascade Tutorial](https://byteiota.com/windsurf-cascade-tutorial-agentic-ai-coding-in-10-minutes/) — Agentic AI tutorial
- [taskade.com: Windsurf Review 2026](https://www.taskade.com/blog/windsurf-review) — Features and pricing
- [playbooks.com: windsurf-cascade skill](https://playbooks.com/skills/openclaw/skills/windsurf-cascade) — Complete skill reference
- [codevelocity.academy: Claude Code vs Windsurf](https://www.codevelocity.academy/en/compare/claude-code-vs-windsurf) — Feature comparison

---

## Kimi Code / Kimi Work

### Skill System

Kimi Code (Moonshot AI's terminal-based coding agent, launched June 2026, powered by K2.7 Code) supports the Agent Skills open standard.

**Skill directories:**
- `.kimi-code/skills/` — Project-level
- `.agents/skills/` — Cross-agent compatibility (also read by Kimi Code)
- `.skills/` — Alternative project-level path

**Important:** Kimi Code does NOT read `.claude/skills/`. This is by design — the open standard specifies the format but not the directory path, and each platform chooses its own discovery locations.

**Discovery:**
- Skills are detected automatically at startup based on directory scanning
- The `name` and `description` frontmatter are used for matching
- Kimi Code uses the same progressive disclosure model as other platforms

### Kimi Work (Desktop Agent)

Kimi Work is Moonshot AI's general-purpose desktop agent for knowledge workers, built on the same Kimi Code model but with additional capabilities:

- **Multi-agent clusters**: Up to 300 sub-agents working in parallel
- **Long-horizon execution**: Supports 13-hour continuous coding, 4000+ autonomous tool calls
- **Skill mechanism**: Users can create custom skills by delegating research tasks to multiple agents
- **WebBridge**: AI-controlled browser for information collection
- **Built-in office skills**: Document processing, PPT generation, stock analysis
- **Hooks system**: `UserPromptSubmit` hook with `inject_prompt` for automatically injecting skill instructions

**Kimi Work skill creation example:**
1. Deploy 5 sub-agents to research different investment philosophies (Charlie Munger, Peter Lynch, etc.)
2. Each agent completes independent research and analysis
3. Results are synthesized into a reusable skill
4. Future use: input a specific stock/fund, and the skill auto-generates multi-dimensional investment analysis

### Sources
- [agensi.io: Kimi Code Skills Guide](https://www.agensi.io/learn/kimi-code-skills-guide) — Getting started
- [h89.cn: 你的 Claude/Codex/opencode/Kimi 为什么互相找不到SKILL](https://h89.cn/archives/632.html) — Directory path differences (Chinese)
- [github.com/MoonshotAI/kimi-cli](https://github.com/MoonshotAI/kimi-cli) — Official CLI repo
- [github.com/Dqz00116/kimi-with-superpowers](https://github.com/Dqz00116/kimi-with-superpowers) — Superpowers workflow for Kimi
- [jimo.studio: Kimi Work来了](https://jimo.studio/blog/kimi-work-launches-build-an-investment-master-skill-in-10-minutes-ushering-in-the-vibe-working-era/) — Kimi Work skill creation (Chinese)
- [github.com/MoonshotAI/kimi-cli/issues/2071](https://github.com/MoonshotAI/kimi-cli/issues/2071) — Feature request for mandatory skill loading gates

---

## MiniMax / Mavis CLI

### Product Ecosystem

MiniMax's agent ecosystem is complex and has undergone significant rebranding:

| Name | What it is | Best for |
|------|-----------|----------|
| **MiniMax Agent** | General-purpose agent for long-horizon tasks | Research, coding, PPTs, apps, document work |
| **Mavis** | Upgraded MiniMax Agent (May 2026) | "MiniMax as a Jarvis" — AI butler-style agent |
| **MiniMax Code** | Renamed desktop app (v3.0.33+) | Local projects, coding workflows, code review |
| **Mini-Agent** | Open-source demo project | Developers learning to build agents with MiniMax M3 |
| **MiniMax M3/M2.7/M2** | Foundation models | Coding, agentic workflows, multimodal, long context |

### Skill System

MiniMax Agent/Mavis supports **Agent Skills** but with a different architecture than the pure SKILL.md model:

- **Built-in five-step Deep Research skill**: A structured research workflow included in the June 2026 changelog
- **Agent Teams**: Multi-agent system with Leader, Worker, and Verifier roles
  - Leader structures goals, Workers execute specialized tasks, Verifiers check deliverability
  - Tasks split into parallel sub-tasks with adversarial quality gates
- **Claude Skills Integration**: The open-source Mini-Agent project ships with 15 professional skills for documents, design, testing, and development, following Anthropic's format
- **MCP Tool Integration**: Native MCP support for knowledge graphs, web search, etc.
- **Agent Skills format**: MiniMax uses a hybrid approach — skills can be packaged as instructions but also integrated with their multi-agent orchestration system

**Mavis CLI (MiniMax Code) skill system:**
- Localized skill search in the desktop app
- Custom MCPs and pre-built integrations (MiniMax MCP, Google Maps, GitHub/GitLab, Slack, Figma)
- Skills are searchable and installable within the MiniMax Code interface
- Agent Teams allow role-specific skills (e.g., a "coder" agent with coding skills, a "verifier" agent with review skills)

### Sources
- [minimax-ai.chat: Complete Guide](https://minimax-ai.chat/models/minimax-agent/) — Features, pricing, Mavis
- [github.com/MiniMax-AI/Mini-Agent](https://github.com/MiniMax-AI/Mini-Agent) — Open-source Mini Agent
- [geeky-gadgets.com: Automate Coding with Mavis Agent](https://www.geeky-gadgets.com/multi-agent-coding-verification/) — Multi-agent system
- [post.smzdm.com: MiniMax发布Mavis](https://post.smzdm.com/p/ae605oxm) — Mavis announcement (Chinese)

---

## Other Platforms

### Gemini CLI / Google Antigravity

Google adopted the Agent Skills open standard:

- **Gemini CLI** (sunset June 18, 2026 for individuals) → **Antigravity CLI** (successor)
- **Skill directories**: `.gemini/skills/`, `.agents/skills/`, `~/.gemini/skills/`, `~/.agents/skills/`
- **Precedence**: Workspace skills > User skills > Extension skills; `.agents/skills/` takes precedence over `.gemini/skills/` at the same tier
- **Commands**: `/skills list`, `/skills link`, `/skills disable`, `/skills enable`, `/skills reload`
- **Official catalog**: `github.com/google-gemini/gemini-skills` (3 skills: gemini-api-dev, gemini-live-api-dev, gemini-interactions-api)
- **Google Cloud skills**: `github.com/google/skills` (13 skills announced at Cloud Next 2026)
- **Published benchmarks**: Gemini 3 Flash — 87% correct API code with skill; Gemini 3 Pro — 96% correct API code with skill

### Roo Code

Roo Code (formerly Roo Cline, a Cline fork) supports Agent Skills with mode-specific targeting:

- **Skill directories**: `.roo/skills/`, `~/.roo/skills/` (Roo-specific); `.agents/skills/`, `~/.agents/skills/` (cross-agent)
- **Mode-specific skills**: `.roo/skills-code/`, `.roo/skills-architect/`, etc. — skills only activate in specific modes
- **Override priority**: Project > Global; `.roo/` > `.agents/`; Mode-specific > Generic
- **Symlink support**: For sharing skill libraries across projects
- **Discovery**: Automatic at startup; file watchers detect changes; mode filtering available
- **Roo Commander**: Third-party tool that bridges Claude Code skills to Roo Code via `.roomodes`

### Amp, Factory, Trae, Firebender, Databricks, Spring AI, Letta, Agentman, Autohand, Mux, Command Code, Qodo, Ona, VT Code, Mistral Vibe, Hermes Agent, and others

All these platforms are documented as supporting the Agent Skills format at varying levels of maturity. The ecosystem is converging rapidly on the `SKILL.md` standard.

### Sources
- [geminicli.com/docs/cli/skills](https://geminicli.com/docs/cli/skills/) — Gemini CLI skills docs
- [rywalker.com: Google Gemini Skills](https://rywalker.com/research/google-gemini-skills) — Research and benchmarks
- [roocodeinc.github.io: Roo Code Skills](https://roocodeinc.github.io/Roo-Code/features/skills/) — Roo Code skill docs
- [github.com/Kastalien-Research/rooskills](https://github.com/Kastalien-Research/rooskills) — Roo Skills bridge tool
- [agentskill.sh/readme](https://agentskill.sh/readme) — Supported platforms list

---

## Cross-Platform Comparison Matrix

| Platform | Skill Format | Directory | Auto-Discovery | Manual Invocation | Plugin System | Cross-Platform Skills | MCP Support |
|----------|-----------|-----------|----------------|-------------------|---------------|----------------------|-------------|
| **Claude Code** | SKILL.md | `~/.claude/skills/`, `.claude/skills/` | Yes | `/skill-name` | Yes (`.claude-plugin/`) | Yes | Native |
| **Codex CLI** | SKILL.md + `agents/openai.yaml` | `~/.agents/skills/`, `.agents/skills/` | Yes | `/skills`, `$mention` | Yes (`.codex-plugin/` with `plugin.json`, `.mcp.json`, hooks) | Yes | Native (via plugins) |
| **OpenCode** | JS/TS plugins (`@opencode-ai/plugin`) | `.opencode/plugins/`, `~/.config/opencode/plugins/` | Yes (file scan) | N/A (hooks fire automatically) | Yes (npm + local JS/TS) | No (platform-specific) | Native |
| **Cursor** | `.cursorrules` + SKILL.md | `.cursor/skills/`, `~/.cursor/skills/` | Yes | `/skill-name` | Yes (`.cursor-plugin/`) | Yes (SKILL.md) | Native |
| **Windsurf** | SKILL.md | `.windsurf/skills/` | Yes | @-mention | Yes | Yes | Native (marketplace) |
| **Kimi Code** | SKILL.md | `.kimi-code/skills/`, `.agents/skills/`, `.skills/` | Yes | Automatic | No | Yes | Partial |
| **Kimi Work** | SKILL.md | Desktop skill panel | Yes | Slash commands | No | Yes | WebBridge (built-in browser) |
| **Gemini CLI** | SKILL.md | `.gemini/skills/`, `.agents/skills/` | Yes | `/skills` command | Extensions | Yes | Via extensions |
| **Antigravity** | SKILL.md | `.agents/skills/`, `~/.agents/skills/` | Yes | Automatic | Yes | Yes | Native |
| **Roo Code** | SKILL.md | `.roo/skills/`, `.agents/skills/` | Yes | Automatic | Yes | Yes | Native |
| **MiniMax/Mavis** | Hybrid (skills + multi-agent) | Desktop app | Yes | UI-based | No | Partial | Native |
| **VS Code** | SKILL.md | `.github/skills/` | Via Copilot | Via chat | Yes | Yes | Via extensions |
| **GitHub Copilot** | SKILL.md | `.github/skills/` | Yes | Chat | Yes | Yes | Via extensions |

---

## Common Denominator: What Makes a Skill Truly Portable

### The Universal Layer: SKILL.md

Every platform that claims Agent Skills compatibility supports at minimum:
1. **A directory named after the skill** (lowercase, hyphens, 1-64 chars)
2. **A `SKILL.md` file** inside that directory
3. **YAML frontmatter** with at least `name` and `description`
4. **Markdown body** with instructions
5. **Progressive disclosure** — metadata loaded at startup, full body on activation

### The Portability Checklist

To make a skill work across **all** platforms, follow these rules:

| Rule | Rationale |
|------|-----------|
| Use `name` matching directory exactly | All platforms enforce this |
| Keep `description` under 1024 chars | Hard limit in spec |
| Write description in third person | It's injected into system prompt; "I can help" breaks discovery |
| Include both "what" and "when" in description | Helps agents match tasks correctly |
| Keep SKILL.md body under 500 lines / <5000 tokens | Recommended for all platforms |
| Use `scripts/` for deterministic code | Source doesn't enter context; only output does |
| Use `references/` for detailed docs | Loaded on demand |
| Use `assets/` for templates | Static resources |
| Use relative paths one level deep | Avoid nested reference chains |
| Put `version` in `metadata` map, not top-level | Some implementations accept top-level `version` but `metadata` is the portable way |
| Avoid platform-specific tool names unless necessary | Use generic instructions where possible |

### Platform-Specific Adaptations Required

While the `SKILL.md` file itself is portable, **deployment requires platform awareness:**

| Platform | Deployment Path | Special Notes |
|----------|----------------|---------------|
| Claude Code | `~/.claude/skills/` or `.claude/skills/` | Also supports API upload for Claude.ai |
| Codex | `~/.agents/skills/` or `.agents/skills/` | Add `agents/openai.yaml` for UI polish |
| Cursor | `.cursor/skills/` | Also scans `.claude/skills/` and `.codex/skills/` for cross-compatibility |
| Windsurf | `.windsurf/skills/` | Access via Customizations panel |
| Kimi Code | `.kimi-code/skills/` or `.agents/skills/` or `.skills/` | Does NOT read `.claude/skills/` |
| Gemini/Antigravity | `.gemini/skills/` or `.agents/skills/` | `.agents/skills/` preferred for portability |
| Roo Code | `.roo/skills/` or `.agents/skills/` | Supports mode-specific `skills-{mode}/` |
| VS Code / Copilot | `.github/skills/` | Enterprise/org-level distribution |

### The "Multi-Platform Plugin" Pattern

Sophisticated projects (e.g., [contentforge](https://github.com/indranilbanerjee/contentforge), [pencil-skill](https://github.com/Nisus74/pencil-skill)) now ship multiple plugin manifests:

```
my-project/
├── skills/
│   └── my-skill/
│       └── SKILL.md          # Universal skill core
├── .claude-plugin/
│   └── plugin.json             # Claude Code manifest
├── .codex-plugin/
│   └── plugin.json             # Codex manifest
├── .cursor-plugin/
│   └── plugin.json             # Cursor manifest
├── .github/plugin/
│   └── plugin.json             # Copilot manifest
├── gemini-extension.json       # Gemini/Antigravity
├── openclaw.plugin.json        # OpenClaw
├── plugin.yaml                 # Hermes Agent
└── .mcp.json                   # MCP server config (shared)
```

This is the **maximum portability** approach — one core skill, packaged for every platform's plugin system.

### What Skills CANNOT Do (Common Limitations)

| Limitation | Explanation |
|------------|-------------|
| Skills don't call APIs directly | They can instruct agents to use MCP tools or run scripts that call APIs |
| Skills aren't versioned by the spec | Use `metadata.version` or git |
| Skills can't conflict-resolve automatically | Agents use description matching; overlapping descriptions may cause issues |
| Skills don't persist state across sessions | Stateless by design; use MCP servers or external storage for persistence |
| Skills aren't executable tools | They are instructions; tools are added via MCP or platform-specific plugin systems |

---

## Recommendations for Prometheus Skill Pack

### 1. Core Skill Design

For a universal "deep-research" skill, design around the **Agent Skills specification** as the primary format:

```
deep-research/
├── SKILL.md
├── scripts/
│   ├── search-orchestrator.py
│   ├── evidence-collector.py
│   └── citation-manager.py
├── references/
│   ├── research-methodologies.md
│   ├── source-evaluation.md
│   └── output-formats.md
└── assets/
    ├── report-template.md
    └── knowledge-asset-schema.json
```

### 2. Multi-Platform Distribution Strategy

Ship the core skill as a GitHub repository with:
- **Primary**: `SKILL.md` in standard `deep-research/` directory
- **Installation**: `npx skills add prometheus/deep-research` via skills.sh
- **For Claude Code**: `.claude-plugin/plugin.json` + `marketplace.json`
- **For Codex**: `.codex-plugin/plugin.json` + `agents/openai.yaml` per skill
- **For Cursor**: `.cursor-plugin/plugin.json`
- **For Copilot**: `.github/skills/deep-research/SKILL.md`
- **For Roo Code**: `.roo/skills/deep-research/SKILL.md`
- **For OpenCode**: Since OpenCode doesn't natively read SKILL.md, provide a bridge plugin OR document using the skill via the OpenCode SDK

### 3. MCP Integration Strategy

The deep-research skill should reference existing Prometheus MCP servers:
- `tavily-mcp` — Web search
- `sequential-thinking` — Reasoning chains
- `surreal-memory` — Persistent knowledge storage
- `prometheus-knowledge` — Knowledge graph queries
- `forge-rs` — Tool execution

In the skill instructions, use fully qualified MCP tool names: `tavily-mcp:search`, `sequential-thinking:think`, etc.

### 4. Progressive Disclosure for Deep Research

Structure the skill to leverage progressive disclosure:
- **Discovery level**: `name: deep-research`, `description: Conducts exhaustive multi-source research, evidence synthesis, and structured report generation. Use when the user needs comprehensive investigation, literature review, competitive analysis, or fact-checking on any topic.`
- **Activation level**: Core workflow in `SKILL.md` (planner → decomposer → search → collect → verify → resolve → synthesize → generate)
- **Execution level**: Detailed methodologies in `references/`, deterministic scripts in `scripts/`

### 5. Knowledge Asset Output

Design the skill to emit **persistent knowledge objects** that other agents can query:
- `citations.json` — Structured citation database
- `knowledge_graph.json` — Entity-relationship graph
- `evidence.json` — Evaluated evidence with confidence scores
- `contradictions.json` — Detected conflicts and resolutions
- `source_cache/` — Cached raw sources
- `search_trace.json` — Search query history
- `reasoning_trace.md` — Reasoning process documentation

These should be saved to a predictable location (e.g., `.prometheus/research/<topic>/`) so other skills and agents can discover and extend them.

### 6. CLI Tooling

Following the Prometheus Skill Pack pattern, provide:
- `prometheus-research` CLI — Standalone research runner
- `prometheus-research init` — Initialize research project
- `prometheus-research query <topic>` — Execute research
- `prometheus-research resume <session-id>` — Resume interrupted research
- `prometheus-research export <format>` — Export to markdown, docx, pdf, etc.

### 7. AG-UI / A2UI Integration

For the web frontend, expose research progress via:
- **AG-UI protocol** — Real-time research progress streaming
- **MCP app** — Research as an MCP tool that can be invoked from any MCP-compatible client
- **assistant-ui** — React components for research visualization

---

## Sources & Citations

| # | Source | URL | Date |
|---|--------|-----|------|
| 1 | Agent Skills Specification | https://agentskills.io/specification | 2026 |
| 2 | Agent Skills Overview | https://agentskills.io/home | 2026 |
| 3 | Agent Skills Guide (agentskill.sh) | https://agentskill.sh/readme | 2025-08 |
| 4 | Agent Skills Explained (dev.to) | https://dev.to/loc_carrre_0d798813c662/agent-skills-explained | 2026-02-07 |
| 5 | Agent Skills Overview (inference.sh) | https://inference.sh/blog/skills/agent-skills-overview | 2026-04-13 |
| 6 | Agent Skills 101 (serghei.pl) | https://blog.serghei.pl/posts/agent-skills-101/ | 2026-02-19 |
| 7 | Agent Skills Guide 2026 (termdock.com) | https://www.termdock.com/en/blog/agent-skills-guide | 2026-03-16 |
| 8 | AI SDK Cookbook: Agent Skills | https://ai-sdk.dev/cookbook/guides/agent-skills | 2026 |
| 9 | OpenAI Codex Plugin Docs | https://developers.openai.com/codex/plugins/build | 2026 |
| 10 | OpenAI Codex Plugin Enterprise (azalio) | https://www.azalio.io/openai-adds-plugin-system-to-codex | 2026-03-27 |
| 11 | OpenAI Codex Plugin (infoworld) | https://www.infoworld.com/article/4151214/openai-adds-plugin-system-to-codex | 2026-03-27 |
| 12 | OpenCode Plugin Docs | https://opencode.ai/docs/plugins/ | 2026-07-03 |
| 13 | OpenCode Architecture Deep Dive | https://cefboud.com/posts/coding-agents-internals-opencode-deepdive/ | 2025-09-14 |
| 14 | Awesome OpenCode | https://github.com/awesome-opencode/awesome-opencode | 2026 |
| 15 | Cursor Rules vs SKILL.md (agensi) | https://www.agensi.io/learn/cursor-rules-vs-skill-md-complete-guide | 2026-04-23 |
| 16 | Cursor AI Skills Marketplace | https://www.promptspace.in/blog/cursor-ai-skills-marketplace-skill-md | 2026-04-12 |
| 17 | Claude Code vs Cursor vs Codex (agensi) | https://www.agensi.io/learn/claude-code-skills-vs-cursor-rules-vs-codex-skills | 2026-03-17 |
| 18 | Windsurf Cascade Tutorial | https://byteiota.com/windsurf-cascade-tutorial-agentic-ai-coding-in-10-minutes/ | 2026-01-23 |
| 19 | Windsurf Review 2026 | https://www.taskade.com/blog/windsurf-review | 2026-04-10 |
| 20 | Windsurf Cascade Skill (openclaw) | https://playbooks.com/skills/openclaw/skills/windsurf-cascade | 2026-02-18 |
| 21 | Kimi Code Skills Guide (agensi) | https://www.agensi.io/learn/kimi-code-skills-guide | 2026-06-25 |
| 22 | Kimi Code skill directory issue | https://h89.cn/archives/632.html | 2026-06-22 |
| 23 | Kimi Work Skill Creation | https://jimo.studio/blog/kimi-work-launches-build-an-investment-master-skill-in-10-minutes | 2026-06-08 |
| 24 | MiniMax Agent Complete Guide | https://minimax-ai.chat/models/minimax-agent/ | 2026-06-15 |
| 25 | MiniMax Mini-Agent (GitHub) | https://github.com/MiniMax-AI/Mini-Agent | 2025-10-31 |
| 26 | Mavis Agent Multi-Agent System | https://www.geeky-gadgets.com/multi-agent-coding-verification/ | 2026-05-18 |
| 27 | Gemini CLI Skills Docs | https://geminicli.com/docs/cli/skills/ | 2026-04-30 |
| 28 | Google Gemini Skills Research | https://rywalker.com/research/google-gemini-skills | 2026-02-22 |
| 29 | Roo Code Skills Docs | https://roocodeinc.github.io/Roo-Code/features/skills/ | 2026-05-15 |
| 30 | Roo Skills (GitHub) | https://github.com/Kastalien-Research/rooskills | 2026 |
| 31 | ContentForge Multi-Platform Plugin | https://github.com/indranilbanerjee/contentforge | 2026-06-28 |
| 32 | Pencil Skill Multi-Platform | https://github.com/Nisus74/pencil-skill | 2026-05-03 |
| 33 | Claude Code Dec 2025 - Jan 2026 | https://www.vincirufus.com/en/posts/claude-code-december-january-updates/ | 2026-01-09 |
| 34 | Claude Code Plugin Cross-Platform | https://github.com/openai/codex-plugin-cc | 2026-03-30 |
| 35 | A2A AgentSkill Metadata Proposal | https://github.com/a2aproject/A2A/issues/1395 | 2026-01-21 |
| 36 | VS Code Skill Frontmatter Alignment | https://github.com/microsoft/hve-core/issues/671 | 2026-02-19 |
| 37 | Chainlink Agent Skills | https://github.com/smartcontractkit/chainlink-agent-skills | 2026-02-11 |
| 38 | Panel Debate Skill | https://github.com/wyattowalsh/panel-debate-skill | 2026-01-29 |
| 39 | SkillCheck Free | https://github.com/olgasafonova/skillcheck-free | 2026-01-19 |
| 40 | Codex Remotion Plugin | https://github.com/tim-osterhus/codex-remotion-plugin | 2026-04-01 |
| 41 | Codex Plugin Marketplace | https://www.codex-marketplace.com/docs | 2026 |
| 42 | Daloopa Codex Plugin | https://github.com/daloopa/daloopa-plugin-codex | 2026-05-29 |
| 43 | OpenCode Orchestrator Plugin | https://github.com/agnusdei1207/opencode-orchestrator | 2026-01-13 |
| 44 | OpenCode Telemetry Plugin | https://github.com/pai4451/opencode-telemetry-plugin | 2026 |
| 45 | Kimi CLI Require-Skills Feature | https://github.com/MoonshotAI/kimi-cli/issues/2071 | 2026-04-25 |
| 46 | Kimi with Superpowers | https://github.com/Dqz00116/kimi-with-superpowers | 2026-03-31 |
| 47 | Gemini CLI Changelog | https://github.com/google-gemini/gemini-cli/blob/main/docs/changelogs/index.md | 2026-06-03 |
| 48 | Roo Code Skills (Zhihu) | https://zhuanlan.zhihu.com/p/2001240807777268532 | 2026-02-01 |
| 49 | VTEX Skills Multi-Platform Export | https://github.com/vtex/skills | 2026-03-16 |
| 50 | Agent Skills Guide (smartcity.team) | https://www.smartcity.team/consultingskills/tools/什么是agent-skills/ | 2026-01-21 |

---

*Report compiled on 2026-07-03. All URLs and data points verified against current web sources as of this date.*
