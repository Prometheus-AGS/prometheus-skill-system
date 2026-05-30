# Assessment: Machine Refresh & Global Install (Pull + Rebuild + Reinstall)

**Date**: 2026-05-30
**Assessor**: kbd-assess
**Phase**: `machine-refresh-2026-05-30`
**Prior phase**: `machine-installation-2026-05-25` (reflect_complete, 5/5 goals met)

**Goal**: Pull latest repo + submodules, rebuild all tools/skills/CLIs, and (re)install
them globally so they are usable from Claude Code, **Claude Desktop**, OpenCode, Codex,
and Zed on this machine.

> This is a **refresh/verify** phase, not a greenfield install. The 2026-05-25 phase
> already built and installed the full stack. This phase pulls 5 days of upstream drift,
> rebuilds against it, fixes skill-install drift, and closes the one platform the prior
> phase never covered: **Claude Desktop**.

---

## 1. Current Machine State (observed live, 2026-05-30)

### Repo & submodule drift (the reason this phase exists)

| Repo | Tracking | Behind upstream | Action |
|------|----------|-----------------|--------|
| `prometheus-skill-pack` (this repo) | `origin/main` | **2 commits** (`cdd08c7..eb3134b`) | `git pull` |
| `tools/liter-llm` | upstream | **149 commits** | `git submodule update --remote` + rebuild |
| `tools/surreal-memory-server` | upstream | **3 commits** | `git submodule update --remote` |
| `tools/prometheus-knowledge` | no upstream tracking (`n/a`) | unknown | inspect; pin or set tracking before update |
| `skills/imported/artifact-refiner` | `v1.2.0-1` | 0 | current ✅ |
| `skills/imported/sycophancy-correction` | `main` | 0 | current ✅ |

There is also a new remote branch `feat/kbd-orchestrator-w1-w3-2026-05-27` — informational
only; main is the integration target.

> **Note on remote name**: `origin` =
> `git@github.com:Prometheus-AGS/prometheus-skill-system.git`. Working tree is clean
> (only untracked `.agents/`, `.opencode/opencode-loop/`, and a stray assess md). A plain
> `git pull` is a fast-forward (2 ahead on remote, 0 local). Safe.

### Binaries — all 4 present in PATH (from prior phase)

| Binary | Location | Status | Rebuild needed? |
|--------|----------|--------|-----------------|
| `forge` | `~/.local/bin/forge` | ✅ in PATH | Yes — repo 2 commits behind |
| `pk-cherry` | `~/.local/bin/pk-cherry` | ✅ in PATH | Yes — pk submodule may move |
| `liter-llm` | `~/.local/bin/liter-llm` | ✅ in PATH | **Yes — submodule 149 behind** |
| `prometheus` | `~/.local/bin/prometheus` | ✅ in PATH | Yes — `setup` cmd lives here |

> `pk-cherry` is the real MCP binary; `pk-mcp` is a library crate (lesson from prior phase).

### Services — all healthy under launchd ✅ (no reinstall, only restart after rebuild)

| Service | launchd label | Port | Health |
|---------|---------------|------|--------|
| `forge-mcp` | `dev.prometheusags.forge-mcp` | 8943 | `{"service":"forge-mcp","status":"ok"}` |
| `pk-mcp` | `dev.prometheusags.pk-mcp` | 8942 | `{"entry_count":0,"status":"ok"}` |
| `openai-proxy` | `dev.prometheusags.openai-proxy` | 8181 | `{"status":"ok"}` |
| `surreal-memory-server` | Docker | 23001 | up (prior) |
| `surrealdb` | Docker | 28000 | up (prior) |

**After rebuild, the two launchd MCP services must be `kickstart -k`'d** to pick up new binaries.

### Skill install — drift across platforms (the second reason this phase exists)

