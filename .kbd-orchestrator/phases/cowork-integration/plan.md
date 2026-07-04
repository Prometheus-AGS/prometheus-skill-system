# Plan — cowork-integration

_Generated: 2026-07-03 | KBD Plan stage | OpenSpec backend_

---

## Executive Summary

This phase delivers two parallel workstreams across 24 changes (12 cowork + 5 dsg-cli-foundation + 7 integration):

1. **cowork fork extension** (12 changes, 4 waves) — extend `GQAdonis/cowork-skills` with Zed/Kimi/MiniMax platform support, Codex + OpenCode plugin management, prometheus-pack awareness, and GitHub Releases binary distribution.
2. **disk-space-guardian full implementation** (5 changes, 4 rounds) — implement `dsg` CLI from scratch: Cargo workspace, safety module, scanner core, ecosystem detectors, agentskills.io SKILL.md, Claude Code plugin.json, marketplace entry, OpenCode npm config, Codex config.toml.
3. **Integration layer** (7 changes) — submodule wiring for both repos into prometheus-skill-pack, SKILL.md for cowork management, and marketplace entries.

---

## Submodule Decision

**DECISION: Both repos become git submodules of prometheus-skill-pack.**

Placement follows the existing two-tier pattern from `docs/SUBMODULES.md`:

| Repo | Submodule Path | Rationale |
|---|---|---|
| `git@github.com:GQAdonis/disk-space-guardian.git` | `tools/disk-space-guardian` | Infrastructure layer (alongside liter-llm, prometheus-knowledge-rs, surreal-memory-server) |
| `git@github.com:GQAdonis/cowork-skills.git` | `tools/cowork-skills` | CLI tool, not an agent framework — infrastructure layer |

**Rationale for `tools/` over `skills/imported/`:**
- `skills/imported/` is for **agent framework skills** (artifact-refiner, sycophancy-correction, prometheus-entity-management) that have agentskills.io SKILL.md files and are distributed as skills
- `tools/` is for **infrastructure CLIs and services** (liter-llm, prometheus-knowledge-rs, surreal-memory-server) that are binaries/services
- Both `cowork` and `dsg` are CLIs — they belong in `tools/`
- The agentskills.io SKILL.md for each lives in the **skill-pack** (`skills/process/cowork-management/SKILL.md`, `skills/devops/disk-space-guardian/SKILL.md`) once they're ready, and will NOT live in the tool itself (the tool directory contains the source code, not the skill)

---

## disk-space-guardian — Full Implementation Scope

### Current State: Spec-only (zero code)

The dsg project (`/Users/gqadonis/Projects/prometheus/disk-space-guardian`) has:
- Extensive planning docs (`docs/README.md`, `AGENTS.md`, `CLAUDE.md`)
- `.kbd-orchestrator/` phase tracking state
- `.claude/`, `.codex/`, `.kimi/`, `.opencode/` openspec skill skeletons
- **Zero Rust code** — no Cargo.toml, no `src/`, no binary

### Phase 1 CLI (5 changes covering full MVP)

The 5 OpenSpec changes in dsg-cli-foundation cover:

1. **Establish capability specs** — decompose docs/README.md into OpenSpec capability specs; bind 4 open design decisions (lsof TOCTOU, symlink handling, trash failure semantics, mtime anchoring)
2. **Scaffold Cargo workspace + CLI** — `Cargo.toml` workspace, `dsg` binary crate, clap 4 skeleton, TOML config loading from `~/.config/dsg/config.toml`
3. **Safety module** — dry-run/execute split; trash-crate integration (never `rm`); lsof+git-status activity verification; exclusion lists; min-age guards
4. **Scanner core** — parallel filesystem walk (jwalk); size+staleness reporting; ecosystem-detector trait; human-readable + JSON output; `dsg scan` command
5. **Ecosystem detectors + end-to-end clean** — detectors for Rust, Node, Python, Go, Docker, Xcode, Homebrew; wire `dsg clean` through safety module; integration tests

**Out of scope for Phase 1 (deferred):** TUI ratatui interface, Windows support, `dsg watch`, `dsg schedule`, `dsg config`, MCP server, knowledge wiki.

### Plugin/Skill artifacts to add AFTER Phase 1 CLI ships

These 7 integration changes come AFTER the dsg CLI is functional:

