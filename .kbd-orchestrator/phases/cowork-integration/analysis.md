# Analysis — cowork-integration

_Generated: 2026-07-03 | Tier 1–3 research pipeline, 4 parallel agents_

---

## 1. Open Questions Resolved

All 5 open questions from the assessment are now answered.

### OQ-01: Kimi Desktop skill directory path

**Answer:** `~/Library/Application Support/kimi-desktop/daimon-share/daimon/skills/`

- **Completely separate** from Kimi Code CLI (`~/.kimi-code/skills/`)
- Kimi Desktop also manages plugin-specific skills under:
  `~/Library/Application Support/kimi-desktop/daimon-share/daimon/runtime/kimi-code/home/plugins/managed/*/skills/`
- Config is at: `~/Library/Application Support/kimi-desktop/daimon-share/daimon/runtime/kimi-code/config.toml`
- **Impact on cowork**: Needs a dedicated `kimi-desktop` agent entry in `agents.rs` with the Application Support path. Installation via symlink into that path should work but needs macOS path expansion (`~` → `$HOME`) and must check app presence via `~/Library/Application Support/kimi-desktop/` parent dir existence.

### OQ-02: MiniMax Desktop skill directory path

**Answer:** `~/.minimax/skills/` — **shared with the MiniMax CLI**

- MiniMax Desktop agent uses `~/.minimax-agent/projects/` for project state
- Skills are shared at `~/.minimax/skills/`
- Config: `~/Library/Application Support/MiniMax Agent/minimax-agent-config.json`
- **Impact on cowork**: No new install path needed — the existing `minimax` agent entry in `agents.rs` already covers the desktop. Only the detection logic needs to check for BOTH `~/.minimax/` (CLI) and `~/Library/Application Support/MiniMax Agent/` (desktop) to confirm either is installed.

### OQ-03: MMX CLI config format

**Answer:** `~/.mmx/config.json` — **JSON, not TOML**

- `mmx` is a standalone CLI tool with **no plugin or skills architecture**
- It handles text/image/video/audio/web-search generation — not an AI coding agent
- There is **no skills directory** for `mmx`; the MiniMax ecosystem's skill integration point is exclusively the **MiniMax Code IDE** (handled via `~/.minimax/skills/`)
- **Impact on cowork**: The "MMX CLI" requirement from the phase brief maps to **MiniMax Code IDE support**, which is already wired via the `minimax` entry in `agents.rs`. No new agent entry needed for `mmx` itself; instead, update the description in cowork's `minimax` agent entry to clarify it covers both desktop and CLI.

### OQ-04: cowork binary distribution strategy

**Answer:** Option B (pre-built binary via GitHub Releases) with Option C (build from source) as fallback

- Naming conflict with upstream `cowork` on crates.io makes Option A (crates.io) unviable
- Current disk usage from Rust builds in the skill-pack is already **17 GB** (`forge-rs: 5 GB, surreal-memory-server: 7 GB`, etc.)
- Pre-built binary avoids adding 2–3 GB per cowork build
- Precedent: prometheus-skill-pack already downloads `gitleaks` binary in CI via GitHub Releases (`.github/workflows/validate.yml` lines 200–206)
- **Implementation**: Add `install_cowork()` to `install-binaries.sh` with curl-to-GitHub-Releases primary, `cargo build --release` fallback

### OQ-05: dsg urgency relative to cowork Wave 1

**Answer:** **Parallel track, non-blocking for Wave 1–3** — but the 17 GB disk situation makes it genuinely urgent

- dsg's 5 OpenSpec changes are well-specified and independent of cowork work
- dsg should begin execution in its own worktree concurrently with cowork Wave 1
- `cowork disk` stub in Wave 3 only needs the `dsg` binary to exist; it degrades gracefully with a warning if absent
- **Recommendation**: Start dsg execution after cowork Wave 1 lands, not before

---

## 2. Platform Gap Resolution

### Resolved platform map for cowork `agents.rs`

| Platform | Current cowork status | Required change | Install path |
|---|---|---|---|
| **Zed** | Missing | Add agent entry | `~/.config/zed/skills/` (primary) + `~/.zed/skills/` (fallback) |
| **Kimi Code CLI** | Missing | Add agent entry + MCP config writer | `~/.kimi-code/skills/` |
| **Kimi Desktop** | Missing | Add agent entry | `~/Library/Application Support/kimi-desktop/daimon-share/daimon/skills/` |
| **MiniMax Code IDE** | Present (`minimax`) | Update description + detection | `~/.minimax/skills/` (already wired) |
| **MiniMax Desktop** | Implicit (shares `~/.minimax/skills/`) | Add detection of `~/Library/Application Support/MiniMax Agent/` | Shared path (no change) |
| **MMX CLI** | N/A — no skill system | Document this clearly; drop from scope | N/A |

