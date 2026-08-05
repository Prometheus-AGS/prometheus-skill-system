# Decision: execution produces portable evidence, not only process output

**Status:** accepted · 2026-08-05 · release 1.7.0

## Context

Agent-generated code can return useful stdout while leaving no durable answer to what ran, which inputs and authority it used, or whether a retry executed twice. Transport success is also weaker than durable completion: the caller can lose a response after the run and receipt are committed.

## Decision

Prometheus Exec treats a signed terminal receipt and its content-addressed evidence as the primary product. Request acceptance, spawn state, ordered events, streams/artifacts, receipt-log entry, and terminal state are persisted in a recovery-safe order. REST, MCP, CLI, embedded, and remote adapters share that service boundary.

## Alternatives considered

- **Return stdout/stderr only:** simpler, but cannot prove code/input/capability identity or reconcile response loss.
- **Sign an ephemeral response:** portable at first, but restart and artifact retention can diverge from the signature.
- **Let each adapter own execution state:** reduces shared code initially, but creates incompatible replay and failure behavior.

## Consequences

The service owns a run ledger, event log, receipt log, and CAS with transactional pins. Callers retain stable request IDs and reconcile by status/events. Storage and ordering are more complex, but retries cannot silently create duplicate evidence-producing runs and verification can happen offline.

## Verification

Fixtures cover same-ID replay/conflict, response loss, crash windows, restart reconciliation, hash-linked receipt/event tamper, transactional artifact retention, and byte-identical contract generation. The canonical details are in [Execution architecture](/docs/execution/architecture-and-tiers).