| Artifact | File | Notes |
|---|---|---|
| agentskills.io SKILL.md | `skills/devops/disk-space-guardian/SKILL.md` | In prometheus-skill-pack, not in dsg repo |
| Claude Code plugin.json | `tools/disk-space-guardian/.claude-plugin/plugin.json` | In dsg repo |
| marketplace.json entry | Add to `marketplace/marketplace.json` | In prometheus-skill-pack |
| OpenCode package.json | `tools/disk-space-guardian/.opencode/package.json` | In dsg repo |
| Codex config.toml | `tools/disk-space-guardian/.codex/config.toml` | In dsg repo (may already exist) |
| Marketplace listing wire | `marketplace/marketplace.json` | Add dsg skill entry |
| CLAUDE.md + install-binaries.sh | prometheus-skill-pack root | Document dsg in Essential Commands |

---

## Dependency Map

```
Wave 0 (parallel start):
  cowork-change-001 ─── clone + Zed agent
  dsg-change-001 ────── establish capability specs (runs in parallel)

Wave 1:
  cowork-change-002 ─── Kimi + Kimi Desktop agents
  cowork-change-003 ─── MiniMax detection + MMX doc
  dsg-change-002 ────── scaffold Cargo + clap

Wave 2:
  cowork-change-004 ─── Claude Code plugins install from git URL (parallel)
  cowork-change-005 ─── Codex TOML config writer (parallel)
  cowork-change-006 ─── OpenCode JSON plugin registration (parallel)
  dsg-change-003 ────── safety module (parallel with cowork Wave 2)
  dsg-change-004 ────── scanner core (parallel with cowork Wave 2)

Wave 3:
  cowork-change-007 ─── cowork pack subcommand
  cowork-change-008 ─── cowork toolchain subcommand
  cowork-change-009 ─── cowork disk stub
  dsg-change-005 ────── ecosystem detectors + clean integration (needs 003 + 004)

Wave 4 (integration, cowork):
  cowork-change-010 ─── GitHub Releases CI/CD (parallel)
  cowork-change-011 ─── install_cowork() in install-binaries.sh (parallel)
  cowork-change-012 ─── cowork SKILL.md + CLAUDE.md (parallel)

Wave 5 (integration, dsg — AFTER dsg Phase 1 is done):
  dsg-int-001 ─────── dsg submodule add to prometheus-skill-pack tools/
  dsg-int-002 ─────── dsg plugin.json + marketplace.json entry
  dsg-int-003 ─────── dsg SKILL.md in skills/devops/disk-space-guardian/
  dsg-int-004 ─────── dsg OpenCode + Codex plugin artifacts
  dsg-int-005 ─────── install_dsg() in install-binaries.sh + CLAUDE.md
```

---

## Change Roster

### Workstream A — cowork fork extensions (12 changes)

**Wave 1 — Foundation + Platform Entries**

#### change-cowork-001: Clone fork + Zed agent entry
- **Worktree**: `/Users/gqadonis/Projects/prometheus/cowork-skills`
- **Work**: Clone `git@github.com:GQAdonis/cowork-skills.git`; verify `cargo build --release`; add `Zed` agent entry to `agents.rs`
  - Primary path: `~/.config/zed/skills/`; fallback: `~/.zed/skills/`
  - Detection: check either parent dir exists
  - Install method: `install_to_dir()` — no manifest needed for Zed
- **Tests**: `cargo test`; manual `cowork install --agent zed` smoke test
- **Recommended agent**: general-purpose (Rust implementation)
- **Goal coverage**: G-01, G-02

#### change-cowork-002: Kimi Code CLI + Kimi Desktop agents
- **Work**: Add two agent entries to `agents.rs`
  - `kimi-code`: path `~/.kimi-code/skills/`; post-install writes `~/.kimi-code/config.toml` MCP entries (port from prometheus `configure_kimi_mcp()` bash)
  - `kimi-desktop`: path `~/Library/Application Support/kimi-desktop/daimon-share/daimon/skills/`; macOS-only guard via `#[cfg(target_os = "macos")]`; detection: parent dir existence; use `dirs::home_dir()` + `Application Support` constant
- **Tests**: detection logic unit tests; `cargo test agents`
- **Recommended agent**: general-purpose
- **Goal coverage**: G-02

