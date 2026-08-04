## 1. Contract crate and canonical identities

- [x] 1.1 Create `substrate/exec-contracts` at version 1.7.0 with no internal repository dependencies
- [x] 1.2 Implement revision-1 request, receipt, capability, event, artifact, grant, device, usage, exit, and error types
- [x] 1.3 Implement RFC 8785 canonicalization and strict `sha256:<hex>` identities for requests, receipts, artifacts, and inputs
- [x] 1.4 Implement Ed25519 signing/verification and P-256 verification with algorithm-tagged key IDs and fixed encodings
- [x] 1.5 Implement semantic validation for schema versions, evidence/tier combinations, time ordering, hashes, limits, and safe artifact paths

## 2. Portable verification and receipt archives

- [x] 2.1 Implement structured offline verification results for signatures, optional requests, and optional artifact trees
- [x] 2.2 Implement versioned immutable receipt-log segment types with hash links, entry hashes, count limits, and explicit key resolution
- [x] 2.3 Add canonicalization, mutation, cross-algorithm, artifact-tamper, path-escape, and archive-corruption fixtures

## 3. CLI and generated contracts

- [x] 3.1 Create `crates/prometheus-exec` version 1.7.0 with side-effect-bounded `init`, `verify`, and contract-generation commands
- [x] 3.2 Implement atomic mode-0600 Ed25519 identity creation without private-key disclosure or silent replacement
- [x] 3.3 Generate and check in deterministic JSON Schema and OpenAPI 3.1 contract components
- [x] 3.4 Document exact command behavior, key/signature encodings, verification failure codes, and the boundary between evidence execution and unrestricted agent tools

## 4. Local verification and change handoff

- [x] 4.1 Run local format, check, warnings-denied clippy, unit, integration, property, and generated-diff checks for the new crates
- [x] 4.2 Prove `verify` performs no initialization, socket, KBD, Sovereign Sync, or network activity in isolated fixtures
- [x] 4.3 Record redacted command results and hashes in the phase evidence, complete the OpenSpec checklist, and prepare the dependency-ordered local commit
