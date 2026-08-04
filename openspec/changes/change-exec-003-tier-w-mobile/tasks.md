## 1. Contracts, topology, and reference component

- [x] 1.1 Create the `exec-tier-w` 1.7.0 crate, pin Wasmtime component-model/Pulley versions, add feature profiles, and extend dependency-direction enforcement
- [x] 1.2 Extend request, receipt, backend, provenance, failure, schema, and OpenAPI contracts for Tier W without changing existing Tier P serialization
- [x] 1.3 Check in `prometheus:component@0.1.0` and `capabilities.wit`, plus a deterministic reference component and byte-stable build fixtures

## 2. Tier W engine and trust boundary

- [x] 2.1 Implement target-aware Cranelift/Pulley engine configuration, component validation, cache identity, and honest backend availability
- [x] 2.2 Implement typed capability hosts with declared read/output/time/random access and pre-instantiation denial of unsupported, `host:exec`, and `host:memory` imports
- [x] 2.3 Enforce fuel, epoch, memory, table, instance, stream, and artifact limits with deterministic terminal failure classification
- [ ] 2.4 Implement signed-generation and explicit-hash component authorization before validation/compilation, including cache re-authorization and rollback fixtures
- [ ] 2.5 Execute the reference/property corpus under Pulley and Cranelift and implement deterministic receipt-projection/output comparison

## 3. Core, service, CLI, and standalone integration

- [ ] 3.1 Wire Tier W through `ExecutionPort`, receipt assembly, receipt-first CAS retention, and desktop/mobile budget profiles without importing Tier P
- [ ] 3.2 Route Tier W through the durable service ledger/events/API and extend non-mutating readiness/doctor checks for backend and trust state
- [ ] 3.3 Extend `prometheus-exec run|status|verify` with component submission and offline verified replay while preserving transport-free verification
- [ ] 3.4 Add standalone and bundled-mobile feature profiles with embedded policy, local receipt log, hash pins, and zero estate/KBD/Sovereign dependencies

## 4. Embedded and mobile surfaces

- [ ] 4.1 Add one embedded Rust execution API for run, ordered events, receipt, artifact, and verify operations using the existing process-global runtime
- [ ] 4.2 Expose the embedded API through `skill-ffi`/FRB and Tauri-compatible adapters with returned-value, grant-pending, interruption, and key-boundary fixtures
- [ ] 4.3 Cross-build iOS and Android profiles, certify Pulley selection/no-JIT behavior, and record the per-ABI `gen_ui_core` binary-size delta
- [ ] 4.4 Run receipt-producing round trips on a physical iOS device and Android device, or archive an explicit pending-evidence disposition when hardware is unavailable

## 5. Local certification and handoff

- [ ] 5.1 Run local format, warnings-denied Clippy, unit/property/integration, tamper, replay, cross-backend, service, FFI, and false-green doctor gates
- [ ] 5.2 Archive redacted Tier W receipts/artifacts/measurements, complete artifact-refiner and distinct-model review, update phase progress, and prepare the dependency-ordered local commit