#### change-cowork-003: MiniMax detection update + MMX documentation
- **Work**:
  - Update `minimax` entry detection: check EITHER `~/.minimax/` OR `~/Library/Application Support/MiniMax Agent/`
  - Add doc comment in `agents.rs` explaining `mmx` media CLI has no skill system
  - Update `README.md` to clarify MiniMax Desktop coverage and remove any "MMX CLI" promise
- **Tests**: detection unit tests
- **Recommended agent**: general-purpose
- **Goal coverage**: G-02

**Wave 2 — Plugin Management (parallelizable)**

#### change-cowork-004: Claude Code plugin install from git URL
- **Work**: Extend `commands/plugins.rs` → `plugins install <git-url>` flow
  - Clone git URL to temp dir
  - Discover `.claude-plugin/plugin.json`; validate required fields (`name`, `version`, `skills`, `license`)
  - Register in `~/.claude/settings.json` `plugins` key (JSON merge, idempotent)
  - Write entry to `~/.claude/plugins/installed_plugins.json`
  - Report installed skill paths
- **Tests**: mock git clone; unit tests for JSON merge; schema validation tests
- **Recommended agent**: general-purpose
- **Goal coverage**: G-04

#### change-cowork-005: Codex TOML config writer + templates
- **Work**: After `cowork install --agent codex`:
  - Parse/merge `~/.codex/config.toml` (add `[mcp_servers.*]` sections for skill pack MCPs)
  - Set `goals.enabled = true`
  - Copy goal templates from prometheus pack's `templates/codex/` to `~/.codex/goals/`
  - Port TOML merge logic from prometheus `configure_kimi_mcp()` bash into Rust (use `toml` crate already in Cargo.toml)
- **Tests**: idempotent TOML write test; template copy test
- **Recommended agent**: general-purpose
- **Goal coverage**: G-04

#### change-cowork-006: OpenCode JSON plugin registration
- **Work**: After `cowork install --agent opencode`:
  - Append `.opencode/` absolute path to `~/.opencode/opencode.json` `plugin[]` array
  - Idempotent: check for existing entry before appending (JSON read → filter → append → write)
  - Do NOT shell out to `npx @prevalentware/opencode-goal-plugin install`
  - Also ensure `@opencode-ai/plugin`, `@opencode-ai/sdk`, `zod` are listed in `.opencode/package.json` for the pack
- **Tests**: idempotent JSON writer test; existing entry de-duplication test
- **Recommended agent**: general-purpose
- **Goal coverage**: G-04

**Wave 3 — Prometheus-Pack Awareness**

#### change-cowork-007: `cowork pack` subcommand
- **Work**: New clap subcommand `pack` with subcommands:
  - `pack status` — reads prometheus pack version from `~/.cowork/prometheus-skill-pack/package.json`; counts installed skills per platform; prints summary table
  - `pack update` — shells to `bash ~/.cowork/prometheus-skill-pack/scripts/install-skills-flat.sh`
  - `pack repair` — detects broken symlinks (`target_exists() == false`); runs install for affected platforms
  - Pack location config: `PROMETHEUS_SKILL_PACK` env var or `~/.cowork/prometheus-skill-pack/` default
- **Tests**: status command unit tests; broken symlink detection test
- **Recommended agent**: general-purpose
- **Goal coverage**: G-03

#### change-cowork-008: `cowork toolchain` subcommand
- **Work**: New clap subcommand `toolchain`:
  - `toolchain status` — shells to `bash ~/.cowork/prometheus-skill-pack/shared/scripts/detect-toolchain.sh --json`; pretty-prints Rust toolchain status, MCP service health, binary locations
  - `toolchain check` — returns exit code 0 if all required tools present, 1 otherwise (usable in CI)
  - `toolchain install <tool>` — stub that prints install instructions per tool
- **Tests**: JSON output parsing test; exit code test
- **Recommended agent**: general-purpose
- **Goal coverage**: G-03

#### change-cowork-009: `cowork disk` stub subcommand
- **Work**: New clap subcommand `disk`:
  - `disk status` → runs `dsg status --json` if `dsg` on PATH; else prints actionable error message with install instructions
  - `disk scan [--deep] [--ecosystem <name>]` → delegates to `dsg scan`
  - `disk clean [--dry-run] [--ecosystem <name>]` → delegates to `dsg clean`
  - All subcommands: graceful degradation if `dsg` absent (non-zero exit with clear message)
  - Embed install URL: `https://github.com/GQAdonis/disk-space-guardian/releases/latest`
