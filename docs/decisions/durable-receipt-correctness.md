# Decision: durable receipts, not transport outcomes, define write correctness

**Status:** accepted · 2026-08-03 · release 1.6.1

## Context

Hooks and workers can lose an HTTP response after the server has durably accepted a write. A process can also be alive while its storage or model executor is unavailable. Treating a timeout as failure creates duplicate logical memories; treating a retry attempt as success loses writes.

## Decision

Every durable write uses the Memory v2 operation ledger. The caller generates a stable `operation_id`, computes a SHA-256 hash over canonical compact JSON payload bytes, and retains both until a terminal receipt exists.

- New durable acceptance returns `202`.
- Same-ID/same-hash replay returns `200` with the stored receipt.
- Same-ID/different-hash reuse returns `409` and never overwrites the original.
- `committed` and `rejected` are the only terminal states.
- GET-by-ID and resumable ordered SSE reconcile response loss and reconnects.
- `/health` is liveness; `/ready` is ingestion capability.

Long memories persist their tokenizer plan and per-part embedding state. Executor generations resume unfinished parts and commit one logical memory.

## Alternatives considered

- **Direct v1 write plus transport retry:** rejected because response loss cannot distinguish accepted from unaccepted work.
- **Server-generated operation IDs:** rejected because a caller cannot reconcile a response it never received.
- **Attempt-count terminal inference:** rejected because attempts describe control flow, not durable state.
- **In-memory event stream only:** rejected because process restart would erase reconciliation evidence.

## Consequences

Callers must persist operation identity and handle non-terminal receipts. The server stores ledger, event, plan, part, and executor evidence. This costs storage and coordination complexity, but makes replay, conflict, restart, and audit behavior deterministic.

## Verification

The server contract tests bind Rust request/receipt serialization and HTTP statuses to `openapi/surreal-memory-v2.openapi.json`. Root certification proves health/readiness, exact replay, hash conflict, response-loss reconciliation, terminal retrieval, SSE resume, and long-memory execution.

