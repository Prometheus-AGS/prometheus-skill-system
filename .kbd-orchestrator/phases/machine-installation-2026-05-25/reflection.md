# Reflection: machine-installation-2026-05-25
**Date**: 2026-05-25  
**Executor**: claude-code (claude-sonnet-4-6)  
**Backend**: openspec  

---

## Goal Achievement

| Goal | Status | Evidence |
|------|--------|----------|
| All project binaries built and in PATH | ✅ MET | `forge`, `pk-cherry`, `liter-llm`, `prometheus` in `~/.local/bin/`; `which` confirms each |
| forge-mcp and pk-mcp running as launchd agents on startup | ✅ MET | Both plists loaded; health checks return `{status:ok}` on ports 8943 and 8942 |
| Skills installed to all supported AI tool platforms | ✅ MET | 81 skills × 9 platforms: claude-code, opencode, cursor, codex, gemini, roo, windsurf, cline, zed |
| MCP servers wired into opencode, codex, and zed | ✅ MET | 5 servers (forge-rs, prometheus-knowledge, sycophancy-correction, liter-llm, surreal-memory) wired in all 3 tools |
| `prometheus setup` subcommand | ✅ MET | `--check`, `--dry-run`, `--non-interactive` implemented; all 9 components detected healthy |

**Overall: 5/5 goals MET (100%)**

---

## Delivered Changes

| # | Change ID | Status | Gaps Closed | Notes |
|---|-----------|--------|-------------|-------|
| 1 | change-install-001-build-and-install-binaries | ✅ DONE | G-BIN-1, G-BIN-2, G-SVC-3 | Built pk-cherry (pk-mcp MCP server), liter-llm, forge; installed all to `~/.local/bin/` |
| 2 | change-install-002-launchd-plists-forge-and-pk | ✅ DONE | G-SVC-1, G-SVC-2 | forge-mcp (8943) + pk-mcp (8942) plist created, loaded, both healthy |
| 3 | change-install-003-install-skills-all-platforms | ✅ DONE | G-SKILL-1–4, G-INST-4 | Added zed target to install-skills-flat.sh; 81 skills × 9 platforms installed |
| 4 | change-install-004-wire-mcp-all-tools | ✅ DONE | G-MCP-1, G-MCP-2, G-MCP-3 | opencode.json, ~/.codex/config.toml, ~/.config/zed/settings.json all updated |
| 5 | change-install-005-prometheus-setup-command | ✅ DONE | G-INST-1, G-INST-2, G-INST-3 | commands/setup.rs created; 3 unit tests pass; state written to `~/.prometheus/setup-state.json` |

---

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with formal QA | 1/5 |
| First-pass pass rate | 1/1 (100%) |
| Changes skipped (< 3 files / config-only) | 4/5 |
| Unit tests (change-005) | 3 passed |

### QA Disposition per Change

- **change-001**: QA skipped — binary install, no source files modified
- **change-002**: QA skipped — 2 plist config files only
- **change-003**: QA skipped — 1 script file changed
- **change-004**: QA skipped — 3 config files only (no source changes)
- **change-005**: QA passed — 3 tests: `component_status_needs_action_only_for_gap_states`, `component_status_labels_are_non_empty`, `setup_state_path_ends_with_expected_filename`

---

## Key Discovery

**`pk-mcp` is a library crate, not a binary crate.** The plan incorrectly named `pk-mcp` as the build target and install binary. The actual MCP server binary is `pk-cherry` (`tools/prometheus-knowledge/pk-cherry/src/main.rs`), which wraps `pk-mcp::McpServer`. This was discovered inline during change-001 when `cargo build -p pk-mcp` produced no binary. The plan acceptance criteria referenced `~/.local/bin/pk-mcp` but the correct install is `~/.local/bin/pk-cherry`. All downstream changes (plist, setup.rs component detection) correctly use `pk-cherry`.

**Impact on assessment**: The assessment gap `G-BIN-2` ("pk-mcp binary not installed") was silently based on an incorrect assumption about the crate structure. Future assessors should check `src/main.rs` existence before naming a crate as a "binary target."

---

## Final Verification

```
$ prometheus setup --check
🚀 Prometheus Setup

Component Status
  ✅ surreal-memory-server (Docker, port 23001) — running (Docker)
  ✅ openai-proxy (launchd, port 8181) — running (launchd)
  ✅ forge-mcp SSE server (launchd, port 8943) — ok
  ✅ prometheus-knowledge MCP server (launchd, port 8942) — ok
  ✅ liter-llm stdio MCP proxy (~/.local/bin/liter-llm) — installed
  ✅ prometheus CLI (~/.local/bin/prometheus) — installed
  ✅ forge code enrichment CLI (~/.local/bin/forge) — installed
  ✅ pk-cherry knowledge MCP binary (~/.local/bin/pk-cherry) — installed
  ✅ sycophancy-correction binary (/usr/local/bin/) — installed

✨ All components healthy — nothing to do.
```

---

## Technical Debt Introduced

None introduced by this phase. Pre-existing debt noted:
- `~/.opencode/package-lock.json` — stale npm artifact in the opencode config directory (pre-existing, not caused by this phase)

---

## Lessons Captured

1. **Verify crate type before naming binary targets in the plan.** A library crate (`lib.rs` only, no `main.rs`) will not produce a binary. Check `Cargo.toml [[bin]]` sections or the presence of `src/main.rs`.

2. **launchctl output pollutes stdout.** `launchctl list <label>` prints the plist dict to stdout when the service is found. Always redirect both stdout and stderr to `/dev/null` in detection code.

3. **`which` crate is not in the prometheus-cli workspace.** Use `std::process::Command::new("which")` instead — no dependency needed.

4. **MCP config format varies per tool.** opencode uses `"mcp"` (not `"mcpServers"`); codex uses TOML `[mcp_servers.*]`; zed uses `"context_servers"` and requires an `mcp-remote@latest` npx wrapper for SSE servers.

5. **Config-only and binary-install changes should remain QA-exempt.** The 4-skipped-QA pattern was correct and efficient; only source changes warrant artifact-refiner review.

---

## Recommended Next Phase

1. **Commit this phase** — stage all changes from this session: `scripts/install-skills-flat.sh`, `tools/prometheus-cli/crates/prometheus-cli/src/commands/setup.rs`, `mod.rs`, `main.rs`, openspec change files, and kbd-orchestrator phase files.

2. **Resume `skill-pack-all-phases-2026-05-09`** — the machine is now fully set up. The next phase in the master plan (`skill-pack-all-phases`) should pick up from where Phase 1 (pglite-certification) left off.

3. **Update the assessment for `pk-cherry`** — if a future phase re-assesses binary gaps, the assessment template should check for `pk-cherry` explicitly, not `pk-mcp`.