| Target | Path | Skill count | Notes |
|--------|------|-------------|-------|
| Repo (native) | `skills/**/SKILL.md` | **82** | source of truth (99 incl. imported) |
| claude-code | `~/.claude/skills` → `~/.TOOLS/skills/claude` | 451 | **symlink to aggregate** — not driven by this repo's flat install; expected to differ |
| codex | `~/.codex/skills` | 328 | stale / over-populated |
| cursor | `~/.cursor/skills` | 128 | partial |
| zed | `~/.config/zed/skills` | 105 | partial |
| opencode | `~/.config/opencode/skills` | 94 | partial |

The non-claude targets should converge to the same ~99-skill set. The spread (94/105/128/328)
shows `install-skills-flat.sh` has not been re-run uniformly since the repo grew. **Re-running
the flat installer is the fix.** `install-skills-flat.sh` already covers: claude-code,
opencode, cursor, codex, gemini, roo, windsurf, cline, **zed** (9 platforms).

### MCP wiring — Claude Desktop is the gap

`.mcp.json` (source of truth) declares 7 servers:
`surreal-memory, sycophancy-correction, forge-rs, prometheus-knowledge, liter-llm, tavily, sequential-thinking`.

| Tool | Prometheus MCP servers wired? | Status |
|------|-------------------------------|--------|
| claude-code | project + user `.mcp.json` | ✅ (prior phase) |
| opencode | `~/.config/opencode/opencode.json` | ✅ (prior phase) |
| codex | `~/.codex/config.toml` | ✅ (prior phase) |
| zed | `~/.config/zed/settings.json` | ✅ (prior phase) |
| **Claude Desktop** | `~/Library/Application Support/Claude/claude_desktop_config.json` | ❌ **MISSING the prometheus stack** |

Claude Desktop currently has 13 servers (`mssql, filesystem, codex, surreal-memory,
dart-mcp-server, github, sequential-thinking, docfork, tavily-mcp, resend, dify-kb,
kubernetes, sycophancy-correction`). It is **missing** `forge-rs`, `prometheus-knowledge`
(pk-cherry), and `liter-llm`. The app bundle is installed at `/Applications/Claude.app`.

---

## 2. Gap Register

### G-PULL — Repo & submodules behind upstream
| ID | Gap | Action |
|----|-----|--------|
| G-PULL-1 | Repo 2 commits behind `origin/main` | `git pull` (fast-forward) |
| G-PULL-2 | `liter-llm` submodule 149 commits behind | `git submodule update --remote tools/liter-llm` |
| G-PULL-3 | `surreal-memory-server` 3 commits behind | `git submodule update --remote tools/surreal-memory-server` |
| G-PULL-4 | `prometheus-knowledge` has no upstream tracking | Inspect `.gitmodules`/branch; decide pin vs. track before updating |

### G-BUILD — Rebuild against pulled sources
| ID | Gap | Action |
|----|-----|--------|
| G-BUILD-1 | `liter-llm` binary stale (149 commits of drift) | `cargo build --release` in `tools/liter-llm`, reinstall to `~/.local/bin` |
| G-BUILD-2 | `forge`/`forge-mcp` may be stale | rebuild `forge-rs` workspace, reinstall |
| G-BUILD-3 | `pk-cherry` may be stale | rebuild `prometheus-knowledge`, reinstall |
| G-BUILD-4 | `prometheus` CLI may be stale | rebuild `prometheus-cli`, reinstall |
| G-BUILD-5 | launchd MCP services running old binaries after rebuild | `launchctl kickstart -k gui/$(id -u)/dev.prometheusags.forge-mcp` and `…pk-mcp` |

> Prefer `prometheus setup --non-interactive` (built in prior phase) as the install driver
> if it already does build+copy+kickstart; otherwise use `scripts/install-binaries.sh`.

### G-SKILL — Skill install drift
| ID | Gap | Action |
|----|-----|--------|
| G-SKILL-1 | opencode/cursor/codex/zed skill sets diverge from repo's 99 | Re-run `scripts/install-skills-flat.sh` (covers all 9 platforms incl. zed) |
| G-SKILL-2 | Verify claude-code aggregate (`~/.TOOLS/skills/claude`) includes this repo's skills | Confirm symlink/source includes repo skills; reconcile if not |

