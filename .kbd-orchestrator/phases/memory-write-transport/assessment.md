# Assessment — memory-write-transport

Concern (from deployment, 2026-06-12): the bash `memory-bridge.sh` cannot write
to the surreal-memory server. The server speaks the **two-connection SSE MCP
transport** (`GET /mcp/sse` opens a session that emits a `sessionId`, then tool
calls go to `POST /mcp/messages?sessionId=<id>`). A fire-and-forget bash `POST`
to `/mcp/sse` returns **405**; everything falls to the outbox. This is not a
regression — the outbox + graceful degradation exist precisely for this — but
the *write actually reaching the server* currently depends entirely on the
agent's `mcp__surreal-memory__*` tools, and **nothing drains the outbox to the
server**. So bash-originated memory writes (the Phase 4 automatic write-back)
are buffered indefinitely.

## Verified ground truth

| Fact | Evidence |
|------|----------|
| `_mem_call` POSTs JSON-RPC to `MEM_URL` (`…/mcp/sse`) | `shared/scripts/lib/memory-bridge.sh:85-99` |
| `/mcp/sse` is GET,HEAD only → POST 405 | `curl -X OPTIONS` → `allow: GET,HEAD`; POST → 405 |
| Real endpoint is `POST /mcp/messages?sessionId=<id>` | bare POST → 400 "missing field sessionId" |
| Health is `/health` (200), not `/healthz` (404) | fixed this deploy (commit fa7e18e) |
| No streamable-HTTP single-POST transport | `POST /mcp` → 404; `POST /` → 404 |
| `mem0-compress.sh` has the SAME broken POST-to-/mcp/sse | `shared/scripts/mem0-compress.sh:32,36` — never actually worked |
| **`prometheus memory` CLI exists** (built, on PATH) but is READ/ADMIN only | `ping`, `stats`, `search`, `install` — NO write/add subcommand |
| `prometheus memory ping` reaches the server correctly | "✅ Server healthy at http://localhost:23001" |
| `prometheus memory stats` has a deserialization bug | "missing field `name`" — schema drift, separate issue |
| surrealdb is directly reachable on :28000 (`/health` 200) | could bypass MCP, but loses embeddings/scoping/dedup the MCP layer provides |
| Agent MCP write path verified end-to-end | `mcp__surreal-memory__add_memory` → record `secmtdn2yenvid3n23mk` → `search_memories` round-trip |
| Outbox is written but NEVER drained to the server | `memory-outbox-flush.sh` calls `_mem_call` (same broken POST) → re-buffers; nothing else reads it |

## Write surface (consumers that currently silently outbox)

- `shared/scripts/memory-writeback.sh` (PostToolUse on reflection.md + orchestrator `reflect:end`) → `mem_add_memory`
- orchestrator builtin `execute:before` → `mem_create_task_stream`
- `shared/scripts/memory-outbox-flush.sh` (SessionStart) → re-POSTs via `_mem_call` (also 405s)
- `shared/scripts/mem0-compress.sh` (scheduled) → independent broken POST

## Gaps

| ID | Gap | Severity |
|----|-----|----------|
| T1 | `_mem_call` (and mem0-compress) POST to the SSE *stream* endpoint, which is GET-only → every bash write 405s and outboxes. | HIGH — the Phase 4 memory feature does not persist anything from bash. |
| T2 | `memory-outbox-flush.sh` uses the same broken `_mem_call`, so the outbox is never actually drained — it only grows. | HIGH — buffered writes are lost in practice. |
| T3 | No shell-callable WRITE path to surreal-memory exists: `prometheus memory` is read-only; no `add_memory` CLI. | MEDIUM — forces either an SSE-session client in bash, a CLI addition, or accepting agent-only writes. |
| T4 | `prometheus memory stats` deserialization error (schema drift) — the read CLI is partially broken too. | LOW — separate, but signals CLI/server schema drift to watch. |

## Candidate solutions (for the Analyze/Plan stages to weigh)

1. **Add a write subcommand to the `prometheus` CLI** (`prometheus memory add …`)
   that speaks the SSE session handshake in Rust (where it's clean), and have
   the bash bridge + outbox-flush shell out to it. Pro: one correct transport
   client, reused everywhere; the CLI is the documented shell path. Con: edits
   an external tool (`tools/prometheus-cli`, a submodule) — may be out of this
   repo's control.
2. **Implement the SSE session handshake in bash** (`_mem_call` opens `GET
   /mcp/sse` in the background to capture the `sessionId`, then POSTs to
   `/mcp/messages?sessionId=`). Pro: self-contained in this repo. Con: holding a
   stream open + parsing SSE in bash is fragile (the exact complexity the outbox
   was meant to avoid); the sycophancy.sh FIFO pattern is a stdio analog, not HTTP.
3. **Agent-drained outbox (lean into the verified architecture).** Keep bash
   buffering; make the AGENT the drainer — a documented step (or a tiny skill)
   where Claude Code reads `memory-outbox.jsonl` and replays each line via
   `mcp__surreal-memory__add_memory`, then truncates. Pro: uses the only write
   path that actually works today; zero fragile transport code. Con: writes only
   persist when an agent session runs the drain (acceptable — memories are
   agent-authored anyway).
4. **liter-llm MCP proxy as a bridge** — liter-llm exposes an MCP server; check
   whether it can proxy/forward tool-calls to surreal-memory over the SSE
   transport, giving bash a simpler target. Pro: reuses a built tool. Con:
   unverified it proxies arbitrary downstream MCP servers; likely scope creep.
5. **Direct SurrealDB write on :28000** — bypass MCP, INSERT into the memory
   table directly. Pro: simplest POST. Con: loses the MCP layer's embedding
   generation, scoping, dedup, importance — would store unembedded rows that
   semantic_search can't find. Reject unless the MCP add path is truly
   unavailable.

## Verdict

GO to plan a fix. The honest near-term answer is **#3 (agent-drained outbox)** —
it's the only path verified working today and requires no fragile transport
code — paired with **fixing T2** so `memory-outbox-flush.sh` stops pretending to
drain via the broken POST (it should either no-op-with-notice or invoke the
agent-drain path). **#1 (Rust CLI write subcommand)** is the right *durable*
answer if `tools/prometheus-cli` is in scope to edit; it makes bash writes work
unattended. **#2 should be avoided** (re-introduces the fragility the outbox
exists to dodge). Open question for Analyze: is `tools/prometheus-cli` editable
here, or is it an external submodule we only consume?
