# Verification — change-tli-a4-surreal-lean-search-categories-rekey

Repository: `surreal-memory-server`
Depends on: none

## Acceptance criteria

- REST and MCP search results contain no `embedding` key by default and are non-empty for a seeded match; `include_embeddings: true` restores it.
- With categories `["aud:role:x"]`, only memories carrying that category are returned on both the REST hybrid path and the MCP search path, and a filtered KNN still returns `limit` rows when matches sit outside the unfiltered top-k.
- Records seeded with absent and with empty-string `agent_id` (the forms the write paths produce; the predicate also matches NULL) are counted by a `dry_run` re-key, then re-keyed, attributed records are untouched, and replaying the same operation id returns 200 without re-running.
- Gate (amended at execute): hermetic in-process tests through the real `build_router` over the embedded SurrealDB engine (`SurrealStorage::new_mem`) with a deterministic embedder, instead of a separately started server; this exercises the same REST/MCP/operations code and needs no external database.
- A re-key from a non-loopback peer is refused with 403, proven deterministically by an in-process router test that injects `ConnectInfo<SocketAddr>` with a non-loopback address (no reliance on host interfaces).

## Verify commands

Exit 0 = pass, 1 = fail, 2 = BLOCKED (a prerequisite is absent; never reported as a pass).

```verify
if pgrep -x cargo >/dev/null || pgrep -x rustc >/dev/null; then echo "BLOCKED: another cargo/rustc build is running (one build at a time)" >&2; exit 2; fi
cargo test --test team_scoping 2>&1 | tee "${TMPDIR:-/tmp}/tli-a4-test.$$.log"; grep -Eq 'test result: ok\. ([4-9]|[1-9][0-9]+) passed' "${TMPDIR:-/tmp}/tli-a4-test.$$.log" || { echo 'fewer than 4 team_scoping tests passed' >&2; exit 1; }
cargo test -p surreal-memory --test search_correctness
```
