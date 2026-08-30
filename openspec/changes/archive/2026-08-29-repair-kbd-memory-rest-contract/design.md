## Context

See `proposal.md` for motivation. The installed server exposes `/health`, `/api/v1/entities`, `/api/v1/entities/relations`, and entity search; KBD currently assumes older routes and an object observation shape that the current contract rejects. Hook execution must remain non-blocking and macOS Bash 3.2 compatible.

## Goals / Non-Goals

**Goals:**

- Align endpoint discovery, lifecycle writes, and recall reads with the current server.
- Preserve structured event metadata inside the server's string-observation contract.
- Prove reachable-empty and unreachable states are distinguishable.

**Non-Goals:**

- Add compatibility routes to surreal-memory-server.
- Make memory availability a KBD stage blocker.
- Introduce a new client dependency or background retry worker.

## Decisions

1. Derive a REST base as `scheme://host:port` from any configured MCP URL and default to `http://127.0.0.1:23001`. This matches the installed local service while explicit configuration retains authority.
2. Serialize one compact observation object to a JSON string. This preserves the documented fields without changing the shared server entity type.
3. Use entity search for retrieval and perform deterministic ranking in `jq`: same project first, then phase-token overlap, then timestamp. Adding a server-only KBD relevance endpoint would duplicate policy in the wrong layer.
4. Keep bounded probes and writes soft-failing, but emit a diagnostic for route or payload failures so contract regressions are no longer silently indistinguishable from an offline service.

## Risks / Trade-offs

- [String observations require parsing by KBD readers] → Accept legacy plain strings as unusable records and skip them rather than failing recall.
- [Entity search can return a large result set] → Query by the lifecycle entity type and cap local output to five; server-side pagination can be added later without changing the spec.
- [A local default may probe a service the user does not run] → Keep the probe bounded to two seconds and preserve the successful stub fallback.

## Migration Plan

Update source, tests, and references; run fake-server tests and a live write/recall probe; then regenerate and refresh distributions in the reconciliation change. Rollback restores the prior scripts, which returns to non-blocking stubs but loses working recall.