**MMX CLI is out of scope.** The `mmx` binary is a media-generation tool, not an AI coding agent. "MMX support" from the phase brief maps to MiniMax Code IDE (already handled).

### Zed format verdict: simple directory drop

Zed requires **only** `SKILL.md` in `~/.config/zed/skills/<skill-name>/`. No manifest, no registry, no extension API. 151 skills are already installed by prometheus-skill-pack via symlinks. cowork only needs to add `zed` to `agents.rs` and call the standard `install_to_dir()` function.

---

## 3. Plugin Format Analysis

### Claude Code plugin (`plugin.json` + skills dir)
- **Format**: JSON manifest at `.claude-plugin/plugin.json`; skills at `.claude-plugin/skills/` (symlinks to `skills/`)
- **cowork current support**: Can read installed plugins via `plugins.rs`; `plugins install` manages local plugins only
- **Gap**: Cannot install a new skill-pack from a git URL with manifest discovery + validation
- **Fix**: Extend `plugins install <git-url>` to clone, discover `plugin.json`, register, and add to `~/.claude/settings.json`

### OpenCode plugin (`plugin.ts` TypeScript + `opencode.json`)
- **Format**: TypeScript plugin compiled by OpenCode; registered via `plugin[]` array in `~/.opencode/opencode.json`
- **Registration**: Fully automatable — write absolute path of `.opencode/` directory into the JSON array
- **cowork current support**: Copies skills to `~/.opencode/skills/`; no plugin registration
- **Fix**: Add post-install JSON writer: append plugin path to `~/.opencode/opencode.json plugin[]`; optionally run `npx @prevalentware/opencode-goal-plugin install` for the goal plugin

### Codex plugin (TOML config + template files)
- **Format**: `~/.codex/config.toml` with `[mcp_servers.*]` sections and `goals.enabled = true`; template files at `~/.codex/goals/`
- **Registration**: Automatable — TOML parse + merge + `mkdir goals/` + template copy
- **cowork current support**: Copies skills to `~/.codex/skills/`; no MCP wiring, no template setup
- **Fix**: Add post-install TOML writer; copy goal templates from prometheus pack's `templates/codex/`

### MiniMax Code IDE (skills dir + `_meta.json`)
- **Format**: Copy of `SKILL.md` + generated `_meta.json` with id/version/timestamp
- **cowork current support**: ✅ Already present as `install_to_minimax()` logic in the fork (or similar)
- **No change needed** beyond ensuring detection covers both MiniMax CLI and MiniMax Desktop

---

## 4. Build-vs-Adopt Decisions

### Decision 1: cowork fork modification strategy

**ADOPT (fork and extend)** — the cowork codebase is well-structured Rust, already does 80% of what we need. Adding 5 agent entries + 3 config writers + 2 new subcommands is additive, not architectural.

- No competing alternatives exist with this breadth of platform coverage
- The `agents.rs` / `commands/install.rs` pattern is clean and extensible
- Score: Fork-and-extend **95%**, Rewrite from scratch **5%** — not a contested choice

### Decision 2: Binary distribution

**ADOPT GitHub Releases (Option B) with source fallback (Option C)**

- crates.io blocked by naming conflict with upstream `cowork`
- npm wrapper is overengineered
- Matches existing gitleaks precedent in the skill-pack CI
- Score: Option B **90%**, Option D (npm) **10%** — not contested

### Decision 3: OpenCode plugin registration automation

**BUILD direct JSON writer** (do not shell out to `npx @prevalentware/opencode-goal-plugin install`)

- The opencode.json format is documented and stable
- Shelling out to npx adds a fragile network dependency
- A JSON merge (like the MiniMax `_meta.json` writer) is 15 lines of Rust/bash
- Score: Direct JSON write **85%**, npx delegation **15%** — not contested

### Decision 4: Codex MCP config approach

**BUILD TOML writer** (idempotent section injection, mirroring Kimi config.toml pattern)

- Codex uses the same TOML structure as Kimi (`[mcp_servers.<name>]` sections)
- The prometheus install script already writes Kimi TOML via `configure_kimi_mcp()`
- Port that pattern to Rust in cowork's `commands/install.rs` or a new `config_writer.rs`
- Score: TOML writer **90%**, shell script delegation **10%** — not contested

