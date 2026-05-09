---
id: SP-003
title: pk-focus result caching
status: planned
priority: P2
estimated_effort: 0.5d
agent_role: hooks-engineer
depends_on: [SP-002]
unblocks: []
related: []
created_from_conversation_turn: 3-4
---

# SP-003 — pk-focus result caching

## Problem

Even after SP-002 lands a basic in-script cache (hash → JSON file), the cache is not durable across system reboots, has no eviction policy, and is not shared across users on the same machine. For workflows where multiple Claude Code sessions run in parallel against the same project, each session re-hits the LLM for queries the others already ran.

## Evidence

Inspect `~/.cache/prometheus/pk-focus/` after a session. Note the file count and total size after a typical day's use. Compare cache hit rate (count of cache files vs. distinct prompts).

## Why it matters

The savings from SP-002's cache are real but bounded by within-session locality. Cross-session locality (especially in parallel-agent workflows enabled by `00-meta/parallel-agent-routing.md`) is left on the table.

## Proposed fix

Promote the cache to a small SQLite database at `~/.cache/prometheus/pk-focus.sqlite` with:

- Schema: `(query_hash TEXT PRIMARY KEY, project_root TEXT, query_text TEXT, result_json TEXT, ttl INTEGER, hits INTEGER DEFAULT 0)`.
- Reads bump `hits`; high-hit entries get longer TTL.
- A daily cleanup that drops entries with `ttl < now`.

The shell script gains two helper functions: `pk_focus_cache_get` and `pk_focus_cache_put`, both using `sqlite3` (which is universally available on dev machines).

## Trade-offs and risks

- **Adds a SQLite dependency** to the hook-script invocation path. SQLite is universal but adds a tiny cold-start cost on every script call. Measure before/after; if >50ms regression, revisit.
- **Cross-session writes can race.** SQLite handles this via WAL mode; enable `PRAGMA journal_mode=WAL`.

## Acceptance criteria

- [ ] Cache writes go to SQLite.
- [ ] A second, independent session retrieves a cached entry written by the first within the TTL window.
- [ ] A daily cleanup leaves only entries with `ttl >= now`.

## Implementation steps

1. Create `shared/scripts/lib/pk-focus-cache.sh` with the helper functions.
2. Source it from `pk-focus-on-prompt.sh`.
3. Add a `cleanup` mode to the same library invokable as `pk-focus-cache.sh cleanup`, scheduled by SP-009.
4. Update SP-002's bats tests with a multi-session scenario.

## Dependencies

SP-002 (cache layer must exist before promoting it).

## Open questions

- Should the cache be machine-wide or per-user? Default: per-user (`$HOME` location). Multi-user dev machines are rare in this stack.