- **Tests**: presence/absence detection logic; command passthrough test
- **Recommended agent**: general-purpose
- **Goal coverage**: G-03, G-05

**Wave 4 — Distribution + Documentation (parallelizable)**

#### change-cowork-010: GitHub Releases CI/CD workflow
- **Work**: In `tools/cowork-skills` repo:
  - Add `.github/workflows/release.yml` using `cargo-dist`
  - Targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`
  - Triggers on `v*.*.*` tags
  - Add `.github/workflows/ci.yml` for PR builds (`cargo check`, `clippy`, `test`)
  - Configure `cargo-dist` in `Cargo.toml` `[dist]` section
- **Tests**: workflow syntax validation (`actionlint`); verify release draft on tag push
- **Recommended agent**: devops-engineer
- **Goal coverage**: G-05

#### change-cowork-011: `install_cowork()` in prometheus install-binaries.sh
- **Work**: In prometheus-skill-pack:
  - Add `tools/cowork-skills` as git submodule: `git submodule add git@github.com:GQAdonis/cowork-skills.git tools/cowork-skills`
  - Add `install_cowork()` function to `shared/scripts/install-binaries.sh` (or create if absent):
    - Primary: detect OS/arch; download from GitHub Releases API latest tag
    - Fallback: `cargo build --release` in `tools/cowork-skills/`
    - Install binary to `~/.local/bin/cowork` (or `~/.local/bin/co` alias)
  - Wire into `check-prerequisites.sh` (add cowork to required binaries)
- **Tests**: test install function with mock GitHub API response; test fallback path
- **Recommended agent**: devops-engineer
- **Goal coverage**: G-05

#### change-cowork-012: cowork SKILL.md + CLAUDE.md documentation
- **Work**: In prometheus-skill-pack:
  - Create `skills/process/cowork-management/SKILL.md` (agentskills.io compliant):
    ```yaml
    name: cowork-management
    version: 1.0.0
    license: MIT
    description: >
      Install, update, and manage AI coding agent skills across 20+ platforms
      using the cowork CLI. Covers Claude Code, Codex, OpenCode, Kimi, Zed,
      Cursor, GitHub Copilot, and more. Also manages prometheus-skill-pack
      updates, toolchain health, and disk space via dsg delegation.
    metadata:
      category: process
      tags: [cli, skill-management, cowork, install, platform]
    ```
  - Add to CLAUDE.md `## Essential Commands` section: `cowork pack update`, `cowork toolchain status`, `cowork disk scan`
  - Update `marketplace/marketplace.json` with cowork-management skill entry
  - Run `npm run validate:strict skills/process/cowork-management`
- **Recommended agent**: general-purpose
- **Goal coverage**: G-05

---

### Workstream B — disk-space-guardian full implementation (5 changes)

_Runs concurrently starting after cowork-change-001. Executed in dsg's own worktree._

#### change-dsg-001: Establish capability specs
- **Work**: In `/Users/gqadonis/Projects/prometheus/disk-space-guardian`:
  - Read `docs/README.md` §3.2 in full; decompose into OpenSpec capability spec files:
    - `openspec/specs/cli.md` — command surface and UX contract
    - `openspec/specs/config.md` — TOML config schema (`~/.config/dsg/config.toml`)
    - `openspec/specs/safety.md` — safety rules (dry-run, trash, lsof, git, min-age)
    - `openspec/specs/scanner.md` — filesystem scan algorithm and output format
  - Bind 4 open design decisions in `docs/decisions.md`:
    - D-01: lsof TOCTOU — use lsof snapshot, warn but proceed if race detected
    - D-02: symlink handling — scan target, not symlink source; exclude `node_modules` symlinks
    - D-03: trash failure semantics — abort item, log warning, continue batch
    - D-04: mtime vs atime anchoring — use mtime (more reliable across filesystems)
- **Tests**: N/A (spec-only change)
- **Recommended agent**: general-purpose
- **Goal coverage**: Pre-condition for all dsg changes

