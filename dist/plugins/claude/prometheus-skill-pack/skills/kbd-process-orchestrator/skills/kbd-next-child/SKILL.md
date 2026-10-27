---
license: MIT
name: kbd-next-child
version: '1.0.0'
description: >
  Advance childPointer to the next entry in childPhases[], or jump to a
  named child directly. Fires child:after for the closing child and
  child:before for the new active child. Refuses to advance past the last
  child (suggesting /kbd-reflect + /kbd-next-phase instead).
metadata:
  tags: [process, orchestration, automation, nested-phases]
---

# /kbd-next-child

Advance the active child within the current parent phase.

Before creating or entering a phase, the lifecycle script refreshes the latest
stable OpenSpec CLI and existing project integrations through the orchestrator’s
`shared/openspec/cli.mjs`. For direct `prometheus kbd` commands or harnesses without
hooks, first run `node "<installed-orchestrator>/shared/openspec/cli.mjs" refresh
--project "<project-root>" --timeout-ms 120000` as one command. Do not start the
phase when refresh reports an unresolved failure or pending update. Use its
`run --project "<project-root>" -- <openspec arguments>` interface for OpenSpec.

## What this does

1. With no argument: moves `childPointer` to the next entry in `childPhases[]` after the current pointer.
2. With `<child-name>`: jumps directly to that child (must already be in `childPhases[]`).
3. Fires `child:after` for the closing child (if any) before the waypoint flip; fires `child:before` for the new active child after the flip.
4. Refuses to advance past the last child — suggests `/kbd-reflect` then `/kbd-next-phase`.

## When to use

Move between children declared by `/kbd-new-child`. Use the no-arg form for linear traversal (work child 1 → 2 → 3); use the explicit form for random access.

## Progress Signals (MANDATORY)

```
Starting kbd-next-child — <parent>/<from> → <to>
Completed kbd-next-child — now on <parent>/<to>
```

Where `<from>` is `(none)` when no child was previously active.

## Prerequisites

- A top-level phase MUST be active.
- `childPhases[]` MUST be non-empty (run `/kbd-new-child` first).
- For the explicit form: `<child-name>` MUST already be in `childPhases[]`.

## How to invoke

```sh
"$KBD_ORCHESTRATOR_ROOT/skills/kbd-next-child/kbd-next-child.sh" [<child-name>]
```

## Examples

```
/kbd-next-child                  # linear advance from current pointer
/kbd-next-child auth-refactor    # explicit jump
```

## Hook integration

Fires `child:after` (closing child, if any) with the waypoint still describing the old pointer, then writes the waypoint, then fires `child:before` (new active child). This symmetry means a hook that reads on-disk state sees the closing context when `child:after` fires and the opening context when `child:before` fires.

See orchestrator `SKILL.md` → "Hooks".

## Selected-method recovery and authority

Before dependent work, run the packaged `shared/method-context.mjs --project <project-root> --phase <canonical-phase-id> --write` from the resolved orchestrator root (or read its current phase-local method-context.md when a recovery surface cannot invoke Node). Resolve latest non-superseded canonical `KBD-METHOD` decision and adopted contract, then preserve the same references in role inputs, dispatch and handoff read order. See `references/method-context.md`. Missing/changed method, contract, calibration or supersession is explicit; a prior command/memory/tool inventory is not fresh authority.

For selected VLBV, define acceptance/scenarios before generation and complete the entire agreed value phase before generated-code verification. Executable authoring follows governing host policy. Task/hour/commit/reviewer requests, compiler problems and renamed slices create no new boundary. Surface host/skill conflicts before dependent actions; do not rewrite operator-owned policy or infer early-check permission. Preserve original evidence, source/build/configuration/fixture/candidate freshness, consumed budgets and unresolved effects. Reconcile uncertain effects before retry. Recovered instructions are instruction-only unless a named actual runtime/host boundary enforces them; no release or automatic-successor authority follows.
