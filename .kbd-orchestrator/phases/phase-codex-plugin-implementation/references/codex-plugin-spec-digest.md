# Codex Plugin Spec Digest (G-01)

_Researched 2026-07-11 via web. Sources: `learn.chatgpt.com/docs/plugins`, `learn.chatgpt.com/codex/build-plugins`, `learn.chatgpt.com/codex/mcp-server` (developers.openai.com/codex/plugins → 308 → learn.chatgpt.com/docs/plugins)._

## Package layout

```
my-plugin/
├── .codex-plugin/
│   └── plugin.json        # ONLY plugin.json lives here
├── skills/<name>/SKILL.md
├── hooks/hooks.json
├── .mcp.json
├── .app.json
└── assets/
```
Component paths in the manifest must start with `./`, resolve relative to plugin root, stay inside it.

## `plugin.json` (`.codex-plugin/plugin.json`)

- **Required:** `name`, `version`, `description`
- **Optional metadata:** `author` {name,email,url}, `homepage`, `repository`, `license`, `keywords`
- **Component pointers:** `skills`, `mcpServers`, `apps`, `hooks`
- **`interface`:** `displayName`, `shortDescription`, `longDescription`, `developerName`, `category`, `capabilities`, website/privacy/terms URLs, `defaultPrompt`, `brandColor`, `composerIcon`, `logo`, `screenshots`

Purpose: identify the plugin, point to components, provide install-surface metadata.

## `.mcp.json`

Two accepted forms — **direct map** or **wrapped** `{"mcp_servers": {...}}`. Server fields documented: `command`, `args` (docs don't enumerate `env`/`type`, but Codex `config.toml` MCP tables use `type="sse"`+`url` for remote and `command`/`args`/`env_vars` for stdio — the plugin `.mcp.json` mirrors that server shape).

## `hooks/hooks.json`

Same **PascalCase event schema as Claude Code** ("receiving identical event schemas"). Documented example uses `SessionStart`. Entry shape:
```json
{"hooks":{"SessionStart":[{"hooks":[{"type":"command","command":"python3 ${PLUGIN_ROOT}/hooks/session_start.py","statusMessage":"…"}]}]}}
```
- Env vars supplied: **`PLUGIN_ROOT`** (installed package), **`PLUGIN_DATA`** (writable data dir).
- **Trust:** plugin hooks are **"non-managed hooks"** — Codex **skips them until the user reviews & trusts** the current definition. Installing/enabling ≠ trusting.

## Marketplace (`marketplace.json`)

- **Repo path:** `$REPO_ROOT/.agents/plugins/marketplace.json`
- **Personal path:** `~/.agents/plugins/marketplace.json`
- **Legacy path (READ):** `$REPO_ROOT/.claude-plugin/marketplace.json` ← Codex already reads the Claude marketplace
- **Top-level:** `name`, `interface.displayName`, `plugins[]`
- **Per-plugin:** `name`, `source`, `policy`, `category`
  - `source.source` ∈ `"local"` | `"url"` | `"git-subdir"` | `"npm"`; local → `source.path` (`./`-relative, in-root)
  - `policy.installation` ∈ `AVAILABLE` | `INSTALLED_BY_DEFAULT` | `NOT_AVAILABLE`
  - `policy.authentication` ∈ `ON_INSTALL` | first-use

## Install surfaces

ChatGPT Work (web), ChatGPT desktop (Work/Codex), Codex CLI (`/plugins` command; also `codex plugin …`), IDE extension (Settings > Plugins). Bundled skills become available **on a new session after install**; connectors need sign-in; MCP servers may need setup.

## Codex as an MCP server (Mode 2 — for reference)

`codex mcp-server` exposes two tools: **`codex`** (start; required `prompt`, optional `approval-policy`/`sandbox`/`config`/`cwd`/`model`) and **`codex-reply`** (continue; `prompt` + `threadId` from prior `structuredContent.threadId`).

## App Server (Modes 3–4 — UAR-side, out of this phase's packaging scope)

Stateful `thread → turn → item` JSON-RPC over stdio/JSONL (+ experimental WS/Unix). Deferred to UAR repo; captured for G-08 compatibility awareness only.