#### change-dsg-002: Scaffold Cargo workspace + CLI
- **Work**:
  - Create `Cargo.toml` (workspace with single crate `dsg`)
  - Create `dsg/Cargo.toml` with dependencies: `clap@4`, `anyhow@1`, `toml@0.8`, `serde@1`, `tracing@0.1`, `tracing-subscriber@0.3`
  - Create `dsg/src/main.rs` with clap skeleton: `scan`, `clean`, `caches`, `status` subcommands (all stubbed)
  - Create `~/.config/dsg/config.toml` schema loader with `#[derive(Deserialize, Default)]`
  - `cargo build --release` must succeed with all subcommands showing `--help`
  - Add `.github/workflows/ci.yml`: `cargo check`, `clippy -- -D warnings`, `cargo test`
- **Tests**: `cargo test`; `dsg --help` smoke test; `dsg scan --help` subcommand help
- **Recommended agent**: general-purpose
- **Goal coverage**: G-01 (dsg), G-05 (dsg)

#### change-dsg-003: Safety module
- **Work**:
  - `dsg/src/safety.rs`: `SafetyEngine` struct
    - `dry_run: bool` flag (default true; require `--force` or explicit `--execute` to actually delete)
    - `verify_activity(path: &Path) -> ActivityCheck`: runs `lsof +D <path>` (with timeout); runs `git status` if git repo detected
    - `move_to_trash(path: &Path) -> Result<()>`: uses `trash` crate (not `std::fs::remove_*`)
    - `should_exclude(path: &Path, config: &Config) -> bool`: checks exclusion list patterns from config
    - `age_guard(path: &Path, min_age_secs: u64) -> bool`: checks mtime against threshold
  - Integrate with `dsg clean --dry-run` (default) and `dsg clean --force` (execute)
  - Add exclusion list to config TOML: `exclude_paths = ["~/.cargo/registry", "~/Library/Caches/com.apple.*"]`
- **Tests**: unit tests for activity check, age guard, exclusion matching; mock lsof output
- **Recommended agent**: general-purpose
- **Goal coverage**: Safety requirement from dsg goals

#### change-dsg-004: Scanner core
- **Work**:
  - `dsg/src/scanner.rs`: parallel filesystem scan
    - Use `jwalk` crate for parallel walk; `walkdir` as safe single-threaded fallback
    - `ScanResult` struct: path, size_bytes, last_accessed, last_modified, entry_type (file/dir)
    - `scan_directory(root: &Path, options: &ScanOptions) -> Vec<ScanResult>`: returns sorted by size desc
    - `report_human(results: &[ScanResult])`: tabular output with `humansize`
    - `report_json(results: &[ScanResult])`: serde_json serialized output (for `dsg scan --json`)
  - Implement `EcosystemDetector` trait:
    ```rust
    trait EcosystemDetector {
        fn name(&self) -> &str;
        fn detect(path: &Path) -> Vec<PathBuf>;
        fn describe(path: &Path) -> String;
    }
    ```
  - Integrate with `dsg scan`, `dsg scan --deep`, `dsg scan --json`
- **Tests**: unit tests for scan result sorting; JSON output roundtrip; size calculation
- **Recommended agent**: general-purpose
- **Goal coverage**: Core scanner for dsg

#### change-dsg-005: Ecosystem detectors + clean integration
- **Work**: Implement `EcosystemDetector` for each ecosystem:
  - `RustDetector`: finds `target/` dirs; reads `Cargo.toml` to confirm; checks `~/.cargo/registry/`, `~/.cargo/git/`
  - `NodeDetector`: finds `node_modules/` dirs; reads `package.json` to confirm; checks `~/.npm/`, `~/.yarn/`
  - `PythonDetector`: finds `.venv/`, `__pycache__/`; checks `~/.cache/pip/`, `~/.pyenv/`
  - `GoDetector`: finds `~/.cache/go-build/`, `~/go/pkg/mod/`
  - `DockerDetector`: runs `docker system df --format json` (graceful if docker absent)
  - `XcodeDetector`: finds `~/Library/Developer/Xcode/DerivedData/`, `~/Library/Caches/com.apple.dt.Xcode/`
  - `HomebrewDetector`: runs `brew cleanup --dry-run --prune=all` (graceful if brew absent)
  - Wire all detectors into `dsg scan --ecosystem <name>` and `dsg clean --ecosystem <name>`
  - Full end-to-end integration test: scan prometheus-skill-pack's `tools/` directory; verify 17 GB worth of target dirs detected
