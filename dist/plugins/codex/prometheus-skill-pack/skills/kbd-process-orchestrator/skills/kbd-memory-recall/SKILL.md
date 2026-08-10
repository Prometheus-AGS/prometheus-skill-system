---
license: MIT
name: kbd-memory-recall
version: '1.0.0'
description: >
  Query surreal-memory for prior similar KBD work and write a markdown
  digest at .kbd-orchestrator/phases/<phase>/prior-context.md. Used as
  planning input before /kbd-assess. Auto-invoked via the assess:before
  hook; degrades gracefully when the memory endpoint is unreachable.
metadata:
  tags: [process, orchestration, memory, learning]
---

# /kbd-memory-recall

Populate `prior-context.md` for the active phase from surreal-memory.

## What this does

1. Resolves the target phase (argument, or active phase from waypoint).
2. Detects whether the memory endpoint is reachable via `kbd_memory_available`.
3. If reachable: reads `goals.md` + `assessment.md` (when present) as the query, calls `find_relevant` for `entityType = "kbd_lifecycle_event"`, writes a markdown digest with the top matches and their phase paths.
4. If unreachable: writes a stub `prior-context.md` so downstream skills can read the file unconditionally.

## When to use

Before `/kbd-assess` runs, so the planner has prior context. The built-in `auto-memory-recall` hook invokes this skill automatically on `assess:before`; manual invocation is also supported for re-running with different criteria.

## Progress Signals (MANDATORY)

```
Starting kbd-memory-recall — <phase>
Completed kbd-memory-recall — <phase> wrote prior-context.md
```

## Prerequisites

- Phase directory `.kbd-orchestrator/phases/<phase>/` MUST exist.
- `jq` available.
- Either `curl` (for HTTP mode) or a calling tool that supplies `KBD_AVAILABLE_TOOLS` containing `create_entity` (for MCP mode).

## How to invoke

```sh
"$KBD_ORCHESTRATOR_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh" [<phase>]
```

## Examples

```
/kbd-memory-recall                          # active phase from waypoint
/kbd-memory-recall submodule-foo-bar        # explicit phase
```

## Output digest format

```
# Prior context — <phase>

> Auto-populated by /kbd-memory-recall. Replace or extend if needed.

## Most relevant prior phases (top 5)

1. **<project>/<prior-phase>** — <kind> @ <ts>
2. ...

## Patterns observed

- ...
```

## Failure modes

- Memory endpoint unreachable → stub digest:
  `<!-- memory endpoint unreachable; no prior context retrieved -->`
- `find_relevant` returns no matches → digest contains the heading and an explicit `*(no prior matches found)*` line.
- `goals.md`/`assessment.md` missing → query falls back to the phase name as text.

The skill always exits 0 so it composes with the `auto-memory-recall` hook's `on_failure: ignore`.
