# 06 · Deterministic Memory and Learning

Prometheus 1.6.1 separates fast local publication from durable remote acknowledgement. Stop hooks atomically enqueue work and return. A supervised worker extracts learning, submits stable v2 operation IDs, reconciles exact receipts, and publishes immutable project, shared, and global prompt snapshots.

## Runtime flow

```mermaid
flowchart LR
  Hook["Stop hook"] --> Queue["Atomic local queue"]
  Queue --> Worker["Learning worker"]
  Worker --> Ledger["Memory v2 operation ledger"]
  Ledger --> Parts["Persisted tokenizer parts"]
  Parts --> Receipt["Terminal receipt"]
  Receipt --> Snapshots["Immutable scoped snapshots"]
  Snapshots --> Prompt["Bounded next-session context"]
```

The hook performs no inference, network request, inline remote persistence, or service-manager work. Queue states make uncertainty explicit. Learning records move through `pending → processing → completed | rejected`; memory delivery moves through `pending → submitting → accepted → completed | rejected`.

## Durable Memory v2

New writes use `POST /api/v2/operations`. A request contains a caller-generated `operation_id`, schema version, operation kind, dependencies, canonical payload hash, and payload. `202` proves durable acceptance. Only `committed` or `rejected` is terminal.

After response loss, callers GET the receipt by ID or resume ordered SSE events. Reusing the same ID and hash returns the exact stored receipt. Reusing the ID with another hash returns `409` and preserves the original operation.

Long logical memories are planned using the active tokenizer. Each part and embedding state is persisted. Executor restart reuses the plan, skips indexed parts, and commits one logical memory only after every part validates.

## Immutable knowledge snapshots

Project, shared, and global scope publish independently. Each writer stages and validates a content-addressed generation, then atomically advances `current`. Prompt assembly reads a bounded, deterministic selection from one complete generation per scope.

## Health and recovery

`/health` proves process liveness. `/ready` proves whether the durable ledger can ingest operations and reports model/search warm-up separately. Wall-clock time and attempt counters never prove success; receipts and ordered events do.

Canonical documentation:

- [Memory overview](/docs/memory/overview)
- [Operation API](/docs/memory/operation-api)
- [Executor and recovery](/docs/memory/executor-and-recovery)
- [Snapshots and bounded context](/docs/knowledge-learning/snapshots-and-context)
- [Hooks, worker, queues, and receipts](/docs/knowledge-learning/hooks-worker-and-receipts)

---

*Previous: [← 05 · The MCP Server Substrate](05-mcp-substrate.md) · Next: [07 · Sycophancy Correction →](07-sycophancy-correction.md)*
