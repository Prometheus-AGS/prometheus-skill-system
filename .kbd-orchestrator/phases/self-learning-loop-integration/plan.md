# Plan — Self-Learning Loop Integration

**Phase:** self-learning-loop-integration
**Date:** 2026-06-23
**Total changes:** 10
**Backend:** OpenSpec (openspec/changes/change-slli-*/proposal.md)

---

## Goal

Close the gap between the current skill-pack and a full, self-learning, cross-platform looping system that:

1. Runs `pk` and `forge` as always-on macOS launchd HTTP MCP services
2. Runs ALL MCP servers (surreal-memory, sycophancy-correction, liter-llm, sequential-thinking, tavily, pk, forge) as launch agents
3. Configures all MCP servers across ALL supported AI tools (Claude Code, OpenCode, Codex, Kimi, MiniMax, Cursor, Windsurf)
4. Provides L3 outer loop commands (/loop-define, /loop-tick, /loop-report)
5. Auto-wires continuous learning after every executor run
6. Fixes the per-turn progress signaling problem permanently

---

## Ordered Change List

| # | Change ID | Title | Agent | Depends on |
|---|-----------|-------|-------|------------|
| 1 | change-slli-008 | Standardize progress signaling across all kbd-* skills | executor | — |
| 2 | change-slli-002 | MCP launchd services installer (all 7 servers) | executor | — |
| 3 | change-slli-003 | Cross-tool MCP config (all 7 supported tools) | executor | slli-002 |
| 4 | change-slli-001 | L3 outer loop skill (/loop-define, /loop-tick, /loop-report) | executor | — |
| 5 | change-slli-004 | Wire continuous-learning-v2 into SubagentStop[executor] hook | executor | — |
| 6 | change-slli-005 | Upgrade pk-focus-on-prompt.sh with semantic hybrid_search_memories | executor | — |
| 7 | change-slli-006 | Forge-independent reflect path (direct pk ingest) | executor | — |
| 8 | change-slli-007 | Evolver-bridge.json integration in evolve-execute and kbd-reflect | executor | — |
| 9 | change-slli-009 | Periodic nudge script (scheduled cross-session enrichment) | executor | slli-003, slli-004 |
| 10 | change-slli-010 | pmpo-skill-creator --update mode for in-place skill improvement | executor | slli-004 |

---

## Change Details

### change-slli-008: Progress Signaling Fix (FIRST — unblocks all loop work)

**Root cause:** Six concurrent failure modes identified in analysis. Fix closes all six.

**Deliverables:**
- Add `## Progress Signals (MANDATORY)` section to ALL kbd-* skills with:
  - Exact format: `Starting <skill> — <phase-name> (step N of T)`
  - Instruction to read `.kbd-orchestrator/phases/<phase>/progress.json` for real N and T
  - Completion signal: `Completed <skill> — <phase-name> (step N of T)`
- Write `.kbd-orchestrator/position-reminder.txt` at every waypoint update with:
  - Current phase, step N of T, stage, exact next command
- Add `PreToolUse[Write|Edit]` hook variant that checks the current turn has emitted a Starting signal
- Skills that get the update: kbd-assess, kbd-analyze, kbd-plan, kbd-execute, kbd-reflect, kbd-evolve, iterative-evolver, pmpo-outer-loop (new)

**Acceptance:**
- Every kbd-* invocation emits a plain-text `Starting … (step N of T)` line before any tool call
- `.kbd-orchestrator/position-reminder.txt` always matches `current-waypoint.json`

---

### change-slli-002: MCP Launchd Services Installer

**Deliverables:**
- `scripts/install-mcp-services.sh` — idempotent installer for all 7 MCP servers as launchd launch agents
- Launchd plist files at `launchd/` in repo (installed to `~/Library/LaunchAgents/`):
  - `dev.prometheusags.surreal-memory.plist` (port 23001)
  - `dev.prometheusags.pk-mcp.plist` (port 8942, HTTP MCP mode)
  - `dev.prometheusags.forge-mcp.plist` (port 8943, HTTP MCP mode)
  - `dev.prometheusags.sycophancy-correction.plist` (stdio via launchd socket, port 8944)
  - `dev.prometheusags.liter-llm.plist` (port 8945, HTTP MCP mode)
  - `dev.prometheusags.sequential-thinking.plist` (port 8946)
  - `dev.prometheusags.tavily.plist` (port 8947)
