# Reflection: machine-refresh-2026-05-30

**Date**: 2026-05-30
**Executor**: claude-code (claude-opus-4-8)
**Backend**: openspec
**Previous phase**: machine-installation-2026-05-25

---

## Goal Achievement

| Goal | Status | Evidence |
|------|--------|----------|
| Repo + submodules pulled to current upstream | ✅ MET | `eb3134b` HEAD; liter-llm `+149`, surreal-memory `+3`; pk pinned per decision |
| All binaries rebuilt against pulled sources | ✅ MET | 4 binaries with 06:27–06:38 mtimes; liter-llm now `v1.4.0-rc.41` |
| MCP services running fresh binaries | ✅ MET | both kickstarted; 8943 + 8942 return `status:ok` |
| Skills installed uniformly to all platforms | ✅ MET | 88 skills × 13 targets (claude-code, opencode, cursor, codex, gemini, roo, windsurf, windsurf-legacy, amp, zed, antigravity, cline) |
| Claude Desktop wired into the prometheus stack | ⚠️ PARTIAL | config merged (13 → 16 servers, originals intact, absolute paths); **not yet verified to start** because verification requires user app restart |
| End-state certification gate | ✅ MET | `prometheus setup --check`: 11/11 healthy; `prometheus doctor`: all checks passed |

**Overall: 5/6 fully met (83%); 1 partial pending external action (app restart).**

---

## Delivered Changes

| # | Change ID | Status | Gaps Closed | Notes |
|---|-----------|--------|-------------|-------|
| 1 | change-refresh-001-pull-repo-and-submodules | ✅ DONE | G-PULL-1..4 | Required workaround: `--init --recursive` without `--remote` reverted the targeted advances; had to re-apply `--remote --init --recursive` per submodule |
| 2 | change-refresh-002-rebuild-and-reinstall-binaries | ✅ DONE | G-BUILD-1..5 | Caught & fixed 2 upstream package renames in install script; total build ~12 min |
| 3 | change-refresh-003-reinstall-skills-all-platforms | ✅ DONE | G-SKILL-1,2 | Installer reports 88 skills (not 99/106 from `find`) due to deliberate scope filter — orchestrator child-skills + imported submodules are excluded by design |
| 4 | change-refresh-004-wire-claude-desktop-mcp | ✅ DONE | G-DESKTOP-1..4 | Hardened with absolute paths after recognizing fnm multishell path was ephemeral; SSE servers wired via `npx mcp-remote` bridges |
| 5 | change-refresh-005-verify-refresh | ✅ DONE | G-VERIFY-1 | Single-pass verification (no remediation needed) |

---

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with formal QA (artifact-refiner) | 0/5 |
| First-pass pass rate | n/a |
| Changes skipped (<3 source files / config-only) | 5/5 |
| Source files modified | 1 (`scripts/install-binaries.sh`) |

### QA Disposition per Change

- **change-001**: skipped — git state only, no source files
- **change-002**: skipped — only `scripts/install-binaries.sh` was edited (1 file < 3 threshold); binary builds are submodule sources not subject to local QA
- **change-003**: skipped — script-driven symlink install, no source
- **change-004**: skipped — 1 config file (`claude_desktop_config.json`), not a repo source
- **change-005**: skipped — verification report only

No artifact-refiner invocation was required by the existing QA-skip rule. **This is a gap in the QA rule** — see "Lessons" below.

---

## Delta → Root Cause → Corrective Actions

This reflection must name where reality diverged from the plan, not paper over the rough edges.

### Δ 1. The first build silently failed; only `prometheus` got rebuilt

- **What we expected**: `scripts/install-binaries.sh` would rebuild all 4 binaries.
- **What happened**: It aborted at `cargo build -p forge` (package renamed upstream), but the background harness reported exit 0 because of how the script's `( … | tail -3 )` pipe and `set -euo pipefail` interacted with the subshell.
- **Root cause**: Upstream `forge-rs` was renamed `forge` → `forge-cli` and upstream `liter-llm` was renamed `liter-llm` → `liter-llm-cli` during the 149 + N commits of drift. The install script encoded the old package names. The pipe-to-`tail` pattern masked the cargo failure long enough that I only caught it on log inspection.
- **Corrective actions taken**:
  - Updated `scripts/install-binaries.sh` to use `-p forge-cli` and `-p liter-llm-cli`.
  - Verified the build aborted *cleanly* (set -e did stop the script; the misleading "exit 0" was from a separate echo wrapper, not the script body).
- **Corrective actions for next phase**:
  - Add `2>&1 | tee >(tail -3 >&2)` or equivalent so cargo stderr surfaces in real time instead of being clipped to 3 lines.
  - Consider failing the script if `cargo build -p <name>` cannot resolve the package, with a hint to check `cargo metadata` for renames.

### Δ 2. The plan assumed `prometheus setup` could drive the rebuild

- **What we expected during planning**: `prometheus setup --non-interactive` would detect drift and rebuild.
- **What I found at execute time**: `setup.rs` detects components by *presence only* (`detect_binary`, `detect_port`, `detect_launchd`) with no staleness check. It would have reported all-green and skipped the rebuild entirely.
- **Root cause**: The prior phase (`machine-installation-2026-05-25`) built `setup` as a *first-install* tool, not a *refresh* tool. The use case "everything exists but is stale" was not in its design.
- **Corrective actions taken**:
  - Caught this during planning, before execute. Split the role: discrete `install-binaries.sh` rebuilds; `setup --check` verifies.
