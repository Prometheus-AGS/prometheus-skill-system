# Decision: isolate queue publication from immutable prompt snapshot publication

**Status:** accepted · 2026-08-03 · release 1.6.1

## Context

Stop hooks must return quickly and cannot rely on network or model availability. Prompt readers run concurrently with learning workers and must never observe half-written scope context. Project-specific knowledge must not leak into shared or global prompts.

## Decision

Stop hooks perform one atomic local action: write a private temporary learning record, fsync it, and rename it into `pending`. The hook performs no inference, network request, remote write, or service-manager action.

The worker owns two explicit state machines:

- learning: `pending → processing → completed | rejected`;
- memory delivery: `pending → submitting → accepted → completed | rejected`.

After durable receipt completion, the worker publishes project, shared, and global snapshots independently. Each scope has immutable content-addressed generations and an atomically replaced `current` pointer. Prompt assembly reads one validated generation per scope under deterministic size budgets.

## Alternatives considered

- **Inline Stop-hook learning and writeback:** rejected because hook latency and network failure affect session shutdown.
- **One mutable context file:** rejected because concurrent readers can observe partial content and rollback is impossible.
- **One combined global snapshot:** rejected because it breaks scope isolation.
- **Retry/dead-letter counters as primary state:** rejected because they do not represent receipt truth; legacy directories remain migration evidence only.

## Consequences

The worker and doctors must reconcile stale `processing`, `submitting`, and `accepted` records from durable evidence. Snapshot storage retains old generations. In exchange, hooks stay bounded, readers get atomic scope-consistent context, and restart recovery is evidence-driven.

## Verification

Fixtures exercise atomic enqueue, worker interruption, receipt reconciliation, legacy migration, snapshot continuity, bounded context, and non-mutating `pk doctor`. Release certification requires zero unsettled/legacy records and valid current pointers in all three scopes.

