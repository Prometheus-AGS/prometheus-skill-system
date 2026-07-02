# Plan: machine-installation-2026-05-25
**Date**: 2026-05-25  
**Phase**: machine-installation-2026-05-25  
**Backend**: native-kbd (migrated from OpenSpec 2026-07-02)  
**Assessment**: `.kbd-orchestrator/phases/assess/machine-installation-assessment-2026-05-25.md`

---

## Goal

Fully set up this machine for all skills in the pack:
- All project binaries built and in PATH
- forge-mcp and pk-mcp running as launchd agents on startup
- Skills installed to all supported AI tool platforms
- MCP servers wired into opencode, codex, and zed
- A `prometheus setup` subcommand that orchestrates all of the above interactively

---

## Pre-conditions (Verified in Assessment)

| Service/Binary | Status | Action |
|---|---|---|
| `surreal-memory-server` (Docker, 23001) | ✅ Running | SKIP |
| `surrealdb` (Docker, 28000) | ✅ Running | SKIP |
| `openai-proxy` (launchd, 8181) | ✅ Running | SKIP |
| `sycophancy-correction` (`/usr/local/bin/`) | ✅ Installed | SKIP |
| `prometheus-rust-auditor` (`~/.cargo/bin/`) | ✅ Installed | SKIP |
| Claude Code skills (394) | ✅ Installed | SKIP |

---

## Change Order

### change-001 — build-and-install-binaries
**Effort**: 30–45 min  
**Gaps closed**: G-BIN-1, G-BIN-2, G-SVC-3 (liter-llm PATH)  
**Agent**: claude-code  

Build all missing binaries and install every project binary to `~/.local/bin/`:

| Binary | Build command | Install target |
|---|---|---|
| `pk-mcp` | `cargo build --release -p pk-mcp` in `tools/prometheus-knowledge/` | `~/.local/bin/pk-mcp` |
| `liter-llm` | `cargo build --release` in `tools/liter-llm/` | `~/.local/bin/liter-llm` |
| `forge-mcp` | `cargo build --release -p forge-mcp` in `tools/forge-rs/` | `~/.local/bin/forge-mcp` |
| `prometheus` (CLI) | already built at `tools/prometheus-cli/target/release/prometheus` | `~/.local/bin/prometheus` |
| `forge` | already built at `tools/forge-rs/target/release/forge` | `~/.local/bin/forge` |

**Acceptance**:
- `which pk-mcp` → `~/.local/bin/pk-mcp`
- `which liter-llm` → `~/.local/bin/liter-llm`
- `which forge-mcp` → `~/.local/bin/forge-mcp`
- `which prometheus` → `~/.local/bin/prometheus`
- `which forge` → `~/.local/bin/forge`

---

### change-002 — launchd-plists-forge-and-pk
**Effort**: 20 min  
**Gaps closed**: G-SVC-1, G-SVC-2  
**Agent**: claude-code  
**Depends on**: change-001 (binaries must be in PATH first)

Create launchd plist files for `forge-mcp` and `pk-mcp`, modeled on the existing `dev.prometheusags.openai-proxy.plist` pattern:

| Service | Binary | Port | Label | Plist path |
|---|---|---|---|---|
| forge-mcp | `~/.local/bin/forge-mcp` | 8943 | `dev.prometheusags.forge-mcp` | `~/Library/LaunchAgents/dev.prometheusags.forge-mcp.plist` |
| pk-mcp | `~/.local/bin/pk-mcp` | 8942 | `dev.prometheusags.pk-mcp` | `~/Library/LaunchAgents/dev.prometheusags.pk-mcp.plist` |

Both plists must:
- Set `RunAtLoad: true`
- Set `KeepAlive: true`
- Set `ThrottleInterval: 10`
- Include `RUST_LOG=info` in `EnvironmentVariables`
- Run `launchctl load ~/Library/LaunchAgents/dev.prometheusags.<name>.plist`

**Acceptance**:
- `launchctl list | grep forge-mcp` → shows PID (non-zero)
- `launchctl list | grep pk-mcp` → shows PID (non-zero)
- `curl -s http://localhost:8943/health` → 200 (or equivalent health check)
- `curl -s http://localhost:8942/health` → 200 (or equivalent health check)

---

### change-003 — install-skills-all-platforms
**Effort**: 15 min  
**Gaps closed**: G-SKILL-1, G-SKILL-2, G-SKILL-3, G-SKILL-4, G-INST-4  
**Agent**: claude-code  

1. Add `zed` skill dir target to `scripts/install-skills-flat.sh`:
   ```bash
   install_to_dir "zed" "$HOME/.config/zed/skills"
   ```
2. Run `bash scripts/install-skills-flat.sh` to install symlinks to all supported platforms.

