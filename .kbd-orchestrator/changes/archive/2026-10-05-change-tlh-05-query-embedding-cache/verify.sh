#!/usr/bin/env bash
# Generated from verification.md for change-tlh-05-query-embedding-cache. Run by `kbd-apply verify` via tasks.json.
set -euo pipefail
TLI_KBD="/Users/gqadonis/Projects/prometheus/prometheus-skill-pack/.kbd-orchestrator"
ROOT="${TLH_ROOT:-/Users/gqadonis/Projects/prometheus/worktrees/tlh-05}"
cd "$ROOT" || { echo "BLOCKED: worktree $ROOT missing" >&2; exit 2; }
git merge-base --is-ancestor 0af8ae1 HEAD || { echo "base does not contain 0af8ae1" >&2; exit 1; }
( if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi ) || exit $?
( awk -F= '/^IDLE_P95_SECONDS=/{found=1; if ($2+0 > 2.0) bad=1} END{exit !(found && !bad)}' docs/perf/query-embedding-latency.md || { echo 'IDLE_P95_SECONDS missing or > 2.0 (stop rule)' >&2; exit 1; } ) || exit $?
( cargo test -p surreal-memory --test query_embedding_cache ) || exit $?
( cargo test --test query_embed_cache_stats ) || exit $?
( cargo test -p surreal-memory --test search_correctness ) || exit $?
( test "$(grep -c '^name = "quick_cache"' Cargo.lock)" -eq 1 ) || exit $?
( grep -Eqi 'idle.*p50' docs/perf/query-embedding-latency.md && grep -Eqi 'idle.*p95' docs/perf/query-embedding-latency.md && grep -Eqi 'loaded.*p95' docs/perf/query-embedding-latency.md || { echo 'latency evidence lacks idle p50/p95 and loaded p95' >&2; exit 1; } ) || exit $?
( perf=$(git log --diff-filter=A --format=%H -- docs/perf/query-embedding-latency.md | tail -1); cache=$(git log -S get_value_or_guard_async --format=%H -- crates/surreal-memory/src/storage/surreal.rs | tail -1); test -n "$perf" && test -n "$cache" && git merge-base --is-ancestor "$perf" "$cache" && test "$perf" != "$cache" || { echo 'latency evidence must be committed before the cache code' >&2; exit 1; } ) || exit $?
( grep -Eq '^version = "1\.10\.1"' Cargo.toml && grep -q '"version": "1.10.1"' openapi/surreal-memory-v2.openapi.json && grep -A1 '^name = "surreal-memory-server"' Cargo.lock | grep -q '1.10.1' || { echo '1.10.1 bump incomplete (Cargo.toml, openapi, Cargo.lock)' >&2; exit 1; } ) || exit $?
echo "verify OK: change-tlh-05-query-embedding-cache"