### G-DESKTOP — Claude Desktop never wired (NEW vs. prior phase)
| ID | Gap | Action |
|----|-----|--------|
| G-DESKTOP-1 | `claude_desktop_config.json` missing `forge-rs` | Add stdio/SSE entry from `.mcp.json` |
| G-DESKTOP-2 | Missing `prometheus-knowledge` (pk-cherry) | Add entry |
| G-DESKTOP-3 | Missing `liter-llm` | Add entry |
| G-DESKTOP-4 | No installer step targets Claude Desktop | Add Claude Desktop as a wiring target (merge, don't clobber its 13 existing servers); restart app to load |

> **CRITICAL — merge semantics**: Claude Desktop config is hand-curated (13 servers).
> The wiring step MUST merge the 3 prometheus servers in, preserving all existing entries.
> Never overwrite the file wholesale.

### G-VERIFY — End-state certification
| ID | Gap | Action |
|----|-----|--------|
| G-VERIFY-1 | No post-refresh health gate | After all steps: `prometheus setup --check` (or `doctor`); curl 8942/8943/8181; confirm 4 binaries; spot-check skill counts converge; confirm Claude Desktop has 16 servers |

---

## 3. What is already done (no action)

| Component | Status |
|-----------|--------|
| 4 binaries in `~/.local/bin` | ✅ present (rebuild only) |
| forge-mcp / pk-mcp / openai-proxy launchd | ✅ healthy (restart after rebuild) |
| surreal-memory / surrealdb Docker | ✅ up |
| MCP wiring: claude-code, opencode, codex, zed | ✅ from prior phase |
| `install-skills-flat.sh` zed target | ✅ present |
| imported submodules (artifact-refiner, sycophancy) | ✅ current |
| `prometheus setup` subcommand | ✅ exists (reuse as install driver) |

---

## 4. Recommended Phase Plan (preview for /kbd-plan)

| # | Change | Effort | Closes |
|---|--------|--------|--------|
| 1 | `pull-repo-and-submodules` — `git pull`, resolve pk tracking, `submodule update --remote` | 15 min | G-PULL-1..4 |
| 2 | `rebuild-and-reinstall-binaries` — rebuild 4 crates, reinstall, `kickstart -k` services | 30–45 min | G-BUILD-1..5 |
| 3 | `reinstall-skills-all-platforms` — re-run flat installer, verify convergence | 15 min | G-SKILL-1..2 |
| 4 | `wire-claude-desktop-mcp` — **merge** forge-rs/pk/liter-llm into desktop config, restart app | 20 min | G-DESKTOP-1..4 |
| 5 | `verify-refresh` — health gate across binaries, services, skills, all 5 tools | 15 min | G-VERIFY-1 |

**Total effort**: ~1.5–2 hours.

---

## 5. Open Decisions for the user (surface in plan, not assess)

1. **`prometheus-knowledge` submodule has no upstream tracking** — pull to a branch tip, or
   keep pinned at current commit `ee611fc`? (Default: keep pinned unless an update is wanted.)
2. **Install driver** — use the existing `prometheus setup --non-interactive`, or run the
   discrete `scripts/install-binaries.sh` + `install-skills-flat.sh` steps directly?
   (Default: prefer `prometheus setup` if it already does build+copy+kickstart; fall back to scripts.)
3. **Claude Desktop "Claude Desktop" ≠ "Claude Code"** — confirmed the request's "Claude
   Desktop" means the macOS app at `/Applications/Claude.app` (config wired here), distinct
   from Claude Code CLI (already wired).

---

*Assessment written to: `.kbd-orchestrator/phases/assess/machine-refresh-assessment-2026-05-30.md`*
*Next: `/kbd-plan machine-refresh-2026-05-30` to produce the ordered change list.*
