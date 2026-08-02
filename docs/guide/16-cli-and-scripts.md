# 16 · CLI & Scripts Reference

This page is the exhaustive index of every command-line surface the pack exposes: the binary CLIs (cross-referenced), the npm scripts, every installer and validator in `scripts/`, the runtime hook scripts in `shared/scripts/`, and the scheduled jobs. If you are looking for "what command does X," it is here.

## The binary CLIs

Documented in full on the [Tools Reference](13-tools-reference.md) page; summarized here for completeness.

| Binary | Role | Documented in |
|---|---|---|
| `prometheus` | Skill management, canonical KBD control/audit, self-learning, GitOps validation, Cedar policy, sycophancy | [13](13-tools-reference.md) |
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
| `install-mcp-services.sh` | Render/reload launchd or systemd user services, initialize Sovereign Sync operator/device identity, and probe managed ports. `[--unload] [--restart] [--user] [--dry-run]` |
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
| `generate-harness-adapters.js` | Generate Claude Code, Codex, OpenCode, and Kimi lifecycle adapters from one capability manifest |
| `check-harness-adapters.js` | Reject drift between the manifest and generated adapters |
| `check-kbd-direct-writers.js` | Reject new direct writers to canonical KBD compatibility projections |
| `validate-kbd-state.js` | Validate KBD projection schemas and revision relationships |
| `test-kbd-control-plane.sh` | Run runtime/journal/control-plane fixture tests |
| `generate-skill-eval-corpus.js` / `check-skill-evals.js` | Maintain and validate the 36-prompt critical-skill activation corpus |

### `prometheus kbd`

The canonical control CLI accepts a global project path before the subcommand:

```bash
prometheus kbd --path "/path/to/project" status --json
```

| Command | Purpose |
|---|---|
| `status [--json]` | Lifecycle, revision, plan, checkpoint, active path, completion, and lease |
| `pause --reason <text>` | Create the emergency valve and durable checkpoint |
| `revise --reason <text> [--exact-next-work <text>]` | Append immutable plan revision N+1 |
| `resume [--plan-revision <n>]` | Validate checkpoint/lease and resume |
| `cancel --reason <text>` | Terminal cancellation with audit history |
| `claim [--scope project/phase] [--force]` | Claim the single-writer lease |
| `heartbeat` / `release` / `handoff --to <harness>` | Maintain or transfer fenced ownership |
| `audit [--since <revision-or-event>] [--json]` / `watch` | Inspect immutable events |
| `migrate --check|--apply` | Inventory or import legacy ledgers |
| `rollout status|observe|promote` | Record non-authoritative shadow/canary evidence |
| `phase`, `stage`, `change`, `task` | Submit typed work-structure mutations |
| `completion`, `decision`, `blocker` | Record independent completion, decisions, and blockers |

Manual claims should name the receiving native adapter:

```bash
PROMETHEUS_HARNESS=claude-code \
  prometheus kbd --path "/path/to/project" claim
```

Full runbooks: [KBD control plane](/docs/kbd/control-plane),
[tokens](/docs/kbd/tokens-and-authentication), and
[leases](/docs/kbd/leases-and-handoffs).

### The MCP port table

`scripts/mcp-port-table.json` is the declared source of truth for MCP connectivity. The full table is on the [MCP Substrate](05-mcp-substrate.md) page; in brief: surreal-memory (23001, SSE), prometheus-knowledge (8942, SSE), forge-rs (8943, SSE), and the stdio servers sycophancy-correction, liter-llm, sequential-thinking, tavily, and firecrawl.

### `scripts/scheduled/`

| File | Purpose |
|---|---|
| `periodic-nudge.sh` | Runs every 4 hours (`launchd` `ai.prometheus.prometheus-nudge`); POSTs a heartbeat to surreal-memory REST after a `/health` check; no-ops when unreachable; logs to `~/.prometheus/logs/` |

## `shared/scripts/` — the runtime hooks

These are the scripts the hooks fire (the events are mapped on the [Hooks & Lifecycle](15-hooks-and-lifecycle.md) page). Grouped by function:

**Context injection** — `pk-focus-on-prompt.sh`, `position-on-prompt.sh`, `detect-project-context.sh`, `pk-health.sh`, `memory-outbox-flush.sh`.

**Stop event** — `kbd-harness-adapter.sh stop <harness>` queues bounded,
noncritical work and never blocks operator stop. Older summary/position
utilities remain implementation helpers but are no longer the installed
Claude Stop chain.

**Position / waypoint** — `write-position-reminder.sh`, plus `lib/waypoint-render.sh`.

**Memory** — `memory-writeback.sh`, `mem0-compress.sh`, `lib/memory-bridge.sh`.

**PreToolUse guards (blocking)** — `protect-tests.sh` only. It is the sole
remaining hook that can refuse a tool call, and it guards exactly one thing:
edits to existing BDD step definitions and feature files. `scope-guard.sh`,
`pipeline-enforce.sh`, `cedar-skill-gate.sh`, `guard-direct-deploy.sh`, and
`check-child-scope.sh` were unwired from `PreToolUse` — see
[Hooks & Lifecycle](15-hooks-and-lifecycle.md#what-was-removed-and-why). The
scripts remain on disk and can still be invoked directly or from CI.

**PostToolUse companions** — `scope-record.sh`, `validate-gitops-write.sh`, `sycophancy-check-artifact.sh`.

**SubagentStop gates** — `sycophancy-check-reflection.sh`, `evaluate-session.sh`, `propose-skill-update.sh`, `subagent-checkpoint-fallback.sh`.

**KBD lifecycle and fencing** — `kbd-harness-adapter.sh`,
`kbd-phase-status.sh`, `kbd-next-phase.sh`, and
`lib/runtime-authority.sh`.

**Lint / health / verification** — `pk-lint.sh`, `pk-health.sh`, `verify-trace-state.sh`.

**Shared libraries (`lib/`)** — `hook-log.sh`, `path-scope.sh`,
`sycophancy.sh`, `memory-bridge.sh`, `runtime-authority.sh`, and
`waypoint-render.sh`.

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
