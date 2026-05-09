---
id: SP-020
title: Memory dual-store separation (KG vs episodic)
status: planned
priority: P1
estimated_effort: 3-5d
agent_role: rust-codegraph
depends_on: [SP-019]
unblocks: []
related: [SP-008]
created_from_conversation_turn: 3-4
---

# SP-020 — Memory dual-store separation

## Problem

The surreal-memory store currently mixes two kinds of memory in the same tables:

- **Knowledge graph (KG) memory** — durable, factual, generalized: "this entity, this property, this relation." Lifecycle: long. Query pattern: graph traversal, entity resolution.
- **Episodic memory** — temporal, session-scoped, narrative: "during this session, the user did X then Y." Lifecycle: short to medium. Query pattern: timestamp range, semantic search.

Mixing them produces:
- Confused query patterns (do I want all entities like X, or events around X?).
- Confused retention policies (KG entries should rarely expire; episodic should fade).
- Confused embedding strategies (KG benefits from entity-aware embeddings; episodic benefits from contextual session embeddings).

## Evidence

Inspect the current surreal-memory schema. Tables with names like `memory`, `entity`, `relation` are KG-shaped. Tables with `event`, `session`, `interaction` are episodic-shaped. They likely co-exist in the same namespace with no separation.

## Why it matters

Without separation:
- A query for "what does the user know about X" is over-noised by every recent session that mentioned X.
- Retention policies are stuck at the more conservative end (don't expire anything).
- Embeddings are computed once per record without considering which class of retrieval pattern it serves.

## Proposed fix

Split the store into two logical namespaces (in Surreal terms: two databases under the same namespace, or two table prefixes):

**Knowledge graph store (`kg_*` prefix or `kg` database):**
- `kg_entity`, `kg_relation`, `kg_property`.
- Long retention.
- Entity-aware embeddings (skill: see Anthropic's "knowledge graphs for LLMs" or comparable).
- Updated by deliberate, reviewed librarian operations.

**Episodic store (`episode_*` prefix or `episode` database):**
- `episode_session`, `episode_interaction`, `episode_artifact`.
- Configurable retention (default: 90 days).
- Session-context embeddings.
- Updated continuously by hooks and the librarian's event emitter (per SP-019).

**Cross-store relations** (a small bridge):
- An `episode_interaction` can `mention` a `kg_entity` (relation type lives in the bridge layer).
- A `kg_entity`'s `derived_from` relation can point to past `episode_session` records that contributed to its compilation.

**Migration:**
- A one-time `pk migrate-stores` command inventories the current single-store records, classifies each (KG vs episodic) using heuristics + `kind` field if available, and moves accordingly.
- Records that can't be classified land in a `migration_unsorted` bucket; the operator triages.

## Trade-offs and risks

- **Risk: classification heuristic is wrong.** Mitigation: inventory + dry-run before move. Operator approves before commit.
- **Cost: migration takes time on a populated store.** Acceptable; one-time.
- **Risk: cross-store queries become more complex.** Mitigation: shared query helpers; the `kg.*` ↔ `episode.*` bridge is well-documented and exposed via `pk` CLI commands.
- **Conceptual complexity.** Two stores instead of one. Justifiable because the query patterns and retention policies actually differ.

## Acceptance criteria

- [ ] Schema separates KG and episodic into distinct namespaces or table prefixes.
- [ ] Cross-store relation mechanism documented and tested.
- [ ] Migration command exists, tested with dry-run + apply modes.
- [ ] Episodic retention default of 90 days is configurable.
- [ ] KG and episodic embeddings are computed with their respective strategies.
- [ ] `pk` CLI exposes commands for both stores.
- [ ] Performance: queries scoped to one store are no slower than the unsplit baseline.

## Implementation steps

1. Define the two-store schema in surreal-memory.
2. Implement the migration command with dry-run.
3. Implement cross-store relation helpers.
4. Update librarian writes (post SP-019) to route events into episodic, knowledge-derivation into KG.
5. Update query helpers in `pk-mcp` to expose store-scoped queries.
6. Test migration on a synthetic dataset.
7. Document.

## Dependencies

SP-019 (event persistence is a prerequisite — episodic store can't be useful without events as first-class entities).

## Open questions

- Should the episodic retention be per-project? Likely yes (ties to SP-008). Each project has its own retention knob.
- Is there a third store that emerges from this split (e.g. "patterns" — generalizations across many sessions)? Possibly. Don't pre-build it; let it emerge if needed.