**Acceptance**:
- `ls ~/.config/zed/skills/ | wc -l` → > 0
- `ls ~/.opencode/skills/ | wc -l` → > 0
- `ls ~/.cursor/skills/ | wc -l` → > 0
- `ls ~/.codex/skills/ | wc -l` → > 0
- `ls ~/.claude/skills/ | wc -l` → >= 394 (unchanged or higher)

---

### change-004 — wire-mcp-all-tools
**Effort**: 30 min  
**Gaps closed**: G-MCP-1, G-MCP-2, G-MCP-3  
**Agent**: claude-code  
**Depends on**: change-002 (MCP services must be running before wiring)

Wire the `.mcp.json` server block from the repo root into each AI tool's native config file.

**Servers to wire** (from `.mcp.json`):
- `surreal-memory` — SSE `http://localhost:23001/sse`
- `sycophancy-correction` — command `sycophancy-correction`
- `forge-rs` — SSE `http://localhost:8943/sse`
- `prometheus-knowledge` — SSE `http://localhost:8942/sse`
- `liter-llm` — command `liter-llm`

**Targets**:

| Tool | Config file | Format |
|---|---|---|
| opencode | `~/.config/opencode/config.json` (or `opencode.json`) | JSON `mcpServers` block |
| codex | `~/.codex/config.yaml` | YAML `mcpServers` block |
| zed | `~/.config/zed/settings.json` | JSON `context_servers` block |

**Acceptance**:
- `cat ~/.config/opencode/config.json` → contains `surreal-memory` entry
- `cat ~/.codex/config.yaml` → contains `surreal-memory` entry
- `cat ~/.config/zed/settings.json` → contains `context_servers` with at least one MCP entry
- No existing config entries are removed or corrupted

---

### change-005 — prometheus-setup-command
**Effort**: 2–3 hours  
**Gaps closed**: G-INST-1, G-INST-2, G-INST-3  
**Agent**: claude-code  

Add `prometheus setup` subcommand to `tools/prometheus-cli/` with the following behaviour:

#### Behaviour
1. **Detect phase** — probe each component:
   - Docker containers: `docker ps --filter name=<name>`
   - launchd services: `launchctl list | grep <label>`
   - Binary in PATH: `which <binary>`
   - Port open: TCP connect to `localhost:<port>`
2. **Report status** — coloured table (like `prometheus doctor`) showing each component: ✅ installed / ⚠️ needs action / ❌ missing
3. **Interactive prompt** — for each gap: "Install X? [y/N/s(skip)]"
4. **Execute installs** — shell out to the appropriate install action per component type
5. **Persist state** — write `~/.prometheus/setup-state.json` for idempotent re-runs

#### CLI flags
- `--non-interactive` — assume yes to all prompts (CI/automation mode)
- `--dry-run` — show what would be done without executing
- `--check` — report status only, no prompts (same as `prometheus doctor` extended)

#### State file shape
```json
{
  "last_run": "2026-05-25T...",
  "components": {
    "surreal-memory-server": { "status": "skipped_docker", "last_checked": "..." },
    "openai-proxy": { "status": "skipped_launchd", "last_checked": "..." },
    "forge-mcp": { "status": "installed", "last_checked": "..." },
    "pk-mcp": { "status": "installed", "last_checked": "..." },
    "liter-llm": { "status": "installed", "last_checked": "..." }
  }
}
```

#### Files to modify/create
- `tools/prometheus-cli/crates/prometheus-cli/src/commands/setup.rs` — new subcommand
- `tools/prometheus-cli/crates/prometheus-cli/src/commands/mod.rs` — register `Setup`
- `tools/prometheus-cli/crates/prometheus-cli/src/main.rs` — add `Setup` variant to CLI enum

**Acceptance**:
- `prometheus setup --check` exits 0 and prints a status table
- `prometheus setup --dry-run` prints what would be installed without running anything
- `prometheus setup --non-interactive` runs all installs without prompting
- `~/.prometheus/setup-state.json` is created or updated after a run
- `cargo test -p prometheus-cli` passes

---

## Execution Order

```
change-001  →  change-002  →  change-003
                     ↓
               change-004
                     ↓
               change-005 (can overlap with 003/004)
```

Changes 001→002→004 are a hard dependency chain (binaries → launchd → MCP wiring).  
Change 003 (skills install) is independent and can run after change-001 is done.  
Change 005 (setup command) is independent once the shape of components is stable (after 004).

---

## Risk Register

| Risk | Mitigation |
|---|---|
| `pk-mcp` or `forge-mcp` crate names differ from workspace | Inspect Cargo.toml before build step |
| `liter-llm` may not compile on macOS (C FFI deps) | Check Cargo.toml for sys-crate deps; fallback: install pre-built release binary |
| `zed` skill dir path may differ from `~/.config/zed/skills` | Probe actual zed config dir before writing |
| opencode config file may be TOML or YAML, not JSON | Inspect current file format before patching |
| codex MCP config key may differ from `mcpServers` | Read current `~/.codex/config.yaml` before patching |
