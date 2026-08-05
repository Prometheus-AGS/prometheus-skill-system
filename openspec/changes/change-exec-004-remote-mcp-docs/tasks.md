## 1. MCP shared-service surface

- [x] 1.1 Extract a reusable local execution facade for submit, status, events, receipt, artifact, and verification without duplicating durable state behavior
- [x] 1.2 Add `prometheus-exec --mode mcp` with typed rmcp tools, bounded deterministic envelopes, host-owned identity, and no private-key arguments
- [x] 1.3 Generate and drift-check MCP schemas and add cross-surface idempotency, event-cursor, artifact-ceiling, and verification fixtures

## 2. Estate-only remote dispatch

- [x] 2.1 Create the `exec-remote` 1.7.0 crate, signed envelope/enrollment/aggregate contracts, feature isolation, and dependency-direction enforcement
- [x] 2.2 Implement the immutable durable dispatch queue, expiry, same-ID replay/conflict behavior, enrollment/signature checks, and restart reconciliation
- [x] 2.3 Implement the injected transport/origin/target flow, local-service execution handoff, verified per-peer receipt return, and mixed-outcome aggregation
- [x] 2.4 Add two-disposable-peer dispatch, unknown-endpoint, signer-mismatch, replay, response-loss, offline-resume, expiry, restart, and slow-transport isolation fixtures

## 3. Certification evidence integration

- [x] 3.1 Add a deterministic portable execution-evidence index and offline checker for receipts, requests, public identities, artifacts, environments, and hashes
- [x] 3.2 Integrate `pending_evidence`, preserve distinct `pending_review`, prove method independence, and add separated artifact/runtime/install/remote/mobile status fixtures

## 4. Installation, doctors, and signed distribution

- [x] 4.1 Add `prometheus-exec` to managed binary/version/hash manifests and strict atomic install/sign/readback flows without best-effort false success
- [x] 4.2 Extend root and execution doctors for binary, service, UDS, readiness, trust, CAS/receipt reconciliation, MCP schema, and optional remote queue checks with pre-construction exclusions
- [x] 4.3 Publish the exact reference component and capability metadata inside one signed immutable plugin generation and verify tamper, rollback, index parity, and 14 target receipts

## 5. Canonical APIs and documentation

- [x] 5.1 Extend OpenAPI, generated request/receipt/MCP references, CLI/config tables, platform/evidence status, component hash, target count, and release drift contracts
- [ ] 5.2 Add the canonical Docusaurus Execution section with local/remote use cases, architecture, APIs, receipts, certification, operations, platform status, and parser-checked Mermaid diagrams
- [ ] 5.3 Update the numbered guide, crate READMEs, installation/troubleshooting references, and ADRs for evidence-producing execution, two-tier sandboxing, three-form deployment, and method independence

## 6. Local certification and phase completion

- [ ] 6.1 Run focused format, warnings-denied Clippy, unit/property/integration, dependency, generated-diff, installer/doctor, plugin, OpenAPI, docs, and false-green gates locally
- [ ] 6.2 Execute and archive redacted real MCP, local Tier P/W, offline verification, response-loss, restart, and disposable remote use cases with explicit external-evidence dispositions
- [ ] 6.3 Install/sign/read back the final binary and generation, complete artifact-refiner and distinct-model review, verify/archive the OpenSpec change, and close the KBD phase with reflection
