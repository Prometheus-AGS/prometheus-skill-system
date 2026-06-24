# 16 · CLI & Scripts Reference

This page is the exhaustive index of every command-line surface the pack exposes: the binary CLIs (cross-referenced), the npm scripts, every installer and validator in `scripts/`, the runtime hook scripts in `shared/scripts/`, and the scheduled jobs. If you are looking for "what command does X," it is here.

## The binary CLIs

Documented in full on the [Tools Reference](13-tools-reference.md) page; summarized here for completeness.

| Binary | Role | Documented in |
|---|---|---|
| `prometheus` | Skill management, self-learning, GitOps validation, Cedar policy, sycophancy | [13](13-tools-reference.md) |
| `forge` | Enrichment, reflection, drift, templates, MCP server | [13](13-tools-reference.md) |
| `pk` / `pk-cherry` | Karpathy KB CLI / MCP bridge | [13](13-tools-reference.md) |
| `liter-llm` | LLM proxy + MCP tool server | [13](13-tools-reference.md) |
| `surreal-memory-server` | Graph memory + MCP + REST | [13](13-tools-reference.md) |
| `prometheus-rust-auditor` | Staged Rust quality pipeline | [13](13-tools-reference.md) |

## The npm script surface

`package.json` (v1.2.0, ES module) is the most common entry point.

| Command | What it does |
|---|---|
| `npm run validate` | Validate native skills (excludes submodules), 0 errors required |
| `npm run validate:strict` | Strict validation — `license`, `version`, `metadata.tags` become errors; the gate for new skills |
| `npm run validate:skill <path>` | Validate one skill (lenient; includes imported) |
| `npm run validate:signals` | Lint that every process skill declares a `## Progress Signals` section |
| `npm run doctor` | `check-prerequisites.sh --install --build-tools` then `smoke-test.sh` — full system health |
| `npm run build` | Build the marketplace distribution (the `.claude-plugin/` symlinks) |
| `npm run install:user` / `install:project` | Install skills to `~/.claude/skills/` or `.claude/skills/` |
| `npm run install:platforms` | Multi-platform installer (tsx) with full plugin support |
| `npm run install:opencode` / `install:skills` | OpenCode / flat-symlink installs |
| `npm run generate:commands` / `register:commands` / `unregister:commands` | Generate/register native slash commands |
| `npm run skill-matrix` / `skill-matrix:ci` | Pairwise skill-similarity collision report |
| `npm run update` / `update:force` | Pull and delta-install changed skills |
| `npm run format` / `check-format` | Prettier |

## `scripts/` — installers, validators, generators

| Script | Purpose |
|---|---|
| `install-skills-flat.sh` | Install skills as flat symlinks into each platform's skills dir (each becomes a slash command); configures kimi-code MCP. `[--uninstall]` |
| `install-platforms.ts` | Multi-platform symlink installer for Claude Code, OpenCode, Cursor, Codex, etc. `[--platform] [--scope] [--uninstall] [--list]` |
| `install.js` | Copy skills to user or project scope |
| `install-binaries.sh` | Build and install all six tool binaries to `~/.local/bin/` |
| `install-mcp-services.sh` | Render MCP `launchd` plists into `~/Library/LaunchAgents` and bootstrap them. `[--unload] [--user] [--dry-run]` |
| `configure-mcp-all-tools.sh` | Merge `mcp-port-table.json` into each tool's native MCP config (idempotent). `[--dry-run] [--tool]` |
| `prometheus-services.sh` | Manage MCP services as macOS user LaunchAgents (`install`/`load`/`status`/`doctor`/…) |
| `register-slash-commands.sh` | Register skills as OpenCode commands and Codex prompt files. `[--uninstall]` |
| `generate-commands.js` | Generate Claude Code slash-command files from skill frontmatter. `[--output] [--uninstall]` |
| `check-prerequisites.sh` | Check/install Node, Rust, npm deps; build tool binaries. `[--install] [--build-tools]` |
| `check-mcp-health.sh` | Health table (launchctl + HTTP probe) for all MCP services. `[--json]` |
| `smoke-test.sh` | Confirm every tool binary is reachable and answers `--version` |
| `detect-command-conflicts.sh` | Detect slash-command name collisions across installed command dirs |
| `skill-matrix.js` | Pairwise Jaccard similarity of skill name+description; CI fails on un-allowlisted collisions |
| `validate-skills.js` | The AgentSkills.io validator (Ajv schema). `[--strict] [--exclude-submodules]` |
| `validate-progress-signals.js` | Merge gate for the `## Progress Signals` section; ratchet baseline |
| `backfill-strict-fields.js` | Backfill missing `version`/`license`/`metadata.tags` into SKILL.md. `[--dry-run]` |
| `build-marketplace.js` | Build the `.claude-plugin/` symlink distribution |
| `update-skill-pack.sh` | Git-pull then delta-install changed skills across platforms (tracks last SHA). `[--force]` |

