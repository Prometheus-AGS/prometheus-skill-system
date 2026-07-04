# Assessment — cowork-integration

_Generated: 2026-07-03_

## Phase Goals Recap

| ID | Goal |
|----|------|
| G-01 | Architecture assessment of cowork fork + integration plan |
| G-02 | Platform support: Zed, Kimi Code CLI, MMX CLI, Kimi Desktop, MiniMax Desktop |
| G-03 | Skill-pack awareness: update/repair broken installations, toolchain management |
| G-04 | Claude Code plugin mechanics + Codex + OpenCode plugin install/manage |
| G-05 | Integration into install pipeline + documentation |

---

## 1. cowork-skills Fork — Codebase Assessment

### What it is

`git@github.com:GQAdonis/cowork-skills.git` is a Rust CLI tool (forked from `ZhangHanDong/cowork-skills`, v0.1.5, Rust 2024 edition) for installing and managing AI coding agent skills across multiple platforms. It builds two binaries: `cowork` and `co`.

### Current Platform Support (16 agents detected)

| Agent | Skills Dir | Status |
|---|---|---|
| Claude Code | `~/.claude/skills` | ✅ Fully Tested |
| Cursor | `~/.cursor/skills` | Community |
| Codex | `~/.codex/skills` | Community |
| GitHub Copilot | `~/.copilot/skills` | Community |
| Windsurf | `~/.codeium/windsurf/skills` | Community |
| Goose | `~/.config/goose/skills` | Community |
| Amp | `~/.config/agents/skills` | Community |
| Roo | `~/.roo/skills` | Community |
| Kiro CLI | `~/.kiro/skills` | Community |
| Gemini CLI | `~/.gemini/skills` | Community |
| OpenCode | `~/.config/opencode/skills` | Community |
| Antigravity | `~/.gemini/antigravity/skills` | Community |
| Clawdbot | `~/.clawdbot/skills` | Community |
| Droid | `~/.factory/skills` | Community |
| Kilo Code | `~/.kilocode/skills` | Community |
| Trae | `~/.trae/skills` | Community |

**MISSING from cowork (gaps vs. prometheus-skill-pack):**
- **Zed** — Not listed at all. prometheus-skill-pack installs to `~/.config/zed/skills` + `~/.zed/skills`
- **Kimi Code** — Not listed. prometheus installs to `~/.kimi-code/skills` + writes `~/.kimi-code/config.toml`
- **MMX / MiniMax CLI** — Not listed. prometheus detects `mmx` binary + installs to `~/.minimax/skills/`
- **Kimi Desktop** — Not listed anywhere; no known skill directory path yet
- **MiniMax Desktop** — Not listed; prometheus has desktop-specific notes but no separate install path

### Architecture (Key Modules)

```
cli/src/
├── main.rs          — clap 4.5 command definitions (11 top-level commands)
├── agents.rs        — Agent detection (checks ~/.claude, ~/.cursor, etc.) + path registry
├── config.rs        — Project configuration (Skills.toml parser)
├── commands/
│   ├── install.rs   — Install from GitHub/local; symlink or copy; multi-agent targeting
│   ├── plugins.rs   — Claude Code marketplace plugin read/write
│   ├── generate.rs  — Source code → SKILL.md generation
│   ├── audit.rs     — Security pattern detection
│   ├── verify.rs    — SHA-256 checksum verification
│   └── ...          — 8 more command modules
├── parser/          — Multi-language code parsing (Rust/syn, TS/Python/tree-sitter, Swift/regex)
└── github/          — octocrab GitHub API client
```

### Commands Available

| Command | Purpose |
|---|---|
| `cowork init` | Install built-in skills to `~/.claude/skills/` |
| `cowork install` | Install skills from GitHub/local path |
| `cowork generate` | Generate skills from source code |
| `cowork search` | Search GitHub for skill repos |
| `cowork plugins` | Manage Claude Code marketplace plugins |
| `cowork list` | List available skills |
| `cowork status` | Show configuration state |
| `cowork doctor` | Check config issues |
| `cowork config` | Manage Skills.toml |
| `cowork test` | Generate trigger tests |
| `cowork audit` | Security audit |
| `cowork verify` | Verify checksums |

