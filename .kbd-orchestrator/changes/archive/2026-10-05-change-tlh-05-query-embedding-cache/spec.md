# change-tlh-05-query-embedding-cache

**Title:** surreal-memory caches query embeddings (quick_cache, query path only) after measuring single-embedding latency under load; release 1.10.1
**Plan PR:** 05
**Repository:** `surreal-memory-server`
**Phase:** phase-team-learning-hardening
**Depends on:** none
**Backend:** native-kbd
**Base branch:** a worktree branch off `origin/main` of surreal-memory-server at or after `0af8ae1`, at `/Users/gqadonis/Projects/prometheus/worktrees/tlh-05` (created with `git -C /Users/gqadonis/Projects/prometheus/surreal-memory-server worktree add`). Delivered through a pull request; the user merges.
**Analysis section:** `.kbd-orchestrator/phases/phase-team-learning-hardening/analysis.md` §G2b

## Why

Every `POST /api/v1/search` embeds its query again (`search_memories` -> `embed_text` -> `embedding_service.embed`, no cache); one SubagentStart recall sends 4 searches (5 for the lead) with identical text in series and misses its 2.8 s deadline under load (assessment G2b, analysis §G2b).

## What Changes

- Measure first: `tests/query_embedding_latency.rs` (integration, real executor, `#[ignore]` by default, run explicitly) records p50/p95 latency of one query embedding idle and with 8 concurrent writers; the result is committed to `docs/perf/query-embedding-latency.md`, which includes machine-readable lines `IDLE_P50_SECONDS=`, `IDLE_P95_SECONDS=` and `LOADED_P95_SECONDS=`. If idle p95 > 2.0 s the change stops and reports instead of implementing (the cache cannot rescue it).
- Add `quick_cache = "0.6"` (the version already in Cargo.lock through surrealdb-core; no second copy) and a bounded query-embedding cache in `SurrealStorage`: key = (embedding model id, NFC-normalised whitespace-collapsed query text), capacity from `SURREAL_MEMORY_QUERY_EMBED_CACHE` (default 512, 0 disables). Use `get_value_or_guard_async` so concurrent identical misses embed once. Only `search_memories` (and therefore hybrid search) uses it; the write path never reads or fills it.
- Expose cache hits/misses as a `query_embed_cache` object in `GET /api/v2/operations/stats` (`src/operations.rs`), proven by a root-crate integration test `tests/query_embed_cache_stats.rs` that drives the real router.
- Bump the release version to 1.10.1 in the same three files release PR #45 changed for 1.10.0: `Cargo.toml`, `Cargo.lock` and `openapi/surreal-memory-v2.openapi.json` (the repository has no CHANGELOG). The tag push is operator-approved and is not part of the gate.

## Scope

- `Cargo.toml`
- `Cargo.lock`
- `crates/surreal-memory/Cargo.toml`
- `crates/surreal-memory/src/storage/surreal.rs`
- `crates/surreal-memory/tests/query_embedding_cache.rs`
- `crates/surreal-memory/tests/query_embedding_latency.rs`
- `docs/perf/query-embedding-latency.md`
- `openapi/surreal-memory-v2.openapi.json`
- `src/operations.rs`
- `tests/query_embed_cache_stats.rs`

## Constraints

- Implement the whole change, then run the gate once (implementation-first, integration-only).
- One cargo/rustc build on the machine at a time.
- Hooks exit 0 and print nothing when a dependency is absent.
- A change with a Depends-on starts from `origin/main` after that dependency has merged; its verify.sh exits 2 (BLOCKED) when the dependency is absent from the base.
- Tests use scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; files mutated by a gate are restored with mktemp + trap.
- Outward-facing steps (tag pushes, issue creation) are confirmed with the user first.

## Plan reference

Ordering and the scoped Task model assignments for this change's tasks: `.kbd-orchestrator/phases/phase-team-learning-hardening/plan.md` (rows keyed by `change-tlh-05-query-embedding-cache` and backend task ID).

## Recalled lessons (from prior-context.md)

- "For every external integration, run at least one test against the real service before claiming the work is done." Applied as an acceptance criterion.
