# Goals — phase-codex-plugin-implementation

## Objective

Give the prometheus-skill-pack **full Codex-plugin parity** with its existing
Claude-Code plugin/marketplace: research the latest Codex plugin specifications
(web) and produce a Codex CLI + desktop plugin package and marketplace for this
skill set — the same way `.claude-plugin/plugin.json` + `marketplace/marketplace.json`
are produced today — while staying compatible with UAR consuming this repo as a
git submodule.

## Goals

| ID | Goal | Verification (target) |
|----|------|-----------------------|
| **G-01** | Research the latest Codex plugin specs via web (`developers.openai.com/codex/plugins`, `learn.chatgpt.com/codex/{build-plugins,mcp-server,app-server}`) and vendor a spec digest under `references/` | `references/codex-plugin-spec-digest.md` exists with manifest schema, marketplace paths, PLUGIN_ROOT/PLUGIN_DATA, hook event names, all cited |
| **G-02** | Emit a Codex plugin package: `.codex-plugin/plugin.json` wiring `skills`/`mcpServers`/`hooks`/optional `apps`, `./`-relative in-root paths + `interface` block | `.codex-plugin/plugin.json` validates against the digest; paths resolve inside root |
| **G-03** | Emit a Codex marketplace: repo `.agents/plugins/marketplace.json` (+ personal `~/.agents/plugins/marketplace.json` install path), mirroring the 11-plugin granularity of the Claude marketplace | `codex plugin marketplace add .` succeeds; lists the plugin(s) |
| **G-04** | Adapt skills to Codex plugin discovery honoring documented constraints: **real directories (non-symlink)**, the fixed **catalog budget**, and the `skills/<domain>/<name>/SKILL.md` layout UAR also depends on | Skills load in a fresh Codex session; `codex-catalog-stat.py` within budget; UAR skill-tree unchanged |
| **G-05** | Wire the 7 prometheus MCP servers (surreal-memory, liter-llm, sycophancy-correction, forge-rs, prometheus-knowledge, tavily, sequential-thinking) into the plugin MCP config in Codex format, carrying this session's fixes (forge `FORGE_MCP_TOKEN`, liter-llm proxy config, tavily name-collision, keys-from-env) | Each server initializes via the plugin; no committed secrets |
| **G-06** | Translate `hooks/hooks.json` (PascalCase: SessionStart/UserPromptSubmit/PreToolUse/PostToolUse/SubagentStop/Stop/PreCompact) to Codex **snake_case** hooks with `${PLUGIN_ROOT}`/`${PLUGIN_DATA}` and independent per-component trust — or document why deferred (CLAUDE.md open item) | Hooks parse in Codex; trust model documented |
| **G-07** | Build/validate/install tooling that **generates** the Codex plugin + marketplace (generated output, not hand-edited), integrated with `codex-sync-skills.sh` + `install-platforms.ts` codex target | `npm run build`-equivalent emits + validates Codex artifacts idempotently |
| **G-08** | Guarantee UAR-deployment compatibility: emitted artifacts + skill tree stay consumable by UAR's submodule ingestion (`$UAR_BUILTIN_SKILLS_DIR`, immutable built-ins); don't regress `.codex/config.toml` / `.codex/skills/` generation | UAR builtin-skill scan still loads the pack; `.codex/` regen unaffected |
| **G-09** | Parity checklist vs the Claude-Code plugin (skills/mcp/hooks/marketplace/apps), publishing checklist, and CLAUDE.md Codex-section docs | `docs/` parity + publishing checklists; CLAUDE.md updated |

## Context & reference spec

The invoking brief (a full Codex plugin architecture overview: package format,
skills, MCP both directions, apps/connectors, hooks with trust, marketplaces,
and the App Server thread/turn/item model) is the design north-star. It frames
Codex plugins as an **installable agent-capability package format** separating
Instructions (skills) / Capabilities (MCP+apps) / Lifecycle (hooks) /
Distribution (plugins+marketplaces). This phase implements the **Distribution +
packaging** layer for THIS skill set — Modes 2–4 (UAR↔Codex MCP / App-Server
adapters) are UAR-repo work, out of scope here except for compatibility (G-08).

## UAR integration constraints (from worktree scan — see references/uar-integration-notes.md)

- UAR consumes this repo as a **git submodule** (`crates/prometheus-skill-system`)
  and reads the **`skills/<domain>/<name>/SKILL.md` tree directly** at startup —
  independent of any plugin manifest. **Do not break that layout.**
- **No Codex plugin format exists yet** in this repo or UAR — this phase fills a
  genuine gap. Today Codex is wired only via `.codex/config.toml` (TOML
  `[mcp_servers.<name>]`) + `.codex/skills/`, both **generated output**.
- **SKILL.md / agentskills.io is the portability contract** across harnesses.
- MCP server names must match the existing 7-server set; Codex expects
  `[mcp_servers.<name>]` TOML (`type="sse"`+`url`, or `command`+`args`+`env_vars`).

## Non-goals

- UAR-side plugin loader / `PluginRuntime` / App-Server adapter (UAR repo).
- Changing UAR's submodule ingestion mechanism.
- Publishing to a public registry (that's a later marketplace-distribution phase).

## References

- `references/codex-plugin-spec-digest.md` (G-01 output — to be produced)
- `references/uar-integration-notes.md` (worktree scan findings)
- `.claude-plugin/plugin.json`, `marketplace/marketplace.json` (parity source)
- CLAUDE.md → "Codex CLI Integration" (existing constraints)
- `scripts/codex-sync-skills.sh`, `scripts/install-platforms.ts`, `config/codex-catalog.txt`