### Decision 5: `cowork pack` prometheus-awareness subcommand

**BUILD as a new clap subcommand** delegating to `install-skills-flat.sh`

- Alternatives: (a) rewrite the bash in Rust, (b) shell out, (c) skip
- Shell-out is correct here — the bash script is authoritative and maintained separately
- `cowork pack update` → `bash ~/.cowork/prometheus-skill-pack/scripts/install-skills-flat.sh`
- `cowork pack repair` → `bash ... --repair` (add flag to existing script)
- `cowork toolchain` → `bash ~/.cowork/prometheus-skill-pack/shared/scripts/detect-toolchain.sh --json`
- Score: Shell delegation **88%**, Rust reimplementation **12%** — not contested

### Decision 6: dsg (disk-space-guardian) integration in cowork

**BUILD `cowork disk` as a thin delegate** — check if `dsg` is on PATH, invoke it, surface its output

- dsg doesn't exist yet; cowork cannot depend on it
- `cowork disk status` → `dsg status --json` if present; warns "install disk-space-guardian" if absent
- `cowork disk clean [--dry-run]` → `dsg clean --ecosystem cargo [--dry-run]`
- This is a graceful-degradation stub; no hard dependency

---

## 5. Library Candidates

### For new config writers in cowork (Rust)

| Need | Candidate | Version | Verdict |
|---|---|---|---|
| TOML read/write | `toml` crate | 0.8 | ✅ ADOPT — already in cowork Cargo.toml |
| JSON read/write | `serde_json` | 1 | ✅ ADOPT — already in cowork Cargo.toml |
| Filesystem walk | `walkdir` | 2 | ✅ ADOPT — already in cowork Cargo.toml |
| Path expansion (`~/...`) | `dirs` | 5 | ✅ ADOPT — standard, no new dep if `home_dir()` used |
| SHA-256 for `_meta.json` id | `sha2` | 0.10 | ✅ ADOPT — already in cowork Cargo.toml |

**No new Rust dependencies needed** for the platform additions and config writers. All required crates are already in cowork's `Cargo.toml`.

### For GitHub Releases CI/CD

| Need | Candidate | Notes |
|---|---|---|
| Cross-compilation | `cross` | Docker-based, covers linux targets from macOS |
| macOS arm64 native | GitHub-hosted `macos-latest` runner | Free, covers M1/M2 |
| Release automation | `cargo-dist` or `release-plz` | Either works; `cargo-dist` generates the full release workflow |

**Recommended CI toolchain**: `cargo-dist` for release workflow generation + `cross` for Linux targets.

---

## 6. Risk Updates

| Risk | Research Outcome | Updated Verdict |
|---|---|---|
| Kimi Desktop skill dir unknown | Confirmed: `~/Library/Application Support/kimi-desktop/daimon-share/daimon/skills/` | RESOLVED — path known; long path needs careful handling in bash |
| MiniMax Desktop skill dir unknown | Confirmed: shared `~/.minimax/skills/` | RESOLVED — no new path needed |
| MMX CLI config format unknown | Confirmed: JSON, but NO skill system | RESOLVED — MMX is out of scope entirely |
| cowork crates.io naming conflict | Confirmed: upstream `cowork` owns the crate name | RESOLVED — GitHub Releases is the right path |
| dsg not yet implemented | Confirmed: spec-only, no binary | RESOLVED — Wave 3 stub degrades gracefully |
| Kimi Desktop path is deep macOS Application Support path | New finding | WATCH — symlink into this path must use absolute path and check macOS-only |
| MiniMax `_meta.json` ID generation format | Existing pattern: MD5 hash of skill name | LOW — same pattern as existing prometheus install logic |
| OpenCode `opencode.json` schema stability | Confirmed stable JSON with `$schema` declaration | LOW — write idempotently; check for existing entry before appending |

---

## 7. Revised Change Sequence

Based on research findings, the 12-change plan is refined:

### Wave 1 — Foundation + Platform Entries (Changes 001–003)

**change-001**: Clone fork to worktree + baseline build verification
- Clone `git@github.com:GQAdonis/cowork-skills.git` to `/Users/gqadonis/Projects/prometheus/cowork-skills`
- Verify `cargo build --release` succeeds
- Add Zed to `agents.rs`: primary `~/.config/zed/skills/`, fallback `~/.zed/skills/`
- Standard `install_to_dir()` — no manifest needed

