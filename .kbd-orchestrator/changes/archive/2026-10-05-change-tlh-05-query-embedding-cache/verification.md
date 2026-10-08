# Verification — change-tlh-05-query-embedding-cache

Repository: `surreal-memory-server`
Depends on: none

## Acceptance criteria

- Latency evidence is committed before the cache code, with idle and loaded p50/p95.
- Through the real storage with a counting embedding service, 4 searches with the same text embed once; 4 concurrent identical searches embed once; a different text embeds again; capacity 0 embeds every time.
- Writing a memory never reads or fills the query cache (count unchanged by `add_memory`).
- `Cargo.lock` contains exactly one `quick_cache` entry.
- `GET /api/v2/operations/stats` includes `query_embed_cache` with hits and misses (asserted by the root-crate test `tests/query_embed_cache_stats.rs` through the real router).
- Existing `search_correctness` integration test still passes.

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi
awk -F= '/^IDLE_P95_SECONDS=/{found=1; if ($2+0 > 2.0) bad=1} END{exit !(found && !bad)}' docs/perf/query-embedding-latency.md || { echo 'IDLE_P95_SECONDS missing or > 2.0 (stop rule)' >&2; exit 1; }
cargo test -p surreal-memory --test query_embedding_cache
cargo test --test query_embed_cache_stats
cargo test -p surreal-memory --test search_correctness
test "$(grep -c '^name = "quick_cache"' Cargo.lock)" -eq 1
grep -Eqi 'idle.*p50' docs/perf/query-embedding-latency.md && grep -Eqi 'idle.*p95' docs/perf/query-embedding-latency.md && grep -Eqi 'loaded.*p95' docs/perf/query-embedding-latency.md || { echo 'latency evidence lacks idle p50/p95 and loaded p95' >&2; exit 1; }
perf=$(git log --diff-filter=A --format=%H -- docs/perf/query-embedding-latency.md | tail -1); cache=$(git log -S get_value_or_guard_async --format=%H -- crates/surreal-memory/src/storage/surreal.rs | tail -1); test -n "$perf" && test -n "$cache" && git merge-base --is-ancestor "$perf" "$cache" && test "$perf" != "$cache" || { echo 'latency evidence must be committed before the cache code' >&2; exit 1; }
grep -Eq '^version = "1\.10\.1"' Cargo.toml && grep -q '"version": "1.10.1"' openapi/surreal-memory-v2.openapi.json && grep -A1 '^name = "surreal-memory-server"' Cargo.lock | grep -q '1.10.1' || { echo '1.10.1 bump incomplete (Cargo.toml, openapi, Cargo.lock)' >&2; exit 1; }
```
