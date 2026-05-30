# Plan: machine-refresh-2026-05-30

**Date**: 2026-05-30
**Backend**: OpenSpec (`openspec/` present, `change_backend: openspec`)
**Assessment**: `.kbd-orchestrator/phases/assess/machine-refresh-assessment-2026-05-30.md`
**Goal**: Pull latest repo + submodules, rebuild all binaries against the drift, fix skill-install
drift across platforms, wire the prometheus MCP stack into Claude Desktop, and certify the end state.

## Locked decisions (from user, 2026-05-30)

1. **prometheus-knowledge submodule** → **keep pinned** at `ee611fc`. Do NOT `update --remote` it.
2. **Install driver** → use **discrete `scripts/install-binaries.sh`** for the actual rebuild, and use
   **`prometheus setup --check`** as the *verification gate only*.
   > **Why the split**: `setup.rs` detects components by **presence only** (`detect_binary`,
   > `detect_port`, `detect_launchd`) — it has **no staleness detection**. Since all 4 binaries already
   > exist and all 3 services are healthy, `prometheus setup --non-interactive` would report green and
   > **skip the rebuild**, defeating the purpose of this refresh. So: explicit build first, `setup --check` verifies.
3. **Claude Desktop MCP** → wire **all 3**: `forge-rs`, `prometheus-knowledge`, `liter-llm`.

## Key technical constraint (Claude Desktop)

Claude Desktop natively supports **stdio** MCP servers only. The 3 target servers split:
- `liter-llm` → **stdio** (`command: liter-llm, args: ["mcp","--transport","stdio"]`) — direct wire ✅
- `forge-rs` → **SSE** on `http://localhost:8943/mcp` — needs an `npx mcp-remote <url>` stdio bridge
- `prometheus-knowledge` → **SSE** on `http://localhost:8942/mcp` — needs an `npx mcp-remote <url>` stdio bridge

The wiring step will use `mcp-remote` for the two SSE servers and a direct stdio entry for liter-llm.

---

## Ordered change list

| # | Change ID | Closes | Agent | Effort | Depends on |
|---|-----------|--------|-------|--------|------------|
| 1 | `change-refresh-001-pull-repo-and-submodules` | G-PULL-1,2,3,4 | (direct / general-purpose) | 15 min | — |
| 2 | `change-refresh-002-rebuild-and-reinstall-binaries` | G-BUILD-1..5 | rust-build-resolver (on failure) | 30–45 min | 001 |
| 3 | `change-refresh-003-reinstall-skills-all-platforms` | G-SKILL-1,2 | (direct) | 15 min | 001 |
| 4 | `change-refresh-004-wire-claude-desktop-mcp` | G-DESKTOP-1..4 | (direct) | 20 min | 002 |
| 5 | `change-refresh-005-verify-refresh` | G-VERIFY-1 | (direct) | 15 min | 002,003,004 |

**Total**: ~1.5–2 hours. Order is strict for 1→2→{3,4}→5; 003 and 004 can run after their deps in either order.

---

### change-refresh-001 — pull-repo-and-submodules
**Closes**: G-PULL-1, G-PULL-2, G-PULL-3, G-PULL-4
**Effort**: 15 min

