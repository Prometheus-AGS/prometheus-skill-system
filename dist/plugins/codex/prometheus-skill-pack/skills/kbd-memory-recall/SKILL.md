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
2. Resolves and probes the memory service REST origin via `kbd_memory_available` and `GET /health`.
3. If reachable: requests lifecycle entities from `GET /api/v1/entities/search?q=kbd_lifecycle_event`, decodes their string observations, and ranks them locally by same-project affinity, query-token overlap, recency, entity name, and observation index.
4. Writes the five highest-ranked matches to a markdown digest. A reachable empty result produces a normal digest with explicit empty sections.
5. If the REST service is unavailable or returns an invalid contract: writes an atomic stub `prior-context.md` so downstream skills can read the file unconditionally.

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
- `curl` available for shell-owned REST recall.
- A reachable service origin selected from an explicit override, project configuration, or the canonical local default `http://127.0.0.1:23001`.

An in-process `create_entity` MCP tool can make memory available to an agent, but it does not provide an HTTP origin to this shell skill. In that MCP-only mode the skill writes a specific stub instead of inventing a REST URL.

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
- Entity search returns an empty array → digest contains the heading and an explicit `*(no prior matches found)*` line.
- Entity search returns an HTTP error or invalid JSON contract → atomic diagnostic stub; orchestration continues.
- `goals.md`/`assessment.md` missing → query falls back to the phase name as text.

The skill always exits 0 so it composes with the `auto-memory-recall` hook's `on_failure: ignore`.
