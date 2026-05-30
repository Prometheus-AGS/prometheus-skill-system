# Reflection: setup-rebuild-and-staleness-2026-05-30

**Date**: 2026-05-30
**Executor**: claude-code (claude-opus-4-8 → claude-sonnet-4-6)
**Backend**: openspec
**Previous phase**: machine-refresh-2026-05-30

---

## Goal Achievement

| Goal | Status | Evidence |
|------|--------|----------|
| `--rebuild` flag forces cargo build+install of all 4 binaries | ✅ MET | `prometheus setup --rebuild` rebuilt prometheus, forge, pk-cherry, liter-llm from source; fresh mtimes 16:43–16:50 confirmed |
| Per-component staleness detection (binary mtime vs submodule HEAD commit time) | ✅ MET | `source_commit_time_for`, `binary_mtime`, `is_stale` implemented; synthetic stale test confirmed detection |
| Stale surfaces as `ComponentStatus::Stale` with UX | ✅ MET | ⚠️ yellow icon, "stale (source newer than binary)" label, `needs_action()=true`, serde round-trip |
| `--check` reports stale separately from missing | ✅ MET | Output: "X gap(s) detected: Y missing, Z stale." with distinct colors |
| Unit tests cover the staleness comparator | ✅ MET | 8/8 tests pass: `is_stale_returns_true/false/equal`, `source_commit_time_for_unknown_binary_returns_none`, `stale_status_serializes_snake_case`, plus 3 original tests extended |

**Overall: 5/5 goals MET (100%)**

---

## Delivered Changes

| # | Change ID | Status | What |
|---|-----------|--------|------|
| 1 | change-staleness-001-add-stale-component-status | ✅ DONE | `Stale` enum variant, icon/label/needs_action impl, serde test |
| 2 | change-staleness-002-add-staleness-comparator | ✅ DONE | `is_stale` (pure), `repo_root`, `binary_mtime` (pinned to ~/.local/bin/), `git_commit_time`, `source_commit_time_for`, `binary_is_stale` + 4 unit tests |
| 3 | change-staleness-003-wire-staleness-into-detection | ✅ DONE | 4 detect fns now return `Stale` when binary present and source newer; smoke-driven bug found and fixed |
| 4 | change-staleness-004-add-rebuild-flag-and-installers | ✅ DONE | `--rebuild` clap flag; 4 install fns; `cargo_build_and_install` shared helper; `kickstart_or_warn`; REBUILD_TARGETS scope; `run()` signature extended |
| 5 | change-staleness-005-improve-check-output-grouping | ✅ DONE | Implemented inline during change-004; missing_count + stale_count separation |
| 6 | change-staleness-006-verify-and-integration-test | ✅ DONE | Synthetic stale (touch -t 202001010000) detected; --rebuild fixed it; kickstarts fired; `✨ All components healthy` |

---

## Artifact Quality Summary

| Metric | Value |
|--------|-------|
| Changes with QA (artifact-refiner) | 0/6 |
| Changes skipped (<3 source files or config-only) | 6/6 |
| Unit tests added this phase | 5 new (12 total in module) |
| Pre-existing clippy errors blocked | 2 (in sibling crates, unrelated) |

> **QA note**: All changes touched 1–2 source files (setup.rs + main.rs). Per the execution plan these were "opt-in QA" cases, but artifact-refiner was not formally invoked. The reflection from machine-refresh-2026-05-30 identified that the QA-skip rule (file count < 3) is on the wrong axis. That tech debt carries forward here: a 477-line behavior-critical file should qualify for QA regardless of how many files changed.

---

## Delta → Root Cause → Corrective Actions

### Δ 1. `which`-based `binary_mtime` picked up stale `/usr/local/bin/` leftovers

- **Expected**: `binary_mtime("prometheus")` returns `~/.local/bin/prometheus` mtime (today).
- **Happened**: First implementation shelled `which prometheus` → resolved to `/usr/local/bin/prometheus` (April 30 leftover), reported mtime `1777550359` → Stale false-positive.
- **Root cause**: Two `prometheus` binaries on PATH; `which` returns the first match. Our install target (`~/.local/bin/`) is not guaranteed to be earlier on PATH than `/usr/local/bin/`.
- **Fix**: Pinned `binary_mtime` to `~/.local/bin/<name>` directly via `dirs::home_dir().join(".local/bin")`. The install target is always `~/.local/bin/`; the detector now checks that path explicitly.
- **Corrective for future**: Document that `/usr/local/bin/{prometheus,forge}` are stale unrelated binaries and should be removed. They'll continue to shadow the correct binaries in shells that haven't loaded `~/.local/bin/` before `/usr/local/bin/`.

### Δ 2. `--rebuild` initially forced all installable components (7), not just the 4 binaries

- **Expected** (per locked decision): `--rebuild` rebuilds all 4 build-from-source binaries.
- **Happened**: First impl used `rebuild || status.needs_action()` for all components with `install: Some(...)`, producing 7 "would rebuild" actions (including launchd plists and template-forge).
- **Root cause**: The `install: Some(...)` field is shared by plist-loaders, binary-builders, and template-forge — they're all "installable" but have different semantics.
- **Fix**: Added `REBUILD_TARGETS: &[&str]` constant scoping `--rebuild` to the 4 staleness-tracked binaries. Plist loaders (forge-mcp, pk-mcp) are refreshed indirectly via kickstart from the binary installers.
- **No corrective needed**: The REBUILD_TARGETS constant is explicit and easy to maintain.

