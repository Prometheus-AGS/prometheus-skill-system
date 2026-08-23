# Codex Plugin & Marketplace

The skill-pack ships a **Codex CLI/desktop plugin** in parity with its Claude-Code
plugin. All Codex artifacts are **generated** from the canonical Claude sources —
never hand-edit them.

| Generated artifact | From | Emitted by |
|---|---|---|
| `.codex-plugin/plugin.json` | `.claude-plugin/plugin.json` (+ `interface` block) | `scripts/build-codex-plugin.js` |
| `.agents/plugins/marketplace.json` | `.claude-plugin/marketplace.json` (source→`{source,path}`, +`policy`) | `scripts/build-codex-plugin.js` |
| MCP servers | `.mcp.json` (referenced by pointer — Codex reads the `mcpServers`-wrapper form as-is) | — |
| Hooks | `shared/harnesses/hook-contract.json` | `scripts/generate-harness-adapters.js` → `hooks/codex-hooks.json` |

## Build / validate

```bash
npm run build:codex       # regenerate .codex-plugin/plugin.json + .agents/plugins/marketplace.json
npm run validate:codex    # CI guard: fails if artifacts are stale or invalid (no write)
```

`build:codex` is idempotent (byte-stable). `validate:codex` (`--check`) detects
drift and validates required fields + `./`-in-root paths. This is a mandatory
local gate; hosted workflows do not build or validate the package.

Every target in `skill-system.json` also declares `sourceTreeLifecycle`.
Repository-owned harness mirrors are `required` and must be present and
populated. Destinations materialized only during installation are
`install-only` and may be absent from the source checkout. The shared contract
loader enforces this before distribution generation or installation begins.

### Distribution & env

- **External publish:** `CODEX_MARKETPLACE_SOURCE=git-subdir CODEX_MARKETPLACE_REF=main npm run build:codex`
  emits `source.{source:"git-subdir",url,ref,path}` per plugin (needs a pushed
  commit). Default is `local` (in-repo dogfood, byte-stable).
- **MCP env provisioning:** `bash scripts/codex-provision-mcp-env.sh` writes
  `[shell_environment_policy] inherit = "all"` to `~/.codex/config.toml` so Codex
  forwards your shell env (keys/tokens) to the plugin's MCP servers. It persists
  **no secret values**. Fallback for a stubborn server: an inline
  `[mcp_servers.<name>] env = { KEY = "…" }` block in `~/.codex/config.toml`
  (0600, user-local) — as done for `tavily_web` this cycle.

## Install (verified against codex-cli 0.144.1)

```bash
# repo-local dogfood (reads .agents/plugins/marketplace.json)
codex plugin marketplace add .
codex plugin add prometheus-skill-pack@prometheus-skill-pack   # umbrella (INSTALLED_BY_DEFAULT)
# or a domain pack, e.g.:
codex plugin add learn@prometheus-skill-pack

codex plugin list          # 11 plugins resolve to their subdirs
codex mcp list             # the 7 MCP servers register from the plugin's .mcp.json
codex doctor               # health
```

Capabilities become available **on a new Codex session** after install. Personal
scope: `~/.agents/plugins/marketplace.json`. Codex also reads the legacy
`.claude-plugin/marketplace.json`, so the pack was already partially Codex-visible.

## Hooks — interactive, non-managed trust (change-005)

`plugin.json.hooks → ./hooks/codex-hooks.json`. Codex and Claude manifests are
generated separately from one declarative hook contract while retaining the
same PascalCase event schema (`SessionStart`, `PreToolUse`, …).

Each generated command embeds an immutable bundle ID. It first resolves that ID
through `~/.prometheus/plugins/prometheus-skill-pack/runtime/v1/run-hook`; if the
bundle has not been activated, it uses the native plugin payload exposed through
`${PLUGIN_ROOT}` to perform a hash-verified bootstrap. Hook business logic never
resolves through the mutable `stable` or `current` projections.

