# Assessment — phase-codex-plugin-implementation

_Assessed 2026-07-11. Method: web research of the current Codex plugin spec (see `references/codex-plugin-spec-digest.md`) + repo inventory + UAR worktree scan (`references/uar-integration-notes.md`)._

## Headline

Codex-plugin packaging is **greenfield** in this repo (no `.codex-plugin/`, no `.agents/plugins/`), but the gap is **smaller than the phase brief assumed** — three findings de-risk it substantially:

1. **Codex reads `.claude-plugin/marketplace.json` as a documented *legacy* marketplace path.** The existing 11-plugin Claude marketplace is already partially consumable by Codex; native `.agents/plugins/marketplace.json` is an upgrade, not a from-scratch build.
2. **Plugin `hooks/hooks.json` uses the *same* PascalCase event schema as Claude** (`SessionStart`, `PreToolUse`, …). G-06 is a near-copy + trust documentation, **not** the snake_case translation the goal assumed. (Distinct from the failed `config.toml [hooks]` snake_case attempt noted in CLAUDE.md — the *plugin* hook path is the untried, spec-supported one.)
3. **`plugin.json` is near-isomorphic to `.claude-plugin/plugin.json`** (name/version/description/skills/mcpServers/hooks + an `interface` block). Existing build tooling (`build-marketplace.js`, `validate-skills.js`, `install-platforms.ts`, `codex-sync-skills.sh`, `config/codex-catalog.txt`) is extendable rather than replaceable.

## Current-state inventory

| Component | State |
|---|---|
| `.codex-plugin/plugin.json` | **absent** |
| `.agents/plugins/marketplace.json` (repo) + `~/.agents/plugins/` (personal) | **absent** |
| `.claude-plugin/{plugin.json,marketplace.json}` | present — v1.6.0, 30 skills, **11** marketplace plugins (Codex legacy-readable) |
| `.codex/config.toml` (repo) | present — 5 `[mcp_servers]` (Codex TOML MCP; generated) |
| `.codex/skills/` | present — SKILL.md copies (generated) |
| `.mcp.json` | present — canonical **7** servers (surreal-memory, sycophancy-correction, forge-rs, prometheus-knowledge, liter-llm, tavily, sequential-thinking) |
| `hooks/hooks.json` | present — 7 events (SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, SubagentStop, Stop, PreCompact) |
| build/sync tooling | `build-marketplace.js`, `validate-skills.js`, `install-platforms.ts`, `codex-sync-skills.sh`, `config/codex-catalog.txt` present |
| skills tree | `skills/<domain>/<name>/SKILL.md` (the layout UAR ingests as a submodule) |

## Gap analysis (per goal)

| Goal | Current | Target | Gap | Effort |
|---|---|---|---|---|
| **G-01** research spec digest | **DONE this assess** → `references/codex-plugin-spec-digest.md` (cited) | authoritative spec vendored | verify App-Server details if Mode 3/4 pursued | ✅ ~done |
| **G-02** `.codex-plugin/plugin.json` | absent | manifest mirroring Claude + `interface` | generate from `.claude-plugin/plugin.json` + add `interface` | **M** |
| **G-03** native `.agents/plugins/marketplace.json` | absent (legacy `.claude-plugin` readable) | repo + personal, Codex `source`/`policy` schema | transform 11 plugins: `source:"."`→`source.source/path`, add `policy.installation/authentication`, `interface.displayName` | **M** |
| **G-04** skills discovery | partial — `codex-sync-skills.sh` + budget file exist | plugin `skills` pointer → real-dir tree, within catalog budget | wire `skills` pointer; reuse non-symlink sync + budget curation | **L–M** |
| **G-05** MCP wiring | source `.mcp.json` (7) + repo `.codex/config.toml` (5) | plugin `.mcp.json` (map or `mcp_servers`) with all 7 + session fixes | emit plugin `.mcp.json`; carry forge token / liter-llm proxy / tavily-name / keys-from-env | **L–M** |
| **G-06** hooks | source `hooks/hooks.json` (PascalCase) | plugin hooks (SAME schema) + trust docs | point `plugin.json.hooks` at `hooks/hooks.json`; verify plugin (non-managed) hooks actually fire under trust; document | **L** (down from assumed M) |
| **G-07** build tooling | `build-marketplace.js` etc. present | generator emits + validates Codex artifacts idempotently | add Codex emit path (generated output, not hand-edited) + validation | **M** |
| **G-08** UAR compatibility | UAR reads `skills/` tree via submodule | root `.codex-plugin/` + `.agents/` don't touch `skills/`; `.codex/` regen intact | verify skill-tree + `.codex/` unaffected; version parity | **L** |
| **G-09** parity + docs | absent | parity checklist, publishing checklist, CLAUDE.md Codex section | author | **L–M** |

## Key risks / open questions for analyze/plan

- **Do plugin-bundled (non-managed) hooks actually execute in codex-cli 0.144.1 once trusted?** CLAUDE.md records the `config.toml [hooks]` path silently never firing. The plugin `hooks.json` path is spec-supported but **unverified here** — needs an empirical spike before committing G-06 scope. If it also no-ops, G-06 becomes "ship hooks.json, document as pending upstream."
- **`.mcp.json` `env` support in plugin manifests** is not enumerated in docs (only `command`/`args`). The 7 servers need env (tokens/keys) — confirm whether plugin `.mcp.json` honors `env`, or whether servers must rely on `~/.codex/config.toml` env passthrough (recall the tavily name-collision + env-passthrough gotcha from `[[codex-mcp-tavily-name-override]]`).
- **Marketplace `source` for a monorepo skill-pack:** `git-subdir` vs `local` — pick per intended install (in-repo dogfood = `local`; external = `git-subdir`/`git`).
- **Generated vs hand-authored:** `.codex/` is generated; keep new Codex artifacts generated too (single source of truth = `.claude-plugin` + `.mcp.json` + `skills/`).
- **Catalog budget:** 30 skills × Codex fixed catalog budget — confirm the plugin surface stays within the ~130-entry "full description" band (per CLAUDE.md table); curate via `config/codex-catalog.txt`.

## Suggested change decomposition (input to /kbd-plan)

~7 changes: (1) spec digest [done] → fold as reference; (2) plugin.json generator; (3) marketplace transform (Claude→Codex schema); (4) plugin `.mcp.json` emit + env strategy; (5) hooks wiring + trust spike; (6) build/validate/install integration; (7) parity + publishing docs + CLAUDE.md update + UAR-compat verification.

## Overall

Feasibility: **HIGH.** Mostly transformation of existing, well-formed Claude artifacts into the (largely parallel) Codex format, reusing existing tooling. Two empirical unknowns (plugin-hook execution, plugin `.mcp.json` env) should be spiked early in analyze/plan. No conflict with UAR ingestion (root-level Codex files don't touch the `skills/` tree).