### Δ 3. Pre-existing clippy errors in sibling crates block full-workspace lint

- **Expected**: `cargo clippy -p prometheus-cli --bins -- -D warnings` runs clean.
- **Happened**: `prometheus-agents` (`new_without_default`) and `prometheus-learn` (`unnecessary_sort_by`) are dependencies that fail clippy, cascading the failure to our target.
- **Root cause**: Pre-existing issues from May 6 and May 9 commits in sibling crates. Not introduced by this phase.
- **Impact**: Can't achieve full-workspace clippy-clean. The prometheus-cli binary itself compiles and runs cleanly.
- **Corrective action for next phase**: Fix `prometheus-agents/src/trace_protocol.rs:61` (add `impl Default`) and `prometheus-learn/src/trace.rs:149` (`sort_by` → `sort_by_key`) — each is a 1–2 line fix.

### Δ 4. Change-005 (output grouping) was subsumed into change-004

- **Plan**: changes 004 and 005 were listed as separate.
- **Happened**: The missing/stale count split was written inline during the `run()` rewrite. Separate commit would have required re-touching the same function.
- **Root cause**: `run()` is a monolithic function; the output-grouping code is entangled with detection-result iteration.
- **Impact**: Zero — change-005 is done. But the separation in the plan was aspirational, not achievable given the code structure.
- **Corrective**: No action. Future: if `run()` grows further, extract a `print_status_table()` helper to make the output logic independently editable.

---

## Technical Debt Introduced

| Item | Severity | Where | When to Address |
|------|----------|-------|-----------------|
| `binary_mtime` hardcodes `~/.local/bin/` — won't work if install target changes | Low | `setup.rs` | If install dir becomes configurable |
| QA-skip rule (file count < 3) still active — didn't apply artifact-refiner to a 500-line behavior file | Medium | KBD QA policy | Next QA-policy revision (prior reflection already flagged this) |
| Pre-existing clippy: `prometheus-agents` + `prometheus-learn` | Medium | sibling crates | Short fix in a dedicated cleanup phase |
| Stale `/usr/local/bin/{prometheus,forge}` leftovers on machine | Low | machine state | User action: `rm /usr/local/bin/prometheus /usr/local/bin/forge` |
| `run()` is 100+ lines — output grouping, install loop, state-write all in one fn | Low | `setup.rs` | Refactor phase if function grows further |

---

## Lessons Captured

1. **Shell `which` is not a stable anchor for mtime lookups** — always pin to the known install target directory (`~/.local/bin/<name>`) rather than let PATH resolution pick an unexpected binary. PATH order is machine-specific and mutable.

2. **Shared `install: Option<fn>` field conflates plist-loaders and binary-builders** — the `Component` struct doesn't distinguish "install meaning: build from source" from "install meaning: load a plist". When adding selective behavior (`--rebuild` for source binaries only), this ambiguity forces an explicit allowlist (`REBUILD_TARGETS`). A more type-safe design would use an enum: `ComponentInstaller::Cargo { pkg, bin }` vs `ComponentInstaller::Launchd { plist }`.

3. **Tests first, then wiring** — changes 001 and 002 (enum variant + comparator) completed and tested before change 003 wired them in. This meant the integration step had a clear, pre-validated contract. Pure functions are easy to test in isolation.

4. **Smoke tests surface real environment state immediately** — the first smoke after change-003 caught the `/usr/local/bin/` conflict within seconds. Smoke-testing against the live machine (not just unit tests) is essential for install tooling that interacts with PATH and filesystem state.

5. **`is_stale(binary_mtime, source_commit_time) -> bool` kept pure** — the comparator function has no IO side effects. This let us test it with literal `SystemTime` values, no filesystem, no git. The IO functions (`binary_mtime`, `git_commit_time`) return `Option<SystemTime>` and are tested separately with the "returns None on unknown id" case. Keeping IO at the boundary of the pure core is the right pattern for testability.

---

## Recommended Focus for Next Phase

**Highest priority: clean up the clippy debt in sibling crates.** Both fixes are trivial:
- `prometheus-agents/src/trace_protocol.rs:61` — add `impl Default for ClaudeCodeTraceCapture`
- `prometheus-learn/src/trace.rs:149` — `sort_by` → `sort_by_key(|b| std::cmp::Reverse(b.timestamp))`

This unblocks `cargo clippy --workspace -D warnings` and ensures future CI can run lint clean.

**Second priority: `Component` enum refactor** — distinguish plist-loaders from cargo-builders at the type level. This eliminates the `REBUILD_TARGETS` allowlist and makes the `--rebuild` scope self-documenting.

**Third: commit and push** — the Rust source changes in this phase (`setup.rs`, `main.rs`) are not yet committed.

---

*Reflection written to: `.kbd-orchestrator/phases/setup-rebuild-and-staleness-2026-05-30/reflection.md`*
*Next: commit changes, then `[kbd] Reflection complete — advance to next phase with /kbd-new-phase`*