Publishing a new signed plugin generation deterministically refreshes that
embedded bundle ID in both `hooks/hooks.json` and `hooks/codex-hooks.json`.
Those files must change together, and the active-generation manifest plus all
14 target receipts must identify the same bundle before activation. A bundle-ID
refresh changes provenance, not hook event names, matchers, trust behavior, or
the unrestricted Bash/Python tool policy.

**Trust is independent of install.** Plugin-bundled hooks are *non-managed*:
an interactive `codex` session shows a one-time trust prompt before running them.
Consequences:

- Installing/enabling the plugin does **not** run its hooks until trusted.
- **Firing is verified** (change-cpd-006, codex-cli 0.144.1): a `SessionStart`
  hook fires and writes to `${PLUGIN_DATA}`. It can be exercised **headlessly**
  via `codex exec --dangerously-bypass-hook-trust` (built for vetted automation) —
  it is *not* interactive-only. Evidence:
  `.kbd-orchestrator/phases/phase-codex-plugin-distribution-and-ci/references/hook-trust-verification.md`.
- **Portability:** bootstrap uses `${CLAUDE_PLUGIN_ROOT:-$PLUGIN_ROOT}` so the
  generated command can acquire its pinned bundle under both harnesses.
  Codex provides `PLUGIN_ROOT` / `PLUGIN_DATA`, not `CLAUDE_PLUGIN_ROOT`.
- Distinct from the earlier `config.toml [hooks]` snake_case attempt (which
  silently never fired). The plugin `hooks.json` path is the working one.

## MCP servers — env provisioning (change-004)

The 7 servers register from the shared `.mcp.json` (which already carries this
repo's `${VAR:-default}` fallbacks and the forge bearer-token / liter-llm proxy
config / tavily fixes). `codex doctor` warns when a server's env var is unset;
provide keys via the environment or `~/.codex/config.toml` — **never commit
secrets**. (See the tavily name-collision note in CLAUDE.md for a Codex MCP gotcha.)

## Parity checklist vs the Claude-Code plugin

| Component | Claude | Codex | Status |
|---|---|---|---|
| Plugin manifest | `.claude-plugin/plugin.json` | `.codex-plugin/plugin.json` (+`interface`) | ✅ generated |
| Marketplace | `.claude-plugin/marketplace.json` (`source:"."`) | `.agents/plugins/marketplace.json` (`source.{source,path}`+`policy`) | ✅ generated, 11 plugins |
| Skills | 30 curated (array) | same 30 (array); budget curated via `config/codex-catalog.txt`; real dirs via `codex-sync-skills.sh` (Codex ignores symlinks) | ✅ |
| MCP | `.mcp.json` (7) | same `.mcp.json` via pointer | ✅ 7 register |
| Hooks | generated `hooks/hooks.json` (managed) | generated `hooks/codex-hooks.json` (non-managed, interactive trust) | ✅ bundle-pinned |
| Apps (`.app.json`) | — | — | n/a (no connectors yet) |

## Publishing checklist

1. Bump `version` in `.claude-plugin/plugin.json` (source of truth).
2. `npm run build:codex` — regenerate; `npm run validate:codex` — assert clean.
3. `codex plugin marketplace add .` + `codex plugin add …` smoke test; `codex doctor`.
4. Commit `.codex-plugin/`, `.agents/plugins/`, `scripts/build-codex-plugin.js`.
5. For external distribution, switch a plugin's marketplace `source` to `git-subdir`/`git`.

## UAR compatibility (change-008)

UAR consumes this repo as a git submodule and ingests the `skills/<domain>/<name>/SKILL.md`
tree directly (`$UAR_BUILTIN_SKILLS_DIR`). The Codex artifacts live at repo root
(`.codex-plugin/`, `.agents/`) and **do not touch `skills/`** — verified: this
phase produced zero changes under `skills/` and did not regenerate `.codex/`.
UAR submodule ingestion is therefore unaffected.
