# Decision Log — memory-write-transport

## D-001 · Use the REST API (POST /api/v1/memory), not the SSE transport   [analyze · 2026-06-12]
**TL;DR:** adopt cand-001 — bash writes directly to the plain REST route; no SSE session needed.
**Why:** the server exposes REST `/api/v1/...` alongside SSE MCP; the in-repo prometheus-cli client already uses REST; live `POST /api/v1/memory` → 201. The assessment's "bash can't write" was an artifact of POSTing JSON-RPC to the GET-only SSE stream.
**Alternatives:** bash SSE handshake (cand-003 — rejected, fragile) · new Rust CLI subcommand (cand-002 — reference, heavier) · direct SurrealDB (cand-005 — rejected, loses embeddings).
**Learn more:** analysis.md "decisive finding"; tools/surreal-memory-server/src/contracts.rs:325.

## D-002 · Task-stream writes stay MCP-tool-only (no REST route)            [analyze · 2026-06-12]
**TL;DR:** only add_memory gets the REST fix; create_task_stream/add_task_step/complete_step have no REST route.
**Why:** server declares 10 MCP tools but only 10 partly-overlapping REST routes; task-streams are MCP-only. The durable learning is add_memory; task-streams are telemetry.
**Alternatives:** force task-streams through agent-drain (cand-004 — kept as fallback) · drop them.
**Learn more:** analysis.md coverage map.

## D-003 · prometheus-cli is editable here (resolves the assessment's open Q)  [analyze · 2026-06-12]
**TL;DR:** tools/prometheus-cli is in-repo source, not a submodule — a Rust write subcommand IS possible, but not required given the REST fix.
**Why:** .gitmodules lists only surreal-memory-server/prometheus-knowledge/liter-llm; prometheus-cli has its own crates/ here.
**Learn more:** .gitmodules; tools/prometheus-cli/crates/.
