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
| `entityType` | string | Always `"kbd_lifecycle_event"`. |
| `entityId` | string | `<project>/<phase>/<kind>/<edge>/<index>/<ts>` — deterministic, idempotent on retry. |
| `observations[0].kind` | string | `phase` / `child` / `plan` / `execute` / `reflect` / `task` / `assess`. |
| `observations[0].edge` | string | `before` / `after`. |
| `observations[0].name` | string | Active item's canonical name (phase name, change id, task title). |
| `observations[0].index` | integer | 1-based index within the containing loop. |
| `observations[0].total` | integer | Loop total. |
| `observations[0].phasePath` | string | Rendered chain (`parent › child`). |
| `observations[0].sourceTool` | string | Tool that emitted the event (claude-code, codex, …). |
| `observations[0].project` | string | Project name from `project.json`. |
| `observations[0].ts` | string | ISO-8601 UTC timestamp. |

Two relations always present:

- `<entityId> --fires-in--> phase:<phase>`
- `<entityId> --belongs-to--> project:<project>`

Unknown observation fields MUST be tolerated by readers. New fields can be
added in later changes; removed fields require a new `entityType`.

## Retention

- **Default window**: 365 days from `ts`.
- **Retention is the server's job**, not the writer's. The hook writes every
  event; the surreal-memory-server enforces the window via its own retention
  policy.
- **Sensitive content**: the memory writer captures ONLY the structured
  `KBD_HOOK_*` payload — never the stderr stream of third-party hooks, which
  may contain secrets.

## Relevance ordering for recall

`/kbd-memory-recall` queries `find_relevant`. When the server doesn't impose
an explicit ordering, consumers should rank results by:

1. **Same `project`** (highest — same codebase, same constraints).
2. **Same `kind`** (events of the same type, e.g. `plan:before` matches `plan:before`).
3. **Phase-name pattern match** (longest common substring or token overlap).
4. **Recency** (newer first).

Cross-project recall (relaxing #1) is opt-in via a `--cross-project` flag on
`/kbd-memory-recall`; default off so phase planning stays focused.

## Probe + endpoint discovery

`kbd_memory_available()` (in `shared/lib/memory.sh`) probes in this order
and returns the first success:

1. Calling agent's tool list contains `create_entity` → MCP-mode.
2. `$UAR_MEMORY_MCP_URL` → HTTP HEAD/healthz.
3. `$KBD_MEMORY_MCP_URL` → HTTP HEAD/healthz.
4. `.kbd-orchestrator/memory.config.json` → `mcpEndpoint` field.

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
| `UAR_MEMORY_MCP_URL` / `KBD_MEMORY_MCP_URL` | Endpoint URL (HTTP). |
| `.kbd-orchestrator/memory.config.json` `mcpEndpoint` | Alternative to the env var. |
| Project hook entry `id: "auto-memory-recall"` `enabled: false` | Disable automatic recall on assess:before. |
| Project hook entry `id: "kbd-memory-log"` `enabled: false` | Disable event mirroring entirely. |
| `--cross-project` (CLI flag on `/kbd-memory-recall`, future) | Expand recall beyond the current project. |