### What cowork does NOT currently do

1. **MCP server configuration** — No support for writing `config.toml` / `opencode.json` / `mcp.json` entries post-install (prometheus-skill-pack's `configure_kimi_mcp()` / `configure_minimax_mcp()` are entirely absent)
2. **prometheus-skill-pack awareness** — No knowledge of how this pack is structured, its substrate Rust crates, or its install pipeline
3. **Toolchain management** — Does not invoke `cargo build`, detect broken services, or run `detect-toolchain.sh`
4. **Disk space management** — No cleanup of `target/` directories or build artifacts post-install
5. **Codex plugin management** — Only copies skills to `~/.codex/skills/`; no config.toml MCP wiring
6. **OpenCode plugin format** — Only copies to `~/.config/opencode/skills/`; no TypeScript plugin (`@opencode-ai/plugin`) installation
7. **Skill-pack update/repair** — No `cowork update-pack` or `cowork repair` subcommand exists

---

## 2. disk-space-guardian — Assessment

### What it is

A well-researched, thoroughly specified Rust project at `/Users/gqadonis/Projects/prometheus/disk-space-guardian` (fork: `git@github.com:GQAdonis/disk-space-guardian.git`). It is **spec-only** — zero implementation code exists.

### Current State

| Item | Status |
|---|---|
| Research & architecture docs | ✅ 800+ lines in docs/README.md |
| 5 OpenSpec change proposals | ✅ Defined, none executed |
| KBD phase (`dsg-cli-foundation`) | ✅ Planned, assessment + plan written |
| Cargo workspace / src/ | ❌ Does not exist |
| SKILL.md | ❌ Does not exist |
| Any implementation | ❌ None |

### Planned CLI

```
dsg scan [--deep] [--ecosystem rust|node|python] [--stale 30d]
dsg clean [--dry-run] [--force] [--target path]
dsg caches [--list] [--clean cargo]
dsg status [--json]
dsg config
dsg watch          # Phase 2
dsg schedule       # Phase 2
```

### Planned Safety Model (non-negotiable)

- 6-layer pipeline: dry-run → activity verification (lsof/fuser/git) → exclusion lists → trash (never rm) → retention policy → audit log
- Pressure thresholds: no-op (<70%), warn (70-80%), clean safe caches (80-85%), clean build caches (85-90%), aggressive (90-95%), emergency (95-99%), critical (>99%)

### Relevance to cowork-integration

The disk-space-guardian SKILL.md (once created) should be integrated into the cowork install pipeline because:

1. **Building prometheus-skill-pack substrate** (sovereign-sync, surface-bridge, etc.) creates multi-GB `target/` directories
2. **cowork install** clones repos to `~/.cowork/repos/` — no cleanup mechanism exists
3. **Native agent builds** (e.g., dsg itself, sovereign-sync) accumulate large build artifacts
4. **cowork** should be able to invoke `dsg clean --ecosystem cargo --dry-run` as a post-build hook

**Integration approach**: Add `cowork disk` or `cowork clean` subcommand that delegates to `dsg` if installed, or warns with install instructions if not. This is a Phase 2 concern.

---

## 3. prometheus-skill-pack Install Pipeline — Assessment

### Current Platform Coverage (14 platforms)

| Platform | Mechanism | MCP Config | Gap |
|---|---|---|---|
| Claude Code | Symlinks | `.mcp.json` (checked in) | None |
| OpenCode | Symlinks | `~/.opencode/opencode.json` | None |
| Kimi Code CLI | Symlinks + config.toml | `configure_kimi_mcp()` | None |
| MiniMax / Mavis | Copies + `_meta.json` | `configure_minimax_mcp()` | None |
| Cursor | Symlinks | None | None |
| Codex CLI | Symlinks | `~/.codex/config.toml` | None |
| Gemini CLI | Symlinks | None | None |
| Roo Code | Symlinks | None | None |
| Windsurf | Symlinks | None | None |
| Windsurf Legacy | Symlinks | None | None |
| Amp | Symlinks | None | None |
| Zed Editor | Symlinks | None | None |
| Antigravity (Zed fork) | Symlinks | None | None |
| Cline | Symlinks | None | None |

**Zed, Kimi, MMX are already handled in prometheus-skill-pack.** The gap is in cowork, not in the prometheus install script.

### Plugin Formats

| Platform | Plugin Format | Key File |
|---|---|---|
| Claude Code | JSON manifest + skills dir | `.claude-plugin/plugin.json` |
| OpenCode | TypeScript plugin (`@opencode-ai/plugin`) | `.opencode/plugin.ts` |
| Codex | Skills dir + config.toml MCP | `~/.codex/config.toml` |
| MiniMax | Skills dir + `_meta.json` | Per-skill `_meta.json` |
| Others | Skills dir only | None |

### Disk Space Gap in Current Pipeline

`install-skills-flat.sh` invokes `cargo build --release` for four substrate crates (storage-provider, learner-model, surface-bridge, sovereign-sync). No cleanup follows. Estimated disk impact: 1-4 GB per full install run, with no reclaim mechanism.

---

## 4. Gap Analysis Against Phase Goals

### G-01 — Architecture Assessment (THIS DOCUMENT)
- **Status**: IN PROGRESS (this assessment)
- **Gap**: None — assessment being written now
- **Output**: This file

### G-02 — Platform Support (Zed, Kimi Code CLI, MMX CLI, Kimi Desktop, MiniMax Desktop)
- **Zed**: Already in prometheus; NOT in cowork → must add to `agents.rs` with path `~/.config/zed/skills` (primary) and `~/.zed/skills` (fallback)
- **Kimi Code CLI**: Already in prometheus; NOT in cowork → must add to `agents.rs` with path `~/.kimi-code/skills` + post-install `config.toml` MCP wiring
- **MMX CLI**: Partially in prometheus (binary detection only); NOT in cowork → research MiniMax CLI config path; add to `agents.rs`
- **Kimi Desktop**: No skill directory known for desktop variant → requires research; likely ships separate config dir from CLI
- **MiniMax Desktop**: No skill directory known → requires research
- **Effort**: Medium — agents.rs additions + new install logic for MCP config (3-5 changes in cowork)

### G-03 — Skill-Pack Awareness
- **Gap**: cowork has zero knowledge of prometheus-skill-pack layout, substrate crates, or install pipeline
- **Required additions**:
  - `cowork pack status` — shows prometheus-skill-pack version + installed skills count per platform
  - `cowork pack update` — runs `bash scripts/install-skills-flat.sh` in the pack's install location
  - `cowork pack repair` — detects broken symlinks + re-runs install for affected platforms
  - `cowork toolchain` — delegates to `detect-toolchain.sh` and surfaces results in cowork's output format
  - Config in `Skills.toml` or `~/.cowork/config.toml`: `[prometheus] pack_root = "..."`
- **Effort**: Medium-high — new command module + config schema additions

### G-04 — Codex + OpenCode Plugin Management
- **Codex gap**: cowork copies skills but does not write `~/.codex/config.toml` MCP entries → add post-install MCP wiring matching prometheus pattern
- **OpenCode gap**: cowork copies skills but does not install `@opencode-ai/plugin` or write `opencode.json` → add post-install plugin registration
- **Claude Code plugin gap**: cowork CAN read existing plugins (plugins.rs), but cannot install new ones from git URL with manifest validation → extend `plugins install` to support `git@github.com:...` with manifest parsing
- **Effort**: Medium — extend existing commands/install.rs + new MCP config writer modules

### G-05 — Integration into Install Pipeline
- **Current state**: prometheus-skill-pack has its own install script (`install-skills-flat.sh`); cowork is a separate tool
- **Integration path**:
  1. Add cowork binary to the skill-pack's own install bootstrap (install cowork first, then use it)
  2. Create a `cowork-skills` skill in the pack that documents cowork usage
  3. Wire `cowork pack` commands as the primary management interface going forward
  4. Document in CLAUDE.md under "Essential Commands"
- **Effort**: Low-medium — mostly documentation + one bootstrap step

---

## 5. Worktree Strategy

Per the phase brief, work on cowork happens in a **dedicated worktree outside this directory**. Recommended approach:

```bash
# Clone the fork into a project worktree
git clone git@github.com:GQAdonis/cowork-skills.git \
  /Users/gqadonis/Projects/prometheus/cowork-skills

# Create a KBD worktree for the integration work
cd /Users/gqadonis/Projects/prometheus/cowork-skills
git checkout -b cowork-integration-prometheus
```

The worktree lives at `/Users/gqadonis/Projects/prometheus/cowork-skills` and is referenced from prometheus-skill-pack changes via relative path or absolute path in config.

---

## 6. Risk Register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Kimi Desktop / MiniMax Desktop skill dirs unknown | High | Medium | Research before coding; add conditional detection with graceful skip |
| MMX CLI config format differs from MiniMax | Medium | Medium | Add platform-specific detection branch in agents.rs |
| cowork Skills.toml conflicts with prometheus `skills/` naming conventions | Low | Low | Namespace under `[prometheus]` section |
| `dsg` not yet implemented when cowork calls it | High | Low | Guard with `dsg --version` check; warn if absent |
| Cargo build cache growth during cowork development | Medium | Medium | Integrate dsg or add `--clean-build` flag to cowork build step |
| cowork upstream diverges from fork during work | Low | High | Pin to fork HEAD SHA at start; merge upstream changes explicitly |

---

## 7. Recommended Change Sequence (for /kbd-analyze)

**Wave 1 — Foundation**
- change-001: Clone worktree + establish cowork fork development baseline; add Zed to agents.rs
- change-002: Add Kimi Code CLI to agents.rs + MCP config writer module
- change-003: Research + add MMX CLI; stub Kimi Desktop + MiniMax Desktop with graceful skip

**Wave 2 — Plugin Management**
- change-004: Extend `cowork plugins install` to support git URL + manifest validation for Claude Code
- change-005: Add Codex post-install MCP config wiring
- change-006: Add OpenCode post-install plugin registration (`@opencode-ai/plugin`)

**Wave 3 — Skill-Pack Awareness**
- change-007: Add `cowork pack` subcommand (status, update, repair)
- change-008: Add `cowork toolchain` subcommand delegating to detect-toolchain.sh
- change-009: Add `cowork disk` stub that invokes dsg or warns with install instructions

**Wave 4 — Integration**
- change-010: Add cowork bootstrap to prometheus install-skills-flat.sh
- change-011: Create `skills/process/cowork-management/SKILL.md` in skill-pack
- change-012: Update CLAUDE.md "Essential Commands" section

**Disk-space-guardian parallel track** (separate worktree, lower priority):
- dsg-change-001 through dsg-change-005: Execute existing OpenSpec changes to scaffold `dsg` CLI
- Integrate with cowork via change-009 once `dsg` binary is available

---

## 8. Open Questions for /kbd-analyze

1. **Kimi Desktop skill directory**: What path does Kimi Desktop use for skills? Is it the same as Kimi Code CLI (`~/.kimi-code/skills`) or different?
2. **MiniMax Desktop skill directory**: What path does MiniMax Desktop Agent use? Is it shared with the CLI (`~/.minimax/skills`) or separate?
3. **MMX CLI config format**: Does `mmx` use TOML config like Kimi, or JSON like MiniMax?
4. **cowork binary distribution**: Should cowork be published to crates.io, or distributed as a pre-built binary via the prometheus install script?
5. **dsg urgency**: Should disk-space-guardian scaffolding (change-001 through 005) happen before or after cowork integration? Current assessment: parallel track, non-blocking for Wave 1-3.
6. **cowork upstream sync policy**: How often should the GQAdonis fork pull from ZhangHanDong upstream? Recommend: pull before each wave.
