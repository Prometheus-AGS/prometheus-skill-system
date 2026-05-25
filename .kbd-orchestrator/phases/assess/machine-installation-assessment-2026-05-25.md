# Assessment: Machine-Wide Skill Installation
**Date**: 2026-05-25  
**Assessor**: kbd-assess  
**Goal**: Full machine setup — global skill installation, all project binaries, launchd services, per-tool MCP wiring

---

## 1. Current Machine State (observed)

### Services already running
| Service | Status | How running | Endpoint |
|---------|--------|-------------|----------|
| `surreal-memory-server` | ✅ UP (healthy, 23h) | Docker Desktop | `http://localhost:23001` |
| `surrealdb` | ✅ UP (healthy, 23h) | Docker Desktop | `http://localhost:28000` |
| `openai-proxy` | ✅ UP (listening) | launchd (`dev.prometheusags.openai-proxy.plist`) | `http://localhost:8181` |

**Action for these**: Do NOT reinstall. Use Docker Desktop for surreal services. Keep existing launchd plist for openai-proxy.

### Installed binaries (in PATH / cargo)
| Binary | Location | Status |
|--------|----------|--------|
| `sycophancy-correction` | `/usr/local/bin/` | ✅ Installed |
| `prometheus-rust-auditor` | `~/.cargo/bin/` | ✅ Installed |
| `openai-proxy` | `~/.local/bin/openai-proxy` | ✅ Installed |
| `prometheus` (CLI) | Built but **not installed** to PATH | ⚠️ Built only |
| `forge` | Built but **not installed** to PATH | ⚠️ Built only |

### Binaries NOT yet built
| Tool | Path | Gap |
|------|------|-----|
| `pk-mcp` (prometheus-knowledge MCP) | `tools/prometheus-knowledge/` | ❌ Not built |
| `liter-llm` | `tools/liter-llm/` | ❌ Not built |
| `forge-mcp` (SSE server binary) | `tools/forge-rs/` | ❌ `forge` built but MCP server binary name unclear |

### MCP Services NOT running
| Service | Expected port | Status |
|---------|--------------|--------|
| `forge-rs` MCP server | 8943 | ❌ Not running |
| `prometheus-knowledge` MCP | 8942 | ❌ Not running |
| `liter-llm` stdio MCP | n/a | ❌ Not installed to PATH |

### AI Tools Installed
| Tool | Location | Skills installed? | MCP wired? |
|------|----------|-------------------|------------|
| `claude` (Claude Code) | `~/.local/state/fnm_multishells/.../bin/claude` | ✅ 394 skills | Partial |
| `codex` | `~/.local/state/fnm_multishells/.../bin/codex` | Unknown | Unknown |
| `opencode` | `~/.local/bin/opencode` | Via `~/.config/opencode/skills/` | Partial |
| `zed` | `/usr/local/bin/zed` | Via `~/.config/zed/skills/` | Unknown |
| `cursor` | `/usr/local/bin/cursor` | Via `~/.cursor/skills/` | Unknown |

### Skill Installation Coverage
`install-skills-flat.sh` covers: claude-code, opencode, cursor, codex, gemini, roo, windsurf, cline  
Status: **Not yet run globally** — skills currently installed to `~/.claude/skills/` (394) but other platforms unknown.

---

## 2. Gap Register

### G-SVC: MCP Services Not Running as Launchd

| ID | Service | Gap | Action needed |
|----|---------|-----|---------------|
| G-SVC-1 | `forge-rs` MCP SSE server | Not running on port 8943 | Build `forge-mcp` binary → launchd plist |
| G-SVC-2 | `prometheus-knowledge` MCP SSE server | Not running on port 8942 | Build `pk-mcp` binary → launchd plist |
| G-SVC-3 | `liter-llm` | Not installed to PATH | Build from submodule → `~/.local/bin/liter-llm` |

### G-BIN: Binaries Built But Not Installed to PATH

| ID | Binary | Built at | Action |
|----|--------|----------|--------|
| G-BIN-1 | `prometheus` (CLI) | `tools/prometheus-cli/target/release/prometheus` | `cp` to `~/.local/bin/prometheus` |
| G-BIN-2 | `forge` | `tools/forge-rs/target/release/forge` | `cp` to `~/.local/bin/forge` |

### G-SKILL: Skills Not Installed to All AI Tool Platforms

| ID | Platform | Gap | Action |
|----|----------|-----|--------|
| G-SKILL-1 | opencode | May not have all skills | Run `install-skills-flat.sh` |
| G-SKILL-2 | cursor | Skills unknown | Run `install-skills-flat.sh` |
| G-SKILL-3 | codex | Skills unknown | Run `install-skills-flat.sh` |
| G-SKILL-4 | zed | Not in install script | Add zed target to install script |

### G-MCP: MCP Config Not Wired to All Tools

