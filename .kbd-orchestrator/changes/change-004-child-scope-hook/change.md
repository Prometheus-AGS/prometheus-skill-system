---
id: change-004-child-scope-hook
title: check-child-scope.sh advisory child-permission hook
phase: child-loops-and-capabilities
gaps: [C4]
priority: P2
effort: M
agent: claude-code
evolver_item_id: null
status: done
scope:
  - skills/process/kbd-process-orchestrator/shared/lib/check-child-scope.sh
  - hooks/hooks.json
  - shared/scripts/tests/test-child-scope.sh
---

# change-004 — Child scope enforcement hook

## Context

A child loop's scope.json (change-002) declares its allowedWritePaths, but
nothing enforces it — an inner loop can still edit anything. This hook makes the
isolation real (advisory, hook-level).

## Scope

In:

- New `KBD/shared/lib/check-child-scope.sh` (PreToolUse Write|Edit|MultiEdit):
  - When the waypoint path[] has depth > 1 (inside a child), read the current
    child node's scope.json.
  - Block/notice writes to file paths outside allowedWritePaths (and inside
    deniedPaths). Canonicalize paths (reuse the is_descendant cd&&pwd-P rule
    from Phase 3) so macOS symlinks don't defeat matching.
  - `.kbd-orchestrator/**` and SCRATCHPAD.md always allowed.
  - `PROMETHEUS_CHILD_SCOPE_ENFORCE=off|warn|ask`, default warn (notice, exit 0);
    ask emits permissionDecision:ask JSON.
  - No child active / no scope.json / off → exit 0.
- Wire into root hooks.json PreToolUse Write|Edit|MultiEdit group (after
  scope-guard).
- New `shared/scripts/tests/test-child-scope.sh`: depth-1 waypoint → no-op;
  child with scope.json → in-scope pass, out-of-scope warn/ask; always-allowed
  paths.

## Tasks

- [x] 1. Write check-child-scope.sh (path[] depth check, scope.json match, modes)
- [x] 2. Wire into hooks.json
- [x] 3. Write test; run green

## Verification

Test green; an inner-loop write outside the child's allowedWritePaths is flagged
in warn mode; top-level writes are never affected.
