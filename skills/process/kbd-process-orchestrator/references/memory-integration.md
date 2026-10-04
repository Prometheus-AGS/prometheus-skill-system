# Surreal-Memory Integration

> Extracted from the orchestrator SKILL.md. How KBD mirrors lifecycle events into surreal-memory and exposes recall.

**Default-on when reachable.** `shared/lib/memory.sh::kbd_memory_available`
resolves the configured or canonical local surreal-memory service origin and
probes `GET /health`. KBD mirrors every hook fire through
`POST /api/v1/entities` as a `kbd_lifecycle_event` entity. Lessons are a
separate path: `reflect:after` and the `assess|analyze|plan:after` stage
write-backs store attributed lessons through `shared/scripts/lib/learning_write.py`,
and `/kbd-memory-recall` reads them back through
`shared/scripts/lib/learning_recall.py` (design
`docs/design/team-aware-learning-memory.md` §4–§6). When the service is unreachable, memory operations fail open —
no KBD lifecycle operation is blocked.

### What the integration provides

- **Cross-tool coordination**: hook events are stored as string-encoded entity observations, readable through the REST or MCP entity APIs.
- **Cross-project learning**: every decoded lifecycle observation carries its project identifier; recall prioritizes same-project events while retaining lower-ranked cross-project candidates.
- **Phase context**: `/kbd-memory-recall` writes `prior-context.md` (Lessons, pk knowledge, Previous reflection, Knowledge gaps) at the start of every stage via the `auto-memory-recall*` hooks, under a byte budget.
- **Audit trail**: events flow into the memory store *in addition to* the per-phase JSONL log; the JSONL is the in-flight source of truth, the memory mirror is the queryable index.

### Detection contract

`kbd_memory_available` resolves in this order, normalizes HTTP(S) values to
their origin, probes `GET <origin>/health` with bounded timeouts, and caches the
result for the process lifetime:

1. `$UAR_MEMORY_MCP_URL`, then `$KBD_MEMORY_MCP_URL`.
2. `.kbd-orchestrator/memory.config.json` field `restEndpoint`, then legacy `mcpEndpoint`.
3. Canonical local default `http://127.0.0.1:23001`.
4. MCP-only `create_entity` availability as a final agent-owned fallback, with no fabricated REST URL for shell callers.

### Entity contract (lifecycle mirror)

- The writer sends `{name, entity_type, observations}` where
  `entity_type = "kbd_lifecycle_event"` and `observations` contains one compact
  JSON string. It does not send object observations or synthesize relations.

### Lesson write-back and recall contract

- **Write-back.** `reflect:after` runs `shared/scripts/memory-writeback.sh`:
  Delta/Root Cause/Corrective Actions become one `project` lesson, each Lessons
  Learned bullet one lesson (`[GLOBAL]` → global, `[USER]` → user, else
  project), the Next Phase Seed one `project` progress record; Codify as Skill?
  is never written. `assess|analyze|plan:after` run
  `shared/scripts/kbd-stage-writeback.sh`, which stores the stage handoff
  summary at visibility `lead`. Both resolve the pack's shared scripts through
  `KBD_PACK_ROOT` / the plugin root, then the flat installed layout
  (`<root>/skills/kbd-process-orchestrator`), then the source tree.
- **Recall.** A main-thread stage gets the lead view: equality-filtered
  `POST /api/v1/search` queries on `<team>/@lead`, `<team>/@team` (last 50),
  `@project`, then the top 3 of `@user:<hash>` and `@global`; scored semantic
  0.5 + recency 0.3 (30-day half-life) + importance 0.2, de-duplicated on the
  content hash, stopped at the budget. Records outside the view are dropped even
  if a server returns them. Store unreachable or invalid → `pk context` →
  the learning log. Each run appends `{agentType, bytesByChannel, entriesByScope}`
  to `~/.prometheus/learning-index/delivery.jsonl`.

### Built-in hooks

| id | event | mode | purpose |
|---|---|---|---|
| `kbd-memory-log` | `*:*` | augment | Mirror each hook fire into surreal-memory; no-op when unreachable. |
| `auto-memory-recall` | `assess:before` | augment | Populate `prior-context.md` before each `/kbd-assess`. |
| `auto-memory-recall-<stage>` | `analyze\|plan\|execute\|reflect:before` | augment | Refresh `prior-context.md` for the stage. |
| `kbd-stage-writeback-<stage>` | `assess\|analyze\|plan:after` | augment | Store the stage handoff summary at visibility `lead`. |
| `memory-reflection-writeback` | `reflect:after` | augment | Write the accepted reflection back as attributed lessons. |

All ship enabled and can be disabled per-project via `.kbd-orchestrator/hooks-config.json` (set `enabled: false` on a matching `id`).

### Reference

- Event entity schema, retention window, and relevance ordering: [`shared/references/memory-retention.md`](shared/references/memory-retention.md).
- Recall skill: `skills/kbd-memory-recall/SKILL.md`.