- **Corrective actions for next phase**:
  - Extend `prometheus setup` with a `--rebuild` flag that forces cargo build regardless of detected presence — or add per-component staleness detection (compare binary mtime vs. submodule HEAD commit time).
  - Until then, `setup` should NOT be marketed as "the install driver" — only as the verifier. Update the prior phase's reflection to flag this.

### Δ 3. `git submodule update --init --recursive` reverted the targeted `--remote` advances

- **What we expected**: After `git submodule update --remote tools/liter-llm`, a follow-up `--init --recursive` would only initialize *nested* uninitialized submodules.
- **What happened**: It also reset the just-advanced submodules back to the superproject's recorded SHAs.
- **Root cause**: `--init --recursive` without `--remote` is a "reset to gitlinks" operation, not an "initialize missing only" operation. Documented but easy to miss.
- **Corrective actions taken**: Re-applied `git submodule update --remote --init --recursive <path>` per target submodule.
- **Corrective actions for next phase**: When the plan calls for `--remote` updates, never follow with a bare `--init --recursive`. If nested init is needed, pass `--remote` to the targeted path or initialize nested submodules separately.

### Δ 4. Claude Desktop wiring is "done" but unverified at runtime

- **What we expected**: After config merge, Desktop would load the 3 new servers on next restart.
- **What we can confirm**: JSON valid, 16 servers, originals intact, absolute paths used.
- **What we cannot confirm from this session**: That Claude Desktop, with its restricted launch sandbox, can actually exec the absolute `npx` (under `~/.local/share/fnm/node-versions/v24.16.0/installation/bin/npx`) and that `mcp-remote` successfully bridges to the SSE servers, and that `liter-llm mcp --transport stdio` starts cleanly under Desktop's environment.
- **Root cause of the verification gap**: Verification requires an external action (user restart of Desktop) plus a Desktop-internal inspection (logs / server list UI) that I cannot perform from here.
- **Corrective actions for next phase**:
  - User restarts Claude Desktop and inspects the MCP server list / logs. If any of the 3 fail to start, the most likely cause is `mcp-remote` (npm dep) not yet cached on Desktop's first run — `npx -y` will fetch it, but it takes ~30s and could time out on first connect.
  - If `forge-rs` / `prometheus-knowledge` time out, fall back to pre-installing `mcp-remote` globally (`npm i -g mcp-remote`) and use its absolute path in the config.

### Δ 5. The `--skill-count` drift is "converged on the pack subset," not "converged absolutely"

- **What the plan implied**: All platforms would have the same skill count after install.
- **What is actually true**: All platforms have **the same 88 prometheus-pack skills**, but their raw directory counts still differ (94/135/335/112) because those directories hold skills from other sources (rust-skills plugin, anthropic-skills, design plugin, etc.) that this installer does not touch.
- **Root cause**: The assessment conflated "uniform pack install" with "uniform skill count." Only the former was the actual goal.
- **Corrective actions for next phase**: When measuring G-SKILL convergence in future phases, count *pack-installed* skills (look for the installer's marker file or symlink target), not total dir size.

---

## Technical Debt Introduced

| Item | Owner | When to address |
|------|-------|-----------------|
| `scripts/install-binaries.sh` masks cargo failures via `| tail -3` pipe | next refresh phase | when this same gotcha repeats |
| `prometheus setup` has no staleness detection | tools/prometheus-cli | next phase touching the CLI |
| QA-skip rule (`<3 files / config-only`) skipped a script edit that *changed install behavior* — the rule should consider blast radius, not just file count | KBD QA policy | next QA-policy revision |
| Claude Desktop wiring is not part of `install-skills-flat.sh` or `prometheus setup` — manual merge each time | tools/prometheus-cli | when next Desktop drift happens |

---

## Lessons Captured

1. **"Detection by presence" lies during refreshes.** If a binary exists but its source has drifted 149 commits, presence-based detection reports healthy. Future refresh tools must detect staleness (mtime vs. source commit time, or version-string compare).
2. **Upstream package renames are silent breakers.** A 5-day-old install script broke on two package renames. When pulling submodules with significant drift, `cargo metadata --no-deps` should be the first thing run, before any `-p <name>` invocation.
3. **`git submodule update --init --recursive` is destructive after `--remote`.** Always pair `--remote` with `--init --recursive` in the *same* command per path, never as two sequential commands.
4. **Claude Desktop's restricted launch PATH is real.** `fnm` multishell paths are PID-keyed and ephemeral; always pin to `~/.local/share/fnm/node-versions/<ver>/installation/bin/` for absolute paths in Desktop config.
5. **QA-skip by file count is the wrong axis.** A 1-line edit to an install script is higher blast-radius than a 100-line config addition. The QA rule should weigh *what the file controls*, not *how many files were changed*.

---

## Recommended Focus for Next Phase

The actual next phase should be **commit-and-push** — there are real, valuable source changes from this phase that must not be lost:

- `scripts/install-binaries.sh` package-rename fix
- Submodule pointer advances (liter-llm, surreal-memory)
- `.kbd-orchestrator/` phase artifacts
- A new untracked `entity-realtime-surreal-live` skill picked up by the pull

After commit, the **two highest-leverage follow-ups** (good candidates for a planning phase):

1. **`prometheus setup --rebuild` + staleness detection** — closes the gap that made this entire phase necessary. The next time submodules drift, `setup --rebuild` would do everything this 5-change phase did in one command.
2. **`install-skills-flat.sh` extended to Claude Desktop** — eliminate the manual JSON-merge step. A `--with-desktop` flag that merges the prometheus stack into `claude_desktop_config.json` would make future installs single-step.

---

*Reflection written to: `.kbd-orchestrator/phases/machine-refresh-2026-05-30/reflection.md`*
*Next: commit changes, then `/kbd-new-phase` for the next phase.*
