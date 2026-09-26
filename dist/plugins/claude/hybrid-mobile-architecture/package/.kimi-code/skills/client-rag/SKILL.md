---
name: client-rag
description: ALWAYS invoke when adding vector search, embeddings, semantic recall, or retrieval-augmented context to any surface — searching chat history, giving client-side agents memory over conversations, "search my notes", similarity, or wiring pgvector/sqlite-vec. Retrieval runs ENTIRELY on-device (client RAG, not cloud RAG); embeddings are 384-dim everywhere; vault-class data indexes separately and never lands in synced tables. Triggers on RAG, retrieval, vector, embedding, embed, semantic search, similarity, pgvector, sqlite-vec, HNSW, fastembed, search chat history, search conversations, agent memory, recall, context window assembly, knowledge base, BM25, hybrid search, rerank.
---
<!-- TJ-ARCH-MOB-001 compliant -->

> **Binding:** Full design:
> [`client-rag.md`](references/client-rag.md). Lanes and privacy:
> `sync-doctrine` skill. Retrieval remains local and privacy-class boundaries
> remain mandatory even when the optional reference is unavailable.

# Client-Side RAG

## Fixed decisions (do not re-decide per feature)

- **384 dimensions, everywhere** (all-MiniLM-L6-v2 class; matryoshka-truncate
  larger models). One dimension means vectors survive cross-device sync.
- **Engine per tier**: pgvector-in-PGlite (web) · pgvector via pglite-oxide
  (desktop) · sqlite-vec (mobile). Same repository trait; SQL differs behind
  the adapter only. SurrealDB graph-RAG (`gen_ui_db_graph`) is an OPTIONAL
  benchmark-gated add-on — disabling it must not change API or ordering.
- **Embedding runs on-device** in `gen_ui_core` (fastembed/candle,
  `spawn_blocking`). No server round-trips inside the retrieval loop.
- **Embeddings are derived data**: nullable columns, recomputable, backfilled
  idempotently (`embedded_at IS NULL`), and NOT synced unless the entity type
  explicitly opts its columns into columnar sync (ADR-LFS-3).

## Ingestion recipe (embed-on-write)

1. Row commits through the normal PEM write path.
2. After-commit hook enqueues `EmbedJob{table,id}` into the LOCAL job queue
   (separate from the sync `_operation_queue` — different lifecycle).
3. Worker computes the vector, single `UPDATE … SET embedding, embedded_at`.
4. UI never waits; retrieval sees fewer candidates until backfill completes.

Schema shape (additive, LFS-INV-3):

```sql
ALTER TABLE messages ADD COLUMN IF NOT EXISTS embedding vector(384);
ALTER TABLE messages ADD COLUMN IF NOT EXISTS embedded_at timestamptz;
CREATE INDEX IF NOT EXISTS messages_embedding_hnsw
  ON messages USING hnsw (embedding vector_cosine_ops);
```

## Retrieval recipe (one API, fixed pipeline)

Call `RagEngine::retrieve(RetrievalQuery, token_budget)` — never hand-write
retrieval SQL in features. Pipeline: embed query → HNSW top-4k (+ BM25/FTS
where present, RRF-merged) → dedup by source entity → `min_score` floor →
recency tiebreak → cut to k and token budget → `Vec<RetrievedChunk>` with
provenance. Scopes: `ThisConversation | AllConversations | AgentMemory |
Vault`. Chat messages are already the right chunk size — no chunker pipelines;
split artifacts at paragraph boundaries only past ~1k tokens.

Agents: UAR calls the governed `retrieve()` capability before planning with an
authorized scope; approved conclusions are written back as entities (lane 1)
or vault facts (`local`) — never directly into indices.

## Privacy rules (review-blocking)

- `Vault`-scope chunks carry `privacy_class: local`: momentary prompt context
  only — never persisted server-side, never written into synced tables.
- An embedding of `local` data is `local`: vault vectors live in a separate
  local-only index.
- Client RAG ≠ cloud RAG. A server-side retrieval feature is a different
  design with its own privacy review — do not blur them in one API.

## Checklist

- [ ] 384-dim; engine matches the tier matrix; no second vector store
- [ ] Embed-on-write + idempotent backfill; UI never blocks on embedding
- [ ] Features call `RagEngine::retrieve` only; scope chosen by intent
- [ ] Vault vectors in the local-only index; provenance on every chunk
- [ ] Boundary tests: ingest→retrieve round-trip on a real store; ordering
      contract asserted (no mocks of the engine)
