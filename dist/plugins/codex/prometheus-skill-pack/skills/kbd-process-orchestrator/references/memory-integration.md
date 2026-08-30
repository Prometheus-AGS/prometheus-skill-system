# Surreal-Memory Integration

> Extracted from the orchestrator SKILL.md. How KBD mirrors lifecycle events into surreal-memory and exposes recall.

**Default-on when reachable.** `shared/lib/memory.sh::kbd_memory_available`
resolves the configured or canonical local surreal-memory service origin and
probes `GET /health`. KBD mirrors every hook fire through
`POST /api/v1/entities` as a `kbd_lifecycle_event` entity and exposes
`/kbd-memory-recall`, backed by `GET /api/v1/entities/search`, for prior-work
planning input. When the service is unreachable, memory operations fail open —
no KBD lifecycle operation is blocked.

### What the integration provides

- **Cross-tool coordination**: hook events are stored as string-encoded entity observations, readable through the REST or MCP entity APIs.
- **Cross-project learning**: every decoded lifecycle observation carries its project identifier; recall prioritizes same-project events while retaining lower-ranked cross-project candidates.
- **Phase context**: `/kbd-memory-recall` writes `prior-context.md` for each phase before `/kbd-assess`, automatically via the `auto-memory-recall` hook.
- **Audit trail**: events flow into the memory store *in addition to* the per-phase JSONL log; the JSONL is the in-flight source of truth, the memory mirror is the queryable index.

### Detection contract

`kbd_memory_available` resolves in this order, normalizes HTTP(S) values to
their origin, probes `GET <origin>/health` with bounded timeouts, and caches the
result for the process lifetime:

1. `$UAR_MEMORY_MCP_URL`, then `$KBD_MEMORY_MCP_URL`.
2. `.kbd-orchestrator/memory.config.json` field `restEndpoint`, then legacy `mcpEndpoint`.
3. Canonical local default `http://127.0.0.1:23001`.
4. MCP-only `create_entity` availability as a final agent-owned fallback, with no fabricated REST URL for shell callers.

### Entity and recall contract

- The writer sends `{name, entity_type, observations}` where
  `entity_type = "kbd_lifecycle_event"` and `observations` contains one compact
  JSON string. It does not send object observations or synthesize relations.
- Recall searches the lifecycle entity class once, decodes each observation,
  and orders candidates by same project, token overlap, recency, entity name,
  and observation index before rendering the top five.
- Reachable empty search produces a normal empty digest. Unreachable transport
  and invalid response contracts produce distinct atomic stub digests.

### Built-in hooks

| id | event | mode | purpose |
|---|---|---|---|
| `kbd-memory-log` | `*:*` | augment | Mirror each hook fire into surreal-memory; no-op when unreachable. |
| `auto-memory-recall` | `assess:before` | augment | Populate `prior-context.md` before each `/kbd-assess`. |

Both ship enabled and can be disabled per-project via `.kbd-orchestrator/hooks-config.json` (set `enabled: false` on a matching `id`).

### Reference

- Event entity schema, retention window, and relevance ordering: [`shared/references/memory-retention.md`](shared/references/memory-retention.md).
- Recall skill: `skills/kbd-memory-recall/SKILL.md`.
