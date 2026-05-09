---
id: SP-019
title: LibrarianEvent first-class persistence
status: planned
priority: P0
estimated_effort: 1w
agent_role: rust-codegraph
depends_on: [SP-007, SP-008]
unblocks: [SP-020]
related: []
created_from_conversation_turn: 3-4
---

# SP-019 — LibrarianEvent first-class persistence

This is the largest architectural improvement to the Karpathy loop in the entire pack. It promotes events from in-memory to first-class, queryable, related entities in surreal-memory.

## Problem

The librarian process emits events (`LibrarianEvent`) describing what happens during ingestion: "compiled this entry from this source," "merged these two entries," "flagged this as a duplicate." Currently these events live only in the librarian's process memory. They:

- Don't persist across restarts.
- Aren't queryable from outside the librarian.
- Have no relations to the `WikiEntry` records they affect.
- Are lost when the process exits.

The downstream consequences:
- "Why is this entry in the KB?" can't be answered without re-running ingestion.
- "Which entries did the librarian touch in the last 24 hours?" is unanswerable.
- "What did the librarian decide to merge during this session?" can't be audited.
- The librarian's behavior is opaque to operators.

## Evidence

1. Read `pk-core/src/types.rs` (or equivalent). Find `LibrarianEvent`. Note that it implements no `Persist` trait or equivalent.
2. Read `pk-librarian/src/librarian.rs`. Find where events are emitted. They're typically logged or sent to a channel; not stored.
3. Confirm: there's no `event` table in the Surreal schema, no `events.json` file written.

## Why it matters

This is the foundation for several downstream capabilities:
- **Auditability of KB state**: any entry can be traced to the events that produced it.
- **Selective recompute**: when source content changes, the events tell you which entries need re-evaluation.
- **Cross-session continuity**: a session can ask "what did the librarian do in past sessions about this topic?" and get a meaningful answer.
- **SP-020's foundation**: separating knowledge graph from episodic memory requires events to be first-class entities to relate one to the other.

P0 because it's foundational and the absence keeps blocking other work.

## Proposed fix

Promote `LibrarianEvent` to a first-class persistent entity in the surreal-memory store, with relations to the `WikiEntry` records each event affects.

**Schema additions:**

```sql
DEFINE TABLE event SCHEMAFULL;
DEFINE FIELD kind         ON event TYPE string;     -- 'compile', 'merge', 'flag-duplicate', 'flag-stale', 'split', 'archive'
DEFINE FIELD source_uri   ON event TYPE option<string>;
DEFINE FIELD session_id   ON event TYPE option<string>;
DEFINE FIELD model_used   ON event TYPE option<string>;
DEFINE FIELD project_root ON event TYPE string;
DEFINE FIELD timestamp    ON event TYPE datetime DEFAULT time::now();
DEFINE FIELD payload      ON event TYPE object;     -- event-specific data

-- Relations (use Surreal's RELATION pattern)
DEFINE TABLE affects   SCHEMAFULL TYPE RELATION FROM event TO wiki_entry;
DEFINE TABLE compiled_from SCHEMAFULL TYPE RELATION FROM event TO event;  -- chain of derivation
```

**Code changes:**

1. The librarian's event emit path writes to surreal-memory rather than (or in addition to) the in-memory channel.
2. Each emitted event creates an `event` record and `affects` relations to every `wiki_entry` it touches.
3. Existing librarian internals (the in-memory channel for active session state) remain — surreal-memory is the persistence layer, not the runtime layer.
4. A new `pk events list --since=24h --kind=merge` command queries events for operator inspection.
5. A new `pk events for-entry <entry-id>` shows the history of an entry.

**Integration with SP-008:**

Events are scoped to project (per SP-008). The `project_root` field on the event makes per-project queries trivial. Cross-project event queries require `--scope=shared` opt-in.

## Trade-offs and risks

- **Cost: every event now hits the database.** Most events are sub-millisecond writes; for a 1000-event ingestion run this is ~seconds of extra time. Acceptable.
- **Risk: schema evolution.** The `payload` field is `object` (free-form). When event structure changes, queries on payload fields may break. Mitigation: `kind`-specific schema validation at write time; document each kind's expected payload shape in `pk-core/docs/EVENTS.md`.
- **Risk: privacy.** Events reference `WikiEntry` records and might leak across project boundaries via cross-references. Mitigation: relations include `project_root`; cross-project relations require explicit opt-in.

## Acceptance criteria

- [ ] `event` table exists in surreal-memory schema.
- [ ] Librarian writes events to surreal-memory on every emit.
- [ ] Each `event` has `affects` relations to the wiki_entries it touched.
- [ ] `pk events list` returns recent events.
- [ ] `pk events for-entry <id>` returns the derivation chain for an entry.
- [ ] Performance: 1000-event ingestion run completes within 2x the in-memory baseline.
- [ ] Migration: existing wiki_entries from before this feature can be backfilled (with `pk events backfill --best-effort` synthesizing 'compile' events from entry metadata).
- [ ] Per-project scoping (per SP-008) honored.

## Implementation steps

1. Add the schema migration (Surreal).
2. Refactor `pk-librarian/src/librarian.rs` to write events on emit.
3. Add the `pk events` subcommand family in `pk-cli`.
4. Implement backfill for existing entries.
5. Test with synthetic ingestion runs measuring before/after performance.
6. Document in `pk-core/docs/EVENTS.md`.
7. Update prometheus-knowledge `CLAUDE.md` with the new `pk events` commands.

## Dependencies

SP-007 (trace capture verification — events overlap with traces conceptually; verify the relationship) and SP-008 (per-project scoping — events must be project-aware).

## Open questions

- Should events also persist into `~/.prometheus/hooks.log` (per SP-006)? Probably yes for cross-cutting visibility — events are a richer form of trace. Pipe events through the hook-log shim for the operator-facing audit, while surreal-memory is the queryable store.
- Compaction: at what age do events become history (move to a cold table)? Recommend events age out after 90 days into a `event_archive` table, queryable but not in the hot path.
- Should the Cherry Studio fork (the Boss) also see events? Eventually yes via MCP; out of scope for this task.
