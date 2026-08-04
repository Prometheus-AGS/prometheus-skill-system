## Why

Prometheus can describe and distribute agent capabilities, but it does not yet have a portable execution contract that proves what ran, under which limits, and what artifacts were produced. The execution engine must begin with transport-independent, cryptographically verifiable evidence so later local, mobile, and remote backends cannot invent incompatible receipt semantics or claim guarantees they did not enforce.

## What Changes

- Add versioned execution request, capability, policy, grant, event, artifact, error, and receipt schemas shared by every backend and interface.
- Define RFC 8785 canonical payload hashing, Ed25519 signatures, and algorithm-tagged key material with reserved P-256 verification agility.
- Add offline receipt verification that checks signatures, request identity, receipt identity, artifact hashes, and terminal-state invariants without starting a daemon or contacting KBD.
- Add immutable, hash-linked receipt-log segments with deterministic verification and explicit corruption diagnostics.
- Add `prometheus-exec init` and `prometheus-exec verify` commands that are side-effect bounded and usable independently of the future execution service.
- Publish deterministic JSON Schema/OpenAPI components for the public contract.

## Capabilities

### New Capabilities

- `execution-evidence-contracts`: Versioned, canonical request, receipt, event, capability, grant, artifact, and error representations with cryptographic identity rules.
- `execution-receipt-verification`: Offline verification of signed receipts and their referenced artifacts with deterministic pass/fail diagnostics.
- `execution-receipt-log`: Immutable hash-linked receipt segments, archive integrity, and replay-safe verification.

### Modified Capabilities

None.

## Impact

- Adds `substrate/exec-contracts` as the transport-free contract and cryptography crate.
- Adds the initial `crates/prometheus-exec` CLI surface for identity initialization and offline verification.
- Adds checked-in generated contract artifacts consumed by later REST, MCP, FFI, Tier W, Tier P, and remote implementations.
- Introduces Rust dependencies for RFC 8785 canonicalization, SHA-256, Ed25519, P-256 verification, JSON Schema generation, and versioned serialization.
- Does not add hosted validation, restrict Bash/Python use, invoke KBD for execution, or create a runtime daemon in this change.