| ID | Tool | Gap | Action |
|----|------|-----|--------|
| G-MCP-1 | opencode | `~/.config/opencode/opencode.json` MCP servers not set | Wire `.mcp.json` servers |
| G-MCP-2 | codex | `~/.codex/config.yaml` unknown MCP state | Inspect + wire |
| G-MCP-3 | zed | No MCP wiring found | Zed MCP via `settings.json` |

### G-INSTALL: No Unified Install Orchestrator

| ID | Gap | Action |
|----|-----|--------|
| G-INST-1 | No single `prometheus setup` command that detects what's needed and installs/skips | Create `prometheus setup` subcommand with service detection |
| G-INST-2 | No launchd plist templates for forge-mcp and pk-mcp | Create 2 plists |
| G-INST-3 | `prometheus doctor` doesn't check forge/pk-mcp ports | Extend doctor to check all MCP endpoints |
| G-INST-4 | `install-skills-flat.sh` doesn't cover zed | Add zed skill dir to script |

---

## 3. Services: What to Install as Launchd (not Docker)

### Services to keep as Docker (already healthy)
- `surreal-memory-server` → Docker, port 23001 ✅
- `surrealdb` → Docker, port 28000 ✅

### Services to install as launchd (user identity: gqadonis)
| Service | Binary | Port | Plist label |
|---------|--------|------|-------------|
| `forge-rs` MCP | `~/.local/bin/forge-mcp` | 8943 | `dev.prometheusags.forge-mcp` |
| `prometheus-knowledge` MCP | `~/.local/bin/pk-mcp` | 8942 | `dev.prometheusags.pk-mcp` |
| `liter-llm` | `~/.local/bin/liter-llm` (stdio, no port) | n/a | Not a daemon — installed to PATH only |

### Already running as launchd
- `openai-proxy` → `dev.prometheusags.openai-proxy.plist` ✅

---

## 4. Installer Design

The ideal UX is a single command that:
1. Detects what's running (Docker, launchd, PATH) — skip what's healthy
2. Builds binaries that aren't built
3. Installs binaries to `~/.local/bin/`
4. Creates and loads launchd plists for SSE services
5. Runs `install-skills-flat.sh` for all platforms
6. Wires MCP config per AI tool
7. Asks the user when ambiguous (e.g., "forge-mcp not found — build from source?")

This is a `prometheus setup` subcommand (extending the existing `prometheus` CLI).

---

## 5. Recommended Phase Plan

### change-001 — build-and-install-binaries
**Effort**: 30 min  
Build missing binaries and install all binaries to `~/.local/bin/`:
- `cargo build --release` in `tools/liter-llm/`
- `cargo build --release -p forge-mcp` in `tools/forge-rs/`  
- `cargo build --release -p pk-mcp` in `tools/prometheus-knowledge/`
- Copy `prometheus`, `forge`, `forge-mcp`, `pk-mcp`, `liter-llm` to `~/.local/bin/`

### change-002 — launchd-plists-forge-and-pk
**Effort**: 20 min  
Create and load launchd plist files for `forge-mcp` (port 8943) and `pk-mcp` (port 8942), modeled on existing `dev.prometheusags.openai-proxy.plist` pattern.

### change-003 — install-skills-all-platforms
**Effort**: 15 min  
- Add `zed` skill dir target to `install-skills-flat.sh`
- Run `install-skills-flat.sh` to install symlinks to all supported platforms

### change-004 — wire-mcp-all-tools
**Effort**: 30 min  
Wire `.mcp.json` MCP server configuration to:
- opencode (`~/.config/opencode/opencode.json` — MCP section)
- codex (`~/.codex/config.yaml` — MCP section)
- zed (`~/.config/zed/settings.json` — MCP section)
(Claude Code is already wired via `.mcp.json` at project + user scope)

### change-005 — prometheus-setup-command
**Effort**: 2–3 hours  
Add `prometheus setup` subcommand to `tools/prometheus-cli/` that:
- Detects running services (Docker, launchd, port probes)
- Detects installed binaries
- Reports status with colored output (like `prometheus doctor`)
- Interactively offers to build/install/skip each component
- Runs `install-skills-flat.sh` as a step
- Uses `--non-interactive` flag for CI/automation
- Writes a `~/.prometheus/setup-state.json` for idempotent re-runs

---

## 6. Certification: What's Already Done

| Component | Status |
|-----------|--------|
| surreal-memory-server (Docker) | ✅ Running — no action needed |
| surrealdb (Docker) | ✅ Running — no action needed |
| openai-proxy (launchd) | ✅ Running — no action needed |
| sycophancy-correction binary | ✅ Installed at `/usr/local/bin/` |
| prometheus-rust-auditor binary | ✅ Installed at `~/.cargo/bin/` |
| Claude Code skills (394) | ✅ Installed |

---

*Assessment written to: `.kbd-orchestrator/phases/assess/machine-installation-assessment-2026-05-25.md`*