- **Tests**: per-detector unit tests; graceful degradation when tool absent; integration test on real tools/ dir
- **Recommended agent**: general-purpose
- **Goal coverage**: Complete dsg Phase 1

---

### Workstream C — Integration layer (7 changes)

_Runs AFTER dsg Phase 1 (all 5 dsg changes complete) and AFTER cowork Wave 4._

#### change-int-001: Add dsg as git submodule
- **Work**: In prometheus-skill-pack:
  - `git submodule add git@github.com:GQAdonis/disk-space-guardian.git tools/disk-space-guardian`
  - Update `docs/SUBMODULES.md`: add dsg entry with purpose + pin policy
  - Update `.gitmodules` (automatic via git submodule add)
- **Tests**: `git submodule status` shows clean; `git submodule update` succeeds

#### change-int-002: dsg plugin.json + marketplace listing
- **Work**:
  - Create `tools/disk-space-guardian/.claude-plugin/plugin.json`:
    ```json
    {
      "name": "disk-space-guardian",
      "version": "1.0.0",
      "description": "Intelligent disk space management CLI for dev workstations",
      "author": { "name": "Travis James" },
      "license": "MIT",
      "skills": [],
      "compatibility": { "platforms": ["claude-code", "codex", "opencode", "kimi-code", "cursor"] }
    }
    ```
  - Add dsg entry to `marketplace/marketplace.json`:
    ```json
    {
      "name": "disk-space-guardian",
      "description": "dsg CLI: safe, intelligent build cache cleanup for Rust, Node, Python, Go, Docker",
      "source": "./tools/disk-space-guardian",
      "version": "1.0.0",
      "tags": ["disk", "cache", "cleanup", "rust", "devops"],
      "category": "devops"
    }
    ```

#### change-int-003: dsg agentskills.io SKILL.md
- **Work**: Create `skills/devops/disk-space-guardian/SKILL.md` with full 8-section body:
  - Frontmatter: name, version, license, tags, triggers (keywords: `/dsg`, "disk space", "clean caches")
  - Sections: Quick Start, Safety First, Ecosystem Detection, Activity Verification, Retention Policies, Automation Setup, Knowledge Logging, Troubleshooting
  - References: link to `references/` directory in dsg repo (after merge to tools/)
  - Run `npm run validate:strict skills/devops/disk-space-guardian`
- **Recommended agent**: general-purpose

#### change-int-004: dsg OpenCode + Codex plugin artifacts
- **Work**:
  - Create `tools/disk-space-guardian/.opencode/package.json`:
    ```json
    {
      "name": "disk-space-guardian-opencode",
      "version": "1.0.0",
      "private": true,
      "dependencies": { "@opencode-ai/plugin": "^1.15.0", "@opencode-ai/sdk": "^1.15.0", "zod": "^3.23.0" }
    }
    ```
  - Verify `tools/disk-space-guardian/.codex/config.toml` exists (created by dsg-change-002) or create with dsg MCP stub (to be filled when MCP server lands in Phase 3)
  - Add `CODEX_PLUGIN=true` marker in `.codex/config.toml` comments

#### change-int-005: `install_dsg()` in install-binaries.sh + CLAUDE.md
- **Work**:
  - Add `install_dsg()` to `shared/scripts/install-binaries.sh`:
    - Primary: download from GitHub Releases latest `dsg` binary for current OS/arch
    - Fallback: `cargo build --release` in `tools/disk-space-guardian/`
    - Install to `~/.local/bin/dsg`
  - Update `CLAUDE.md` `## Essential Commands` section: add dsg scan/clean commands
  - Update `detect-toolchain.sh` to include dsg in its binary health check

#### change-int-006: cowork submodule add
- **Work**: In prometheus-skill-pack:
  - `git submodule add git@github.com:GQAdonis/cowork-skills.git tools/cowork-skills`
  - Update `docs/SUBMODULES.md`: add cowork entry
  - Wire `install_cowork()` (from change-cowork-011) to reference `tools/cowork-skills` as the source fallback

