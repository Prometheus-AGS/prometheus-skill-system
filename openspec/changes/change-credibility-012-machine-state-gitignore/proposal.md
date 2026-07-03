---
id: change-credibility-012-machine-state-gitignore
title: Remove machine state files from tracking and add to .gitignore
phase: phase-credibility-closure
priority: P2
effort: S
wave: 3
parallel: true
agent: claude
status: done
gap_id: P2-E
verdict: BUILD
scope:
  - .gitignore
---

# change-credibility-012 — Remove machine state files from tracking and add to .gitignore

## Context

Files under `.prometheus/` are machine-local runtime state (e.g., `reflect-rejections.txt`). Files like `.kbd-orchestrator/**/project.json` contain environment-specific paths. Both categories are not portable across machines and should not be version-controlled.

The git status shows these as untracked; this change prevents them from being accidentally committed.

## Scope

1. Add the following patterns to `.gitignore`:
   - `.prometheus/`
   - `.kbd-orchestrator/**/project.json`
   - `SCRATCHPAD.md`
   - `.envrc`
   - `*.local.env`
2. If any files matching these patterns are currently tracked by git, un-track them with `git rm --cached`

## Implementation Notes

Append to `.gitignore`:
```gitignore
# Machine-local runtime state
.prometheus/
.kbd-orchestrator/**/project.json

# Session scratchpads (per CLAUDE.md XC-003)
SCRATCHPAD.md

# Local env overrides
.envrc
*.local.env
```

Check for currently tracked files:
```bash
git ls-files .prometheus/ .kbd-orchestrator/ SCRATCHPAD.md 2>/dev/null
```

If any are tracked, run `git rm --cached <path>` for each.

## Verification

- `.gitignore` contains all added patterns
- `git status` does not show `.prometheus/` or `project.json` files as untracked (they are now ignored)
- `git ls-files .prometheus/` returns empty