- Each plist:
  - `KeepAlive: true`
  - `RunAtLoad: true`
  - `StandardOutPath` and `StandardErrorPath` → `~/.prometheus/logs/<service>.log`
  - `EnvironmentVariables` block for required API keys (read from `~/.prometheus/.env`)
- `scripts/prometheus-services.sh` updated to use launchctl (was previously a stub)
- Health-check command: `scripts/check-mcp-health.sh` — pings each port, reports status table

**Acceptance:**
- `launchctl list | grep prometheusags` shows all 7 services with non-zero PID
- Each port responds to TCP connect
- Log files exist at `~/.prometheus/logs/`

---

### change-slli-003: Cross-Tool MCP Config

**Deliverables:**
- `scripts/configure-mcp-all-tools.sh` — writes MCP server blocks into each tool's config:
  - Claude Code: `~/.claude/settings.json` (already has surreal-memory, pk, forge, liter-llm, tavily, sequential-thinking — verify/update URLs to launchd-hosted ports)
  - OpenCode: `~/.opencode/config.json` or `~/.config/opencode/config.json`
  - Codex: `~/.codex/config.yaml`
  - Kimi Code: `~/.kimi-code/config.toml`
  - MiniMax: `~/.minimax/mcp/mcp.json`
  - Cursor: `~/.cursor/mcp.json`
  - Windsurf: `~/.codeium/windsurf/mcp/mcp.json`
- All 7 MCP servers configured in each tool using launchd-hosted endpoints (SSE for HTTP servers, stdio for binary servers)
- `install-skills-flat.sh` updated to call `configure-mcp-all-tools.sh` at the end of every install run

**Acceptance:**
- Each tool config contains all 7 MCP server entries
- No existing tool config entries deleted
- Script is idempotent (run twice = same result)

---

### change-slli-001: L3 Outer Loop Skill

**Deliverables:**
- `skills/process/pmpo-outer-loop/SKILL.md` — commands: `/loop-define`, `/loop-tick`, `/loop-report`
- `skills/process/pmpo-outer-loop/references/loop-schema.md` — `loop.json` schema documentation
- `skills/process/pmpo-outer-loop/scripts/loop-tick.sh` — shell runner for background cadence
- State file: `.kbd-orchestrator/loops/<name>/loop.json` schema:
  ```json
  {
    "name": "string",
    "goal": "string (machine-checkable condition)",
    "feedback": [{"type": "command|file|url", "source": "string"}],
    "termination": {
      "max_ticks": "number",
      "max_no_progress_ticks": "number",
      "budget": "string (e.g. '2h', '5 USD')"
    },
    "escalation": "never|always|declared",
    "escalation_conditions": ["string"],
    "cadence": "manual|background|cron",
    "evolution_name": "string|null",
    "current_tick": "number",
    "no_progress_ticks": "number",
    "status": "active|paused|completed|escalated",
    "last_tick_at": "ISO8601",
    "created_at": "ISO8601"
  }
  ```
- `/loop-define <name>` — creates `loop.json`, validates all 6 parameters are present
- `/loop-tick [<name>]` — reads feedback sources, evaluates goal, runs one KBD cycle (L1), updates tick counters, checks termination
- `/loop-report [<name>]` — renders progress table: ticks used, no-progress ticks, goal eval result, last feedback snapshot

**Acceptance:**
- `/loop-define test-loop` creates a valid `loop.json` with all 6 required fields
- `/loop-tick` increments `current_tick` and writes feedback snapshot
- `/loop-report` renders a readable progress table

---

### change-slli-004: Wire Continuous Learning into SubagentStop[executor]

**Deliverables:**
- `shared/scripts/evaluate-session.sh` — runs after every executor SubagentStop:
  1. Reads last-completed change's scope from `current-waypoint.json`
  2. Calls `continuous-learning-v2` skill logic: extracts reusable patterns from the executor's output
  3. Calls `pk ingest` with extracted patterns → enriches the knowledge base
  4. Writes a `~/.prometheus/learning-log/<date>.jsonl` entry with what was learned
