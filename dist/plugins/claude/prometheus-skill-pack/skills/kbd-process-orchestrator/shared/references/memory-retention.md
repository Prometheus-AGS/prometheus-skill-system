# Memory retention + recall policy

Reference for the surreal-memory integration introduced by
`ssed-kbd-memory-first-execution`. Consumed by `/kbd-memory-recall` and the
`kbd-memory-log` hook. Server-side retention is configured separately in
your surreal-memory-server deployment — this document defines the *KBD
side* of the contract.

## Canonical entity: `kbd_lifecycle_event`

Every KBD hook fire produces one entity:

| Field | Type | Meaning |
|---|---|---|
| `name` | string | `<project>/<phase>/<kind>/<edge>/<index>/<ts>` — stable for the same hook fire. |
| `entity_type` | string | Always `"kbd_lifecycle_event"`. |
| `observations` | array of strings | One compact JSON lifecycle observation. The installed entity contract does not accept object-valued observations. |
| decoded observation `kind` | string | `phase` / `child` / `plan` / `execute` / `reflect` / `task` / `assess`. |
| decoded observation `edge` | string | `before` / `after`. |
| decoded observation `name` | string | Active item's canonical name (phase name, change id, task title). |
| decoded observation `index` | integer | 1-based index within the containing loop. |
| decoded observation `total` | integer | Loop total. |
| decoded observation `phase` | string | Root phase identifier. |
| decoded observation `phasePath` | string | Rendered chain (`parent › child`). |
| decoded observation `sourceTool` | string | Tool that emitted the event (claude-code, codex, …). |
| decoded observation `project` | string | Project identifier from optional `project.json`, otherwise the signed waypoint. |
| decoded observation `ts` | string | ISO-8601 UTC timestamp. |

The shell writer creates no graph relations. Project and phase ownership live
inside the encoded observation and entity name. Readers tolerate additional
observation fields and legacy object observations during migration.

## Retention

- **Default window**: 365 days from `ts`.
- **Retention is the server's job**, not the writer's. The hook writes every
  event; the surreal-memory-server enforces the window via its own retention
  policy.
- **Sensitive content**: the memory writer captures ONLY the structured
  `KBD_HOOK_*` payload — never the stderr stream of third-party hooks, which
  may contain secrets.

## Relevance ordering for recall

`/kbd-memory-recall` retrieves the lifecycle-event class through canonical
entity search and ranks decoded observations locally by:

1. **Same `project`** (highest — same codebase, same constraints).
2. **Token overlap** with the active phase, goals, and assessment.
3. **Recency** (newer first).
4. **Entity name and observation index** as stable tie-breakers.

The top five observations are rendered. Cross-project candidates remain
eligible but rank after same-project candidates; no undocumented CLI flag is
required.

## Canonical local HTTP contract

- Availability: `GET <origin>/health`
- Lifecycle write: `POST <origin>/api/v1/entities`
- Lifecycle recall: `GET <origin>/api/v1/entities/search?q=kbd_lifecycle_event`

MCP discovery URLs such as `<origin>/mcp/sse` are normalized to `<origin>`
before these REST paths are appended.

## Probe + endpoint discovery

`kbd_memory_available()` (in `shared/lib/memory.sh`) resolves and probes in
this order:

1. `$UAR_MEMORY_MCP_URL`, then `$KBD_MEMORY_MCP_URL`.
2. `.kbd-orchestrator/memory.config.json` → `restEndpoint`, then legacy `mcpEndpoint`.
3. Canonical local default `http://127.0.0.1:23001`.
4. Normalize an HTTP(S) value to its origin and probe `GET <origin>/health` with bounded timeouts.
5. If no REST origin is reachable but the calling agent advertises `create_entity`, report MCP-only availability without exposing a fabricated HTTP URL.

A negative probe is cached for the lifetime of the calling process; a new
process re-probes. There is no on-disk staleness cache to manage.

## Failure semantics

| Failure | KBD-side behavior |
|---|---|
| Endpoint unreachable | `kbd-memory-log` no-ops; `/kbd-memory-recall` writes stub digest. |
| Endpoint returns 5xx | Single stderr warning; no retry within the same hook fire; dispatch continues. |
| Endpoint returns 4xx | Single stderr warning; dispatch continues (likely a payload issue — should be reported but not propagated). |
| jq missing | `kbd-memory-log` and `/kbd-memory-recall` no-op with a warning. |
| curl missing | Same as above. |

## Configuration knobs

| Knob | Effect |
|---|---|
| `UAR_MEMORY_MCP_URL` / `KBD_MEMORY_MCP_URL` | Explicit HTTP(S) service or MCP-transport URL; normalized to its origin. |
| `.kbd-orchestrator/memory.config.json` `restEndpoint` | Preferred project REST service URL. |
| `.kbd-orchestrator/memory.config.json` `mcpEndpoint` | Legacy project discovery URL; normalized to the REST origin. |
| Project hook entry `id: "auto-memory-recall"` `enabled: false` | Disable automatic recall on assess:before. |
| Project hook entry `id: "kbd-memory-log"` `enabled: false` | Disable event mirroring entirely. |