Tasks:
- [ ] `git pull` on `origin/main` (fast-forward; working tree clean, 2 ahead on remote)
- [ ] `git submodule update --remote tools/liter-llm` (149 commits behind → tip)
- [ ] `git submodule update --remote tools/surreal-memory-server` (3 commits behind → tip)
- [ ] **Do NOT** touch `tools/prometheus-knowledge` — keep pinned at `ee611fc` (decision #1)
- [ ] `git submodule update --init --recursive` to ensure nested submodules of updated modules are present
- [ ] Verify: `git submodule status` shows liter-llm + surreal-memory at new tips, pk unchanged

Acceptance:
- Repo at `origin/main` HEAD (`eb3134b` or newer)
- `liter-llm` and `surreal-memory-server` advanced; `prometheus-knowledge` still `ee611fc`

---

### change-refresh-002 — rebuild-and-reinstall-binaries
**Closes**: G-BUILD-1, G-BUILD-2, G-BUILD-3, G-BUILD-4, G-BUILD-5
**Effort**: 30–45 min
**Depends on**: 001

Tasks:
- [ ] Run `bash scripts/install-binaries.sh` — rebuilds + reinstalls `prometheus`, `forge`, `pk-cherry`, `liter-llm` to `~/.local/bin/`
- [ ] If any `cargo build` fails → invoke **rust-build-resolver** agent for minimal fixes (note: BDD immutable-tests rule — do not edit tests to pass)
- [ ] Restart launchd MCP services to load new binaries:
  - [ ] `launchctl kickstart -k gui/$(id -u)/dev.prometheusags.forge-mcp`
  - [ ] `launchctl kickstart -k gui/$(id -u)/dev.prometheusags.pk-mcp`
- [ ] Re-probe health: `curl -s http://localhost:8943/health` and `:8942/health` return `status:ok`

Acceptance:
- All 4 binaries rebuilt (newer mtime) and in PATH
- forge-mcp (8943) + pk-mcp (8942) healthy after kickstart
- `prometheus --version` / `forge --version` / `liter-llm --version` run clean

> **Note**: pk-cherry rebuilds from the *pinned* pk submodule — unchanged source, so it may be a no-op
> rebuild. That's fine; the kickstart still refreshes the running process.

---

### change-refresh-003 — reinstall-skills-all-platforms
**Closes**: G-SKILL-1, G-SKILL-2
**Effort**: 15 min
**Depends on**: 001

Tasks:
- [ ] Run `bash scripts/install-skills-flat.sh` — installs the repo's ~99 skills to all 9 platforms (claude-code, opencode, cursor, codex, gemini, roo, windsurf, cline, zed)
- [ ] Verify convergence: opencode / cursor / codex / zed skill counts align with repo skill count (~99), no longer 94/105/128/328
- [ ] Confirm claude-code aggregate (`~/.claude/skills` → `~/.TOOLS/skills/claude`) includes this repo's skills; reconcile if the symlinked source doesn't pull from the repo

Acceptance:
- Non-claude platform skill dirs converge to the repo skill set
- No stale/orphaned skills causing the 328 over-count on codex

---

### change-refresh-004 — wire-claude-desktop-mcp
**Closes**: G-DESKTOP-1, G-DESKTOP-2, G-DESKTOP-3, G-DESKTOP-4
**Effort**: 20 min
**Depends on**: 002 (services must be healthy to be reachable)

Tasks:
- [ ] **Back up** `~/Library/Application Support/Claude/claude_desktop_config.json` first
- [ ] **MERGE** (never clobber) into existing `mcpServers` (13 servers — preserve all):
  - [ ] `liter-llm`: `{ "command": "liter-llm", "args": ["mcp","--transport","stdio"] }`
  - [ ] `forge-rs`: `{ "command": "npx", "args": ["-y","mcp-remote","http://localhost:8943/mcp"] }`
  - [ ] `prometheus-knowledge`: `{ "command": "npx", "args": ["-y","mcp-remote","http://localhost:8942/mcp"] }`
- [ ] Preserve `coworkUserFilesPath` and `preferences` top-level keys
- [ ] Validate the merged JSON parses; confirm server count is now 16
- [ ] Restart Claude Desktop (or note to user that an app restart is required to load new servers)

Acceptance:
- `claude_desktop_config.json` has 16 mcpServers including the 3 prometheus entries
- All 13 pre-existing servers intact
- JSON valid

> **CRITICAL**: hand-curated config — use a JSON merge (python/jq), not a template overwrite.

---

### change-refresh-005 — verify-refresh
**Closes**: G-VERIFY-1
**Effort**: 15 min
**Depends on**: 002, 003, 004

Tasks:
- [ ] `prometheus setup --check` → all components report healthy (the verification gate)
- [ ] `prometheus doctor` → directories/platforms/connectivity green
- [ ] Probe all 3 MCP ports: 8942, 8943, 8181 return `status:ok`
- [ ] Confirm 4 binaries in PATH with fresh build times
- [ ] Confirm skill counts converged across platforms
- [ ] Confirm Claude Desktop config = 16 servers
- [ ] Write a one-paragraph verification summary into the change for the reflect phase

Acceptance:
- Single green end-state: pull done, binaries fresh, services healthy, skills converged, 5 tools wired (Claude Code, Claude Desktop, OpenCode, Codex, Zed)

---

## OpenSpec emission

Emit each change as an OpenSpec change. Recommended commands:

```
/opsx:new change-refresh-001-pull-repo-and-submodules
/opsx:new change-refresh-002-rebuild-and-reinstall-binaries
/opsx:new change-refresh-003-reinstall-skills-all-platforms
/opsx:new change-refresh-004-wire-claude-desktop-mcp
/opsx:new change-refresh-005-verify-refresh
```

Most changes here are operational (install/config) rather than source-spec changes, so the OpenSpec
proposals will be lightweight (proposal + tasks), and several will be QA-skipped per the prior phase
convention (config/script-only changes < 3 source files).

---

## Next step

Run `/kbd-execute machine-refresh-2026-05-30` to select the backend and dispatch change-refresh-001.
Execute changes strictly in order 001 → 002 → {003, 004} → 005.