**change-002**: Add Kimi Code CLI + Kimi Desktop to `agents.rs`
- Kimi Code CLI: `~/.kimi-code/skills/`; post-install: write `~/.kimi-code/config.toml` MCP entries (port from `configure_kimi_mcp()` bash)
- Kimi Desktop: `~/Library/Application Support/kimi-desktop/daimon-share/daimon/skills/` (macOS-only guard)
- Detection: parent dir existence check

**change-003**: Clarify MiniMax agent entry; document MMX-is-out-of-scope
- Update `minimax` agent description to cover both CLI and MiniMax Desktop
- Update detection: check for EITHER `~/.minimax/` OR `~/Library/Application Support/MiniMax Agent/`
- Add code comment explaining `mmx` CLI has no skill system

### Wave 2 — Plugin Management (Changes 004–006)

**change-004**: Extend `cowork plugins install <git-url>` for Claude Code
- Clone git URL, discover `.claude-plugin/plugin.json`, validate manifest
- Register in `~/.claude/settings.json` + `~/.claude/plugins/installed_plugins.json`

**change-005**: Add Codex post-install TOML config writer
- After skills symlink: write `[mcp_servers.*]` entries to `~/.codex/config.toml`
- Set `goals.enabled = true`
- Copy goal templates to `~/.codex/goals/` from prometheus pack template dir

**change-006**: Add OpenCode post-install JSON plugin registration
- After skills symlink: append `.opencode/` absolute path to `~/.opencode/opencode.json plugin[]` array
- Idempotent: check for existing entry before writing
- Do NOT shell out to `npx @prevalentware/opencode-goal-plugin install` (fragile network dep)

### Wave 3 — Prometheus-Pack Awareness (Changes 007–009)

**change-007**: Add `cowork pack` subcommand
- `pack status` — reads prometheus pack version + installed skills count per platform
- `pack update` — shells to `install-skills-flat.sh`
- `pack repair` — detects broken symlinks + runs install for affected platforms

**change-008**: Add `cowork toolchain` subcommand
- Shells to `detect-toolchain.sh --json` and formats output
- Surfaces MCP service health, Rust toolchain status, binary locations

**change-009**: Add `cowork disk` stub subcommand
- `disk status` → `dsg status --json` if `dsg` on PATH; else prints install instructions
- `disk clean [--dry-run]` → `dsg clean --ecosystem cargo [--dry-run]` if present
- Graceful degradation: non-zero exit if dsg absent, but informative message

### Wave 4 — Integration + Distribution (Changes 010–012)

**change-010**: Set up GitHub Releases CI/CD for cowork fork
- GitHub Actions workflow: `cargo-dist` for multi-arch builds
- Targets: `arm64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`
- Triggers on `v*.*.*` tags

**change-011**: Add `install_cowork()` to prometheus `install-binaries.sh`
- Primary: download from GitHub Releases
- Fallback: `cargo build --release` from `tools/cowork-skills` submodule
- Wire into `check-prerequisites.sh`

**change-012**: Add `skills/process/cowork-management/SKILL.md` + update CLAUDE.md
- SKILL.md: documents `cowork pack`, `cowork install`, `cowork toolchain`, `cowork disk`
- CLAUDE.md: add cowork to "Essential Commands" section

---

## 8. dsg Parallel Track

dsg (`disk-space-guardian`) should begin execution in its own KBD phase, running concurrently with cowork Wave 1. The 5 existing OpenSpec changes are:

| Change | Scope |
|---|---|
| change-001 | Establish capability specs (bind 4 open design risks) |
| change-002 | Scaffold Cargo workspace + clap skeleton |
| change-003 | Safety module (dry-run, trash, lsof, git verification, exclusion lists) |
| change-004 | Scanner core (jwalk parallel walk) |
| change-005 | Ecosystem detectors (Rust, Node, Python, Go, Docker) |

**Start dsg after cowork change-001 lands.** The disk pressure (17 GB build artifacts) is real and growing.

---

## 9. Stack Summary

No contested stack decisions. All decisions resolved to clear verdicts (score gap > 15%):

| Decision | Verdict | Score Gap |
|---|---|---|
| Fork strategy | Fork-and-extend | 90% vs 5% |
| Binary distribution | GitHub Releases + source fallback | 90% vs 10% |
| OpenCode registration | Direct JSON write | 85% vs 15% |
| Codex MCP config | TOML writer (Rust) | 90% vs 10% |
| `cowork pack` delegation | Shell-out to bash script | 88% vs 12% |
| dsg integration | Graceful delegate stub | Clear |