- `hooks/hooks.json` updated: add `evaluate-session.sh` to `SubagentStop[executor]` array (after existing `state-checkpoint.sh`)
- `skills/process/continuous-learning-v2/SKILL.md` verified/updated to support being invoked programmatically (not just interactively)

**Acceptance:**
- After any `/kbd-execute` run, `~/.prometheus/learning-log/` has a new entry
- `pk search <topic>` returns richer results after a related executor run

---

### change-slli-005: Semantic pk-focus Upgrade

**Deliverables:**
- `shared/scripts/pk-focus-on-prompt.sh` extended:
  1. Current behavior (top-5 longest words → `pk focus`) preserved as fast path
  2. NEW: if `surreal-memory` REST endpoint is reachable (`curl -sf http://localhost:23001/health`), call `hybrid_search_memories` with the raw prompt text (max 3s timeout)
  3. Merge semantic results with lexical results (deduplicate by topic key)
  4. Pass merged context list to `pk focus`
- Environment flag `PROMETHEUS_FOCUS_SEMANTIC=0` disables the semantic path (graceful opt-out)

**Acceptance:**
- `pk-focus-on-prompt.sh "how does the evolver bridge work"` → calls surreal-memory when available
- When surreal-memory is down, falls back to lexical-only without error
- Total script runtime stays under 3s regardless of path

---

### change-slli-006: Forge-Independent Reflect Path

**Deliverables:**
- `shared/scripts/forge-reflect-on-stop.sh` updated:
  1. `forge reflect` path is preserved when forge is available (no regression)
  2. When forge is absent/unreachable: directly call `pk ingest --session-summary "$(cat ~/.prometheus/last-session-summary.txt 2>/dev/null)"` as the fallback
  3. `shared/scripts/write-session-summary.sh` — new helper, called by Stop hook, writes a session summary to `~/.prometheus/last-session-summary.txt`
- `hooks/hooks.json` Stop array: add `write-session-summary.sh` BEFORE `forge-reflect-on-stop.sh`

**Acceptance:**
- On a machine without forge: Stop hook still calls `pk ingest` with session summary
- On a machine with forge: behavior unchanged
- `~/.prometheus/last-session-summary.txt` exists after every session Stop

---

### change-slli-007: Evolver-Bridge Integration

**Deliverables:**
- `skills/process/iterative-evolver/SKILL.md` updated: when `evolver-bridge.json` exists at `phases/<current>/evolver-bridge.json`, write `execution_results` back to bridge after each change completes
- `skills/process/kbd-process-orchestrator/SKILL.md` updated: `/kbd-reflect` reads `evolver-bridge.json` if present, reports per-evolver-item status (completed/skipped/failed) to the evolver's `state.json`
- `openspec/changes/change-slli-007/bridge-schema.md` — canonical schema reference (creates the missing schema documentation identified in analysis)

**Acceptance:**
- After `/kbd-reflect`, if `evolver-bridge.json` exists, the evolver's `state.json` shows updated `execution_results`
- Running `/evolve status <name>` shows accurate per-item completion from the KBD phase

---

### change-slli-009: Periodic Nudge Script

**Deliverables:**
- `scripts/scheduled/periodic-nudge.sh` — cross-session enrichment trigger:
  1. Calls `pk ingest --scan-recent-changes` (last 24h git log → KB entries)
  2. Calls `hybrid_search_memories` for the active evolution/loop name
  3. Writes a nudge summary to `~/.prometheus/nudge-log/<date>.txt`
  4. If stall detected (no commits in `max_no_progress_ticks` ticks): emits a `~/Library/Messages/` notification (macOS) or writes to a watched file
- Launchd plist: `dev.prometheusags.prometheus-nudge.plist` — runs every 4 hours
- Installed by `install-mcp-services.sh` alongside the MCP server plists

