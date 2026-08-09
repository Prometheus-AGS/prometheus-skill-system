# Surreal-Memory Integration

> Extracted from the orchestrator SKILL.md. How KBD mirrors lifecycle events into surreal-memory and exposes recall.

**Default-on when reachable.** When the surreal-memory MCP endpoint is detected by `shared/lib/memory.sh::kbd_memory_available`, KBD mirrors every hook fire into the memory store as a `kbd_lifecycle_event` entity and exposes `/kbd-memory-recall` for retrieving prior similar work as planning input. When the endpoint is unreachable, the memory subsystem cleanly no-ops — no KBD operation is ever blocked by a missing memory service.

### What the integration provides

- **Cross-tool coordination**: hook events stored as entity observations, readable by any AI tool that can query the surreal-memory MCP.
- **Cross-project learning**: every event entity carries a `belongs-to project:<name>` relation; `/kbd-memory-recall` (with `--cross-project` in future) surfaces patterns from prior projects on similar work.
- **Phase context**: `/kbd-memory-recall` writes `prior-context.md` for each phase before `/kbd-assess`, automatically via the `auto-memory-recall` hook.
- **Audit trail**: events flow into the memory store *in addition to* the per-phase JSONL log; the JSONL is the in-flight source of truth, the memory mirror is the queryable index.

### Detection contract

`kbd_memory_available` probes in this order, caches the result for the process lifetime, and returns 0 on success:

1. Calling agent's tool list contains `create_entity` (MCP-mode).
2. `$UAR_MEMORY_MCP_URL` or `$KBD_MEMORY_MCP_URL` set and `GET <url>/healthz` returns 2xx within 2 s.
3. `.kbd-orchestrator/memory.config.json` field `mcpEndpoint` reachable.

### Built-in hooks

| id | event | mode | purpose |
|---|---|---|---|
| `kbd-memory-log` | `*:*` | augment | Mirror each hook fire into surreal-memory; no-op when unreachable. |
| `auto-memory-recall` | `assess:before` | augment | Populate `prior-context.md` before each `/kbd-assess`. |

Both ship enabled and can be disabled per-project via `.kbd-orchestrator/hooks-config.json` (set `enabled: false` on a matching `id`).

### Reference

- Event entity schema, retention window, and relevance ordering: [`shared/references/memory-retention.md`](shared/references/memory-retention.md).
- Recall skill: `skills/kbd-memory-recall/SKILL.md`.
