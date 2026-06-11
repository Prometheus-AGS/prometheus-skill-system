---
id: change-002-scope-guard
title: Change-set scope guard (warn mode) + scope-record
phase: safeguards
gaps: [H2]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: done
scope:
  - shared/scripts/scope-guard.sh
  - shared/scripts/scope-record.sh
  - hooks/hooks.json
  - shared/scripts/tests/test-scope-guard.sh
---

# change-002 — Change-set scope guard

## Context

Nothing stops an execution loop from editing files outside the active change's
declared `scope:`. Every change since Phase 1 has carried a `scope:` block in
its frontmatter precisely so this guard can enforce it retroactively. Ships in
warn mode (user decision) — observe before blocking.

## Scope

In:

- waypoint gains `scoped_paths: [...]` (globs copied from the active change's
  `scope:` when a change activates) and `scope_overrides: [{path, reason, approvedAt}]`.
- New `shared/scripts/scope-guard.sh` (PreToolUse Write|Edit|MultiEdit):
  - Read `file_path` from stdin; walk up to `.kbd-orchestrator`; read
    `scoped_paths` + `scope_overrides` from waypoint.
  - No orchestrator / no active change / empty scoped_paths → exit 0.
  - Always-allowed: `.kbd-orchestrator/**`, `SCRATCHPAD.md`.
  - In scope (python3 fnmatch over globs) or already overridden → exit 0.
  - Out of scope: `PROMETHEUS_SCOPE_ENFORCE=off` → exit 0; `warn` (default) →
    print a non-blocking notice to stderr, exit 0; `ask` → emit
    `permissionDecision:"ask"` JSON (fallback exit 2 + message if JSON fails).
- New `shared/scripts/scope-record.sh` (PostToolUse Write|Edit|MultiEdit, `|| true`):
  if the just-written path was out-of-scope, append a `scope_overrides` entry to
  the waypoint so it is not re-flagged.
- Wire both into hooks.json (scope-guard after protect-tests in PreToolUse;
  scope-record in the PostToolUse group).
- New `shared/scripts/tests/test-scope-guard.sh`: in-scope pass; out-of-scope
  warn (stderr notice, exit 0); out-of-scope ask (ask-JSON); always-allowed
  paths; no-state pass; scope-record adds override and silences re-flag.

## Tasks

- [x] 1. Write scope-guard.sh + scope-record.sh (+x)
- [x] 2. Wire both into hooks.json
- [x] 3. Write test covering off/warn/ask + record; run green

## Verification

Test green; warn mode never blocks (exit 0 with notice); ask mode emits ask-JSON;
scope-record persists an approved expansion once.