**Acceptance:**
- `launchctl list | grep prometheus-nudge` shows the agent
- `~/.prometheus/nudge-log/` gains an entry after first trigger
- `pk search` returns more results 4h after a coding session than immediately after

---

### change-slli-010: pmpo-skill-creator --update Mode

**Deliverables:**
- `skills/process/pmpo-skill-creator/SKILL.md` updated: add `/pmpo-skill-creator --update <skill-name>` command
  - Reads existing `~/.claude/skills/<skill-name>/SKILL.md`
  - Diffs against recent usage patterns extracted from `~/.prometheus/learning-log/`
  - Proposes targeted additions (new examples, updated references, corrected instructions)
  - Writes proposed diff to `~/.prometheus/skill-updates/<skill-name>-<date>.diff`
  - Requires user approval before applying (calls `/pmpo-elicit` if available, else prompts in-turn)
- `shared/scripts/propose-skill-update.sh` — called by `evaluate-session.sh` (change-slli-004) when learning patterns match an existing skill

**Acceptance:**
- `/pmpo-skill-creator --update kbd-plan` produces a diff file with additions from recent sessions
- The diff is NOT auto-applied without explicit user confirmation
- Approval gate is clearly presented to the user

---

## Execution Order Rationale

1. **Progress signaling (slli-008) first** — the user has asked for this repeatedly; it must be fixed before any other loop work so the execution of this very plan is visible at every step.
2. **MCP launchd services (slli-002) second** — everything else depends on the services being up. Run standalone.
3. **Cross-tool config (slli-003) third** — depends on slli-002 (services must exist before configuring tools to point at them).
4. **L3 outer loop skill (slli-001) fourth** — pure skill file creation, no runtime dependency.
5. **Continuous learning wiring (slli-004) fifth** — depends only on hook system.
6. **pk semantic focus (slli-005) sixth** — enhancement to existing script, no hard dependencies.
7. **Forge-independent reflect (slli-006) seventh** — enhancement to existing hook script.
8. **Evolver bridge (slli-007) eighth** — skill content update, no runtime dependency.
9. **Periodic nudge (slli-009) ninth** — depends on slli-003 (all tools configured) and slli-004 (learning log exists).
10. **pmpo-skill-creator update mode (slli-010) last** — depends on slli-004 (learning log as input).

---

## Cross-Harness MCP Port Table

| Service | Port | Protocol | Plist label |
|---------|------|----------|-------------|
| surreal-memory | 23001 | SSE | dev.prometheusags.surreal-memory |
| pk-mcp (prometheus-knowledge) | 8942 | HTTP MCP | dev.prometheusags.pk-mcp |
| forge-mcp | 8943 | HTTP MCP | dev.prometheusags.forge-mcp |
| sycophancy-correction | 8944 | stdio via socket | dev.prometheusags.sycophancy-correction |
| liter-llm | 8945 | HTTP MCP | dev.prometheusags.liter-llm |
| sequential-thinking | 8946 | HTTP MCP | dev.prometheusags.sequential-thinking |
| tavily | 8947 | HTTP MCP | dev.prometheusags.tavily |
| periodic-nudge | N/A | cron | dev.prometheusags.prometheus-nudge |

---

## OpenSpec Change References

All changes tracked as OpenSpec proposals under `openspec/changes/change-slli-*/proposal.md`.

---

## Success Criteria for Phase

- [ ] All 7 MCP servers run as launchd launch agents and survive reboot
- [ ] `scripts/check-mcp-health.sh` reports GREEN for all services
- [ ] All 7 supported tools (Claude Code, OpenCode, Codex, Kimi, MiniMax, Cursor, Windsurf) have all 7 MCP servers configured
- [ ] Every kbd-* skill emits `Starting … (step N of T)` before first tool call
- [ ] `/loop-define test-loop` creates a valid `loop.json`
- [ ] After `/kbd-execute`, learning log has a new entry
- [ ] `pk-focus-on-prompt.sh` calls surreal-memory when available
- [ ] Stop hook calls `pk ingest` even when forge is absent
- [ ] `/kbd-reflect` updates evolver-bridge when present
- [ ] `/pmpo-skill-creator --update <skill>` produces a review-gated diff