#### change-int-007: Validate + CI update
- **Work**:
  - Run full validate: `npm run validate:strict`
  - Update `.github/workflows/validate.yml` to:
    - `git submodule update --init tools/disk-space-guardian tools/cowork-skills` in checkout step
    - Add `cargo check` step for each tool submodule
  - Run `npm run build` to rebuild marketplace symlinks
  - Smoke test: `cowork pack status` shows prometheus-skill-pack version; `dsg --version` returns 1.0.0

---

## Summary Table

| Change ID | Workstream | Wave | Parallel | Goal | Agent |
|---|---|---|---|---|---|
| change-cowork-001 | A | 0 | No (blocks 002) | G-01, G-02 | general-purpose |
| change-dsg-001 | B | 0 | Yes (with cowork-001) | dsg pre-cond | general-purpose |
| change-cowork-002 | A | 1 | No | G-02 | general-purpose |
| change-cowork-003 | A | 1 | After 002 | G-02 | general-purpose |
| change-dsg-002 | B | 1 | After dsg-001 | dsg core | general-purpose |
| change-cowork-004 | A | 2 | Yes | G-04 | general-purpose |
| change-cowork-005 | A | 2 | Yes | G-04 | general-purpose |
| change-cowork-006 | A | 2 | Yes | G-04 | general-purpose |
| change-dsg-003 | B | 2 | Yes | dsg safety | general-purpose |
| change-dsg-004 | B | 2 | Yes | dsg scanner | general-purpose |
| change-cowork-007 | A | 3 | No | G-03 | general-purpose |
| change-cowork-008 | A | 3 | After 007 | G-03 | general-purpose |
| change-cowork-009 | A | 3 | After 008 | G-03, G-05 | general-purpose |
| change-dsg-005 | B | 3 | After dsg-003+004 | dsg complete | general-purpose |
| change-cowork-010 | A | 4 | Yes | G-05 | devops-engineer |
| change-cowork-011 | A | 4 | Yes | G-05 | devops-engineer |
| change-cowork-012 | A | 4 | Yes | G-05 | general-purpose |
| change-int-001 | C | 5 | No | G-05 | general-purpose |
| change-int-002 | C | 5 | After int-001 | G-04, G-05 | general-purpose |
| change-int-003 | C | 5 | After int-001 | G-05 | general-purpose |
| change-int-004 | C | 5 | After int-001 | G-04 | general-purpose |
| change-int-005 | C | 5 | After int-001 | G-05 | devops-engineer |
| change-int-006 | C | 5 | After int-001 | G-05 | general-purpose |
| change-int-007 | C | 5 | After all | G-05 | general-purpose |

**Total changes: 24** (12 cowork + 5 dsg + 7 integration)

---

## Goal → Change Mapping

| Goal | Changes |
|---|---|
| G-01: Architecture assessment + integration plan | change-cowork-001 (worktree clone, baseline build) |
| G-02: Platform support (Zed, Kimi, MiniMax) | change-cowork-001, -002, -003 |
| G-03: Pack awareness + toolchain + repair | change-cowork-007, -008, -009 |
| G-04: Claude Code + Codex + OpenCode plugin mgmt | change-cowork-004, -005, -006, change-int-002, -004 |
| G-05: Integration pipeline + documentation | change-cowork-010, -011, -012, change-int-001 through -007 |

---

## Risk Register

| Risk | Mitigation |
|---|---|
| Kimi Desktop path expansion fails | Use `dirs::home_dir()` + Path::join; never string concat for paths |
| TOML idempotent write corrupts config | Read → deserialize → merge → serialize → write; never overwrite directly |
| dsg-change-003 (safety) lsof unavailable on some macOS | Graceful skip: if `lsof` not on PATH, skip activity check and emit warning |
| cowork crates.io conflict | GitHub Releases is primary; cargo build fallback; never `cargo publish` from this fork |
| dsg 17 GB test takes too long in CI | Limit integration test to a 1 GB subset; full scan only in manual runs |
| Symlinks to Kimi Desktop deep path fail on macOS | Test with `std::fs::symlink`; fall back to `std::fs::copy` if symlink fails |

---

## First Change to Apply

```
/kbd-apply change-cowork-001
```

After that: `/kbd-apply change-dsg-001` can run in parallel in the dsg worktree.

---

_Plan complete. Execute with `/kbd-apply change-cowork-001` to begin._
