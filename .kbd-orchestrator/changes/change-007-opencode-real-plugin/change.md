---
id: change-007-opencode-real-plugin
title: Real .opencode/plugin.ts with Plugin function export
phase: phase-compliance-and-power-multiplier
gaps: [D1, D2, D3]
priority: P1
effort: S
agent: typescript-reviewer
evolver_item_id: null
status: proposed
---

# change-007 — Real OpenCode Plugin

## Context

`.opencode/tools/{evolve,gitops,kbd}.ts` define typed *tool* objects, but the
OpenCode plugin spec ([opencode.ai/docs/plugins](https://opencode.ai/docs/plugins/))
requires a JS/TS module that exports a **Plugin function** (default or named) that
receives a context object and returns a hooks object. We currently have neither
that file nor the required `@opencode-ai/sdk` dependency. As a result, OpenCode
loads our `package.json` but doesn't actually wire any of the tools or hooks.

## Scope

In:

- New `.opencode/plugin.ts`:
  - Default export: a Plugin function that takes a context object and returns
    `{ hooks, tools }`.
  - `tools` aggregates the existing three tool files (`evolve`, `gitops`,
    `kbd`).
  - `hooks` mirrors the lifecycle hooks defined in `hooks/hooks.json` —
    specifically: equivalent of SessionStart (project-context detection),
    PostToolUse for write/edit, and Stop.
  - The hooks call out to the same `shared/scripts/*.sh` scripts that the
    Claude Code hooks use, so behavior is consistent across platforms.
- Update `.opencode/package.json` to add `@opencode-ai/sdk` to dependencies.
- Update `scripts/install-platforms.ts` so `--platform opencode` writes an
  `opencode.json` to the project root with `"plugin": ["./.opencode"]` (or to
  `~/.config/opencode/opencode.json` for global install).
- Add a tiny `.opencode/README.md` documenting how to test the plugin.

Out:

- New OpenCode-specific functionality — this is purely a wiring fix, not a
  feature add.

## Deliverables

1. Working `.opencode/plugin.ts` exporting a Plugin function.
2. Updated package.json with sdk dep.
3. Auto-generated `opencode.json` on platform install.

## Acceptance Criteria

- After `npm run install:opencode`, an OpenCode session in the same project
  loads the plugin without errors and the three tools (`evolve`, `gitops`,
  `kbd`) are listed.
- `opencode --print-config` shows the plugin entry under `plugin: [...]`.
- The PostToolUse hook fires on a Write+Edit operation in OpenCode (verified
  by checking that `validate-state.sh` ran).

## Files to Touch

- `.opencode/plugin.ts` (new)
- `.opencode/package.json` (add sdk)
- `.opencode/README.md` (new)
- `scripts/install-platforms.ts` (auto-generate opencode.json)

## Test Plan

- Unit: TypeScript type-check (`tsc --noEmit`) on the new plugin file.
- Integration: install in a sandbox project, run `opencode` CLI, verify tools
  list includes our three.
- Regression: confirm the existing Claude Code path still works (i.e. installing
  for opencode does not pollute `.claude/`).
