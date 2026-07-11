# UAR Integration Notes — Codex Packaging Reconnaissance

_Source: scan of the in-progress UAR worktree `/Users/gqadonis/.claude/worktrees/uar-production-release` (branch `production-release`, HEAD `5417046`), 2026-07-11. Assess-phase input for phase-codex-plugin-implementation._

## (A) How UAR ingests this skill pack

- Consumed as a **git submodule**: `.gitmodules` → `crates/prometheus-skill-system` ← `github.com/Prometheus-AGS/prometheus-skill-system`.
- UAR discovers skills **at startup from disk** as `Builtin`/"Manifest" skills (SKILL.md frontmatter+body). Scan dir = `$UAR_BUILTIN_SKILLS_DIR` (default `crates/prometheus-skill-system/skills`); layout `skills/<domain>/<name>/SKILL.md` (`docs/skill-authoring.md`). In-container path `/opt/uar/skills/builtin/`.
- Sub-submodule imports gated by `UAR_LOAD_IMPORTED_SKILLS=true`.
- Built-ins are **immutable via API** (DELETE → `409 system_skill_immutable`); mutate by changing the submodule + redeploying.
- **UAR consumes the skill directory tree, independent of any plugin/marketplace manifest.**

## (B) Codex plugin state — exists vs planned

- **No Codex CLI *plugin* format anywhere in UAR** (no `.codex-plugin/plugin.json`, `.agents/plugins/marketplace.json`, `codex mcp-server`, `app-server`, `PluginRuntime`).
- `docs/CODEX_ASSESSMENT.md` is a UI/UX review authored *by* the Codex agent — unrelated to packaging.
- Current Codex integration (all in the submodule): `.codex/config.toml` (Codex MCP TOML, `[mcp_servers.<name>]`), `.codex/skills/` (SKILL.md copies), installers `scripts/install-platforms.ts` (codex target: global `~/.codex/skills`, project `.codex/skills`) + `scripts/installers/codex/install.sh`.
- **The Claude-Code plugin format has no Codex mirror — this phase fills that gap.**

## (C) Constraints to honor

- SKILL.md / agentskills.io is the portability contract (`docs/uar-next-fable.md`). Keep `skills/<domain>/<name>/SKILL.md`.
- MCP server naming must match the 7-server set; Codex expects `[mcp_servers.<name>]` TOML (`type="sse"`+`url` remote, or `command`/`args`/`env_vars` stdio).
- Manifest field parity with `.claude-plugin/plugin.json` (`name=prometheus-skill-pack`, `version`, `skills[]`, `mcpServers`, `compatibility.platforms` already lists `"codex"`).
- Marketplace source types: `{type:"git",url,path}` + top-level YAML frontmatter + JSON body; `plugins[]` name/description/source/version/tags/category.

## (D) Concrete file/format references (in UAR submodule = this repo)

- `.codex/config.toml` (Codex MCP TOML)
- `.claude-plugin/plugin.json` (manifest to mirror)
- `marketplace/marketplace.json` (frontmatter + `marketplace_version`/`owner`/`plugins[].source{type,url,path}`)
- `.mcp.json` (canonical 7-server definitions)
- `scripts/install-platforms.ts`, `scripts/build-marketplace.js`, `scripts/installers/codex/install.sh`
- `docs/skill-authoring.md`, `docs/uar-next-fable.md`, `docs/NATIVE_SKILLS.md`

> **Important:** `.codex/` is **generated output** (regenerated 2026-07-11). Produce Codex artifacts via the pack's build scripts, not by hand-editing.
