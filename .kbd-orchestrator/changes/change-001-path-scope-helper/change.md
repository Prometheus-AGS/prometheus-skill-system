---
id: change-001-path-scope-helper
title: Shared path-scope-match helper; refactor 3 hooks onto it
phase: outer-loop-and-ux
gaps: [U1]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: proposed
scope:
  - shared/scripts/lib/path-scope.sh
  - shared/scripts/scope-guard.sh
  - skills/process/kbd-process-orchestrator/shared/lib/check-child-scope.sh
  - shared/scripts/protect-tests.sh
  - shared/scripts/tests/test-path-scope.sh
---

# change-001 — Shared path-scope helper

## Context

The recurring path-matching bug class (macOS /var symlinks, heredoc-in-$(),
glob over-broadening to "**") bit this work THREE times because scope-guard,
check-child-scope, and protect-tests each re-implement canonicalize +
relativize + fnmatch separately. Extract one verified helper.

## Scope

In:

- New `shared/scripts/lib/path-scope.sh` (sourceable):
  - `pscope_relativize <root> <file>` — canonicalize both via cd&&pwd-P
    (macOS-safe), echo the repo-relative path (or the input when outside root).
  - `pscope_match <rel> <globs-json>` — python fnmatch over a JSON array of
    globs; echo "in" or "out". No prefix-stripping (the glob-broadening bug).
  - `pscope_always_allowed <rel>` — true for `.kbd-orchestrator/**`, SCRATCHPAD.md.
- Refactor the three hooks to source it and call these functions, REMOVING
  their inline _canon/REL/fnmatch blocks. Observable behavior unchanged — each
  hook's existing test must stay green untouched.
- New `shared/scripts/tests/test-path-scope.sh`: relativize handles symlinked
  roots; match is exact (no over-broadening: src/feature/** does NOT match
  src/other/x); always-allowed paths.

## Tasks

- [ ] 1. Write path-scope.sh (relativize + match + always_allowed)
- [ ] 2. Refactor scope-guard, check-child-scope, protect-tests onto it
- [ ] 3. Write test-path-scope.sh; run it + all 3 hook tests green

## Verification

test-path-scope.sh green; test-scope-guard.sh, test-child-scope.sh,
test-protect-tests.sh all still green (behavior unchanged).
