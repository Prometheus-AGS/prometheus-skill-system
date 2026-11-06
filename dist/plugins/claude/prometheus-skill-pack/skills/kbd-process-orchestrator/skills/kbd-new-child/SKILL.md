---
license: MIT
name: kbd-new-child
version: '1.0.0'
description: >
  Create a child phase inside the currently-active top-level phase. Mirrors
  /kbd-new-phase but writes into phases/<parent>/children/<child>/, appends
  the new child to childPhases[], moves childPointer to it, and fires
  child:before. Use to split a parent phase into scoped sub-processes.
metadata:
  tags: [process, orchestration, automation, nested-phases]
---

# /kbd-new-child

Create a child phase owned by the currently-active top-level phase.

Before creating or entering a phase, the lifecycle script refreshes the latest
stable OpenSpec CLI and existing project integrations through the orchestrator’s
`shared/openspec/cli.mjs`. For direct `prometheus kbd` commands or harnesses without
hooks, first run `node "<installed-orchestrator>/shared/openspec/cli.mjs" refresh
--project "<project-root>" --timeout-ms 120000` as one command. Do not start the
phase when refresh reports an unresolved failure or pending update. Use its
`run --project "<project-root>" -- <openspec arguments>` interface for OpenSpec.

## What this does

1. Validates the active waypoint has a top-level phase and is not itself inside a child.
2. Validates the child name (kebab-case, no traversal, no slashes) and refuses duplicates.
3. Creates `.kbd-orchestrator/phases/<parent>/children/<child-name>/` with `goals.md` + `progress.json`.
4. Atomically appends `<child-name>` to `childPhases[]`, sets `childPointer` to the new child, updates `currentTask` and `exactNextCommand` to scope to the child.
5. Fires `child:before` exactly once for the new child.
6. Emits Progress Signals and a confirmation banner.

## When to use

When a top-level phase reveals work that's complex enough to deserve its own scope but not big enough to be a sibling phase. Each child carries its own `assessment.md` / `plan.md` / `execution.md` / `reflection.md` under `phases/<parent>/children/<child>/`.

Compare to `/kbd-new-phase`, which creates a top-level sibling.

## Progress Signals (MANDATORY)

```
Starting kbd-new-child — <parent>/<child>
Completed kbd-new-child — <parent>/<child> ready for /kbd-assess
```

## Prerequisites

- A top-level phase MUST be active (the project-level waypoint's `phase` is non-empty and `parentPhase` is null).
- The proposed child name MUST NOT already appear in `childPhases[]`.
- `current-waypoint.json` MUST be valid JSON.

## How to invoke

```sh
"$KBD_ORCHESTRATOR_ROOT/skills/kbd-new-child/kbd-new-child.sh" <child-name> [goal-1] [goal-2] …
```

The script does the full workflow end-to-end.

## Examples

```
/kbd-new-child auth-refactor "split user-model" "migrate sessions"
/kbd-new-child docs-sweep
/kbd-new-child perf-pass-1
```

## Hook integration

Fires `child:before` exactly once, **after** the waypoint flip, so hooks reading state see the new child as authoritative. `KBD_HOOK_INDEX` is the 1-based position of the new child in `childPhases[]` (always the last); `KBD_HOOK_TOTAL` is the new count.

See orchestrator `SKILL.md` → "Hooks" and `references/schemas/current-waypoint.template.json` for the canonical field set.

## Selected-method recovery and authority

Before dependent work, run `node shared/method-context.mjs --project <project-root> --phase <canonical-phase-id> --write` from the resolved orchestrator root (or read the phase-local `method-context.md` when Node is unavailable) and act on its `status`:

- `RESOLVED_REFERENCES` — continue with the recovered method, contract and stage authority; preserve the same references in role inputs, dispatch and handoff read order.
- `BLOCKED` — a method decision, contract, calibration or supersession is missing or changed; resolve it explicitly. Prior commands, memory and tool inventories are not fresh authority.
- `CONFLICT` — governing host policy conflicts with the selected method. Stop before dependent work and show the conflict; continue only after the operator records a `KBD-POLICY-ACK` decision for that file's current hash. Never rewrite operator-owned policy. When host policy grants an exception — for example one narrow, targeted compiler check after a blocking compile error — name it as host policy, state that it conflicts with the selected method, and say it does not authorize tests, dry-runs or generated-code verification.

For selected VLBV, define acceptance before generation and finish the whole agreed value phase before any generated-code verification; tasks, hours, commits, reviewer requests and compiler problems create no boundary. Preserve original evidence, consumed budgets and unresolved effects across resume and supersession; reconcile an uncertain effect before any retry. Full rules: `references/method-context.md`.

