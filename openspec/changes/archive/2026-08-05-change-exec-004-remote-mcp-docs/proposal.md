## Why

The execution kernels and durable local service now produce verifiable Tier P and Tier W receipts, but harnesses, enrolled peers, certification consumers, installers, and public documentation cannot yet use that capability through one coherent release surface. This change completes the product boundary without making `prometheus-exec` mandatory for ordinary agent shell or Python work and without claiming unavailable remote or mobile evidence.

## What Changes

- Add MCP stdio tools for run submission, status/events, receipt, artifact, and offline verification, all backed by the same transport-independent service behavior as REST and embedded callers.
- Add an estate-only `exec-remote` adapter for signed, enrolled-peer R-class dispatch, durable store-and-forward delivery, replay protection, and per-peer terminal receipt aggregation.
- Add receipt-backed certification evidence references and explicit `pending_evidence` behavior while preserving method-independent certification and `pending_review` for unavailable judges.
- Install, sign, diagnose, and distribute `prometheus-exec` plus its trusted Tier W component through the existing strict installer, doctor, immutable plugin generation, rollback, and 14-target receipt contracts.
- Publish canonical OpenAPI, Docusaurus, guide, ADR, platform/status, troubleshooting, and real-use-case documentation generated from the finished contracts and locally verified against the code.
- Record honest evidence boundaries: no installed KBD or Sovereign service is invoked; disposable isolated peers may be used for product tests; remote multi-peer, mobile size, and physical-device claims stay pending when their required environment is unavailable.

## Capabilities

### New Capabilities

- `execution-mcp-surface`: Envelope-free MCP stdio tools that share local execution semantics and return portable signed evidence.
- `execution-remote-dispatch`: Signed R-class routing, enrolled-peer authorization, store-and-forward queues, replay defense, and receipt aggregation.
- `execution-certification-evidence`: Method-independent receipt citations, evidence resolution, and honest pending-evidence status.
- `execution-release-distribution`: Strict installation, non-mutating diagnosis, signed plugin publication, target receipts, rollback, and release metadata for the execution engine.

### Modified Capabilities

- `execution-sidecar-service`: Extend the shared service boundary and readiness model for MCP and optional remote dispatch without coupling local health to remote availability.
- `execution-component-provenance`: Require the released reference component and execution metadata to be covered by the activated signed generation and all target receipts.
- `docusaurus-docs-site`: Add canonical execution documentation, generated API references, diagrams, use cases, and local drift checks.

## Impact

- New Rust workspace: `substrate/exec-remote`; new MCP mode and shared adapters in `crates/prometheus-exec`.
- Existing execution contracts, service APIs, CLI, OpenAPI generation, doctors, installers, plugin generation scripts, target receipts, docs synchronization, and Docusaurus navigation are extended.
- Estate builds gain optional remote transport integration; standalone and bundled-mobile builds remain free of Sovereign Sync, KBD, and remote dependencies.
- Local certification gains disposable-peer and MCP use-case fixtures; GitHub remains limited to deterministic documentation synchronization and Pages deployment.