### The MCP port table

`scripts/mcp-port-table.json` is the declared source of truth for MCP connectivity. The full table is on the [MCP Substrate](05-mcp-substrate.md) page; in brief: surreal-memory (23001, SSE), prometheus-knowledge (8942, SSE), forge-rs (8943, SSE), and the stdio servers sycophancy-correction, liter-llm, sequential-thinking, tavily, and firecrawl.

### `scripts/scheduled/`

| File | Purpose |
|---|---|
| `periodic-nudge.sh` | Runs every 4 hours (`launchd` `ai.prometheus.prometheus-nudge`); POSTs a heartbeat to surreal-memory REST after a `/health` check; no-ops when unreachable; logs to `~/.prometheus/logs/` |

## `shared/scripts/` — the runtime hooks

These are the scripts the hooks fire (the events are mapped on the [Hooks & Lifecycle](15-hooks-and-lifecycle.md) page). Grouped by function:

**Context injection** — `pk-focus-on-prompt.sh`, `position-on-prompt.sh`, `detect-project-context.sh`, `pk-health.sh`, `memory-outbox-flush.sh`.

**Stop chain** — `write-session-summary.sh`, `position-stop-gate.sh`, `forge-reflect-on-stop.sh`.

**Position / waypoint** — `write-position-reminder.sh`, plus `lib/waypoint-render.sh`.

**Memory** — `memory-writeback.sh`, `mem0-compress.sh`, `lib/memory-bridge.sh`.

**PreToolUse guards (blocking)** — `scope-guard.sh`, `protect-tests.sh`, `pipeline-enforce.sh`, `cedar-skill-gate.sh`, `guard-direct-deploy.sh`.

**PostToolUse companions** — `scope-record.sh`, `validate-gitops-write.sh`, `sycophancy-check-artifact.sh`.

**SubagentStop gates** — `sycophancy-check-reflection.sh`, `evaluate-session.sh`, `propose-skill-update.sh`, `subagent-checkpoint-fallback.sh`.

**KBD lifecycle helpers** — `kbd-phase-status.sh`, `kbd-next-phase.sh`.

**Lint / health / verification** — `pk-lint.sh`, `pk-health.sh`, `verify-trace-state.sh`.

**Shared libraries (`lib/`)** — `hook-log.sh`, `path-scope.sh`, `sycophancy.sh`, `memory-bridge.sh`, `waypoint-render.sh`.

**Scheduled (`shared/scripts/scheduled/`)** — `ai.prometheus.mem0-compress.plist`, `ai.prometheus.pk-lint.plist`, `mem0-compress.cron`, `pk-lint.cron`. The mem0 compression runs weekly; the `pk lint --fix` sweep runs weekly.

## How the surfaces relate

```mermaid
graph TD
    NPM[npm scripts] -->|wrap| SH[scripts/*.sh + *.js + *.ts]
    SH -->|install| BIN[tool binaries → ~/.local/bin]
    SH -->|configure| CFG[per-tool MCP configs]
    BIN -->|run as| SVC[launchd / systemd MCP services]
    HOOKS[hooks.json] -->|fire| SHARED[shared/scripts/*.sh]
    SHARED -->|call| BIN
    SCHED[scheduled/*] -->|cron/launchd| SHARED
```

The mental model: `npm run` is the human entry point; it wraps the shell/JS/TS scripts in `scripts/`; those build the binaries and write the configs; the binaries run as background MCP services; and the hook scripts in `shared/scripts/` call those binaries at lifecycle events. The next pages — [Installation](19-installation.md) and [Updating](20-updating.md) — put these commands in the order you actually run them.

---

*Previous: [← 15 · Hooks & Lifecycle](15-hooks-and-lifecycle.md) · Next: [17 · Platform Support →](17-platform-support.md)*
