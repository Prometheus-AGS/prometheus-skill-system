# Execution: machine-refresh-2026-05-30

**Backend selected**: `openspec` (operational changes, executed natively in Claude Code)
**Executor**: claude-code (claude-opus-4-8)
**Dispatch**: direct — these are install/config operations on the local machine; no
cross-tool handoff. KBD remains the source of truth via progress.json.

## Why native dispatch (not another tool)

All 5 changes are local machine operations (git pull, cargo build, file installs, JSON
merge, health probes) that must run on *this* host where the binaries and configs live.
Handing off to Roo/Cursor/etc. would not help — there is no parallelizable source-spec work.

## Execution order (strict)

1. change-refresh-001-pull-repo-and-submodules
2. change-refresh-002-rebuild-and-reinstall-binaries   (depends 001)
3. change-refresh-003-reinstall-skills-all-platforms   (depends 001)
4. change-refresh-004-wire-claude-desktop-mcp          (depends 002)
5. change-refresh-005-verify-refresh                   (depends 002,003,004)

## QA gate disposition (per prior-phase convention)

| Change | Files touched | QA |
|--------|---------------|-----|
| 001 | git state only | skip (no source files) |
| 002 | binaries rebuilt from submodule source; no repo source edits unless build fails | skip unless rust-build-resolver edits source |
| 003 | skill symlinks via script | skip (script-driven, no source) |
| 004 | 1 config file (claude_desktop_config.json) | skip (<3 files, config-only) |
| 005 | verification report only | skip (no source) |

artifact-refiner QA is invoked only if change-002 requires source edits (build failures).

## Verification gate

Final gate is `prometheus setup --check` + `prometheus doctor` + port probes in change-005.
