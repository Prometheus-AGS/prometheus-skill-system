# change-tli-a4-surreal-lean-search-categories-rekey

**Title:** surreal-memory: lean search responses, categories honoured on both hybrid legs, loopback-only re-key operation
**Plan PR:** A4
**Repository:** `surreal-memory-server`
**Phase:** team-aware-learning-memory-impl
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of surreal-memory-server at or after `777cf72`, at `/Users/gqadonis/Projects/prometheus/worktrees/tli-a4` (created with `git -C /Users/gqadonis/Projects/prometheus/surreal-memory-server worktree add`). Delivered through a pull request; the user merges.
**Design section:** `docs/design/team-aware-learning-memory.md` §8 U4–U6, §2

## Why

REST search returns full `Memory` records with embeddings; the MCP categories filter is advertised but dropped in storage (`_categories`, surreal.rs:2090-2096) and absent from `hybrid_search_memories` (2521/2533); there is no way to repair null-agent_id records (assessment A4, analysis D-6).

## What Changes

- Lean DTO at the response boundary: REST `src/api/search.rs` and the MCP search handlers map `Memory` to a DTO without `embedding`; request field `include_embeddings: true` opts back in. Storage still loads embeddings (re-ranking at surreal.rs:2139-2146 needs them).
- Categories (any-match): `search_memories` uses `$categories`; `hybrid_search_memories` gains a `categories` parameter threaded into both the vector and BM25 legs; update the storage trait, every implementation and every caller (REST, MCP, A2A); REST `SearchBody` gains `categories`. If SurrealDB 3.3.0 applies the HNSW limit before the WHERE clause, over-fetch and post-filter so `limit` rows are still returned.
- Re-key as a new kind `rekey_agent_id` in the existing operations framework `src/operations.rs` (mounted at `/api/v2/operations`, src/api/mod.rs:120), reusing its idempotency. Payload `{to_agent_id, user_id?, dry_run}`; predicate `agent_id IS NONE OR agent_id = NULL`; tables `memory` and `task_stream` (task steps have no agent_id).
- Loopback-only: wire `into_make_service_with_connect_info::<SocketAddr>()` in `src/main.rs`; `submit_operation`/`validate_payload` refuse `rekey_agent_id` from a non-loopback peer with 403. Document that it is unavailable from container bridge networks (operator runs it on the host).

## Scope

- `src/api/search.rs`
- `src/mcp/handlers.rs`
- `src/operations.rs`
- `src/main.rs`
- `openapi/surreal-memory-v2.openapi.json`
- `crates/surreal-memory/src/storage/surreal.rs`
- `crates/surreal-memory/src/storage/mod.rs`
- `crates/surreal-memory/src/lib.rs`
- `crates/surreal-memory/tests/search_correctness.rs`
- `crates/surreal-memory/tests/load_repro.rs`
- `tests/team_scoping.rs`
- `docs/operations.md`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Generated hooks/dist are regenerated, never hand-edited (C-01, C-04); bash 3.2 for shell (C-05).
- Generated outputs (`dist/plugins/**`, `hooks/*.json`, `shared/harnesses/generated/*`, `shared/scripts/generated/*`) may be touched by changes with no mutual order: they are never merged by hand. After rebasing onto the latest `main`, rerun the generators; idempotence (C-04) makes the result the same whichever change lands first.
- Hooks exit 0 and print nothing when a dependency is absent.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.
