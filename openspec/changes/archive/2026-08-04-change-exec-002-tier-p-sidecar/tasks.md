## 1. Crate boundaries and core primitives

- [x] 1.1 Create `exec-core`, `exec-tier-p`, and `exec-service` crate manifests with enforced dependency direction and version 1.7.0
- [x] 1.2 Implement execution job/port abstractions, deterministic policy outcomes, and receipt assembly in `exec-core`
- [x] 1.3 Implement atomic artifact CAS storage, safe output collection, pins, and budgeted garbage collection
- [x] 1.4 Implement immutable single-writer receipt-log appends using the contracts segment format

## 2. Policy, grants, and native sandbox

- [x] 2.1 Implement the hard auto-approval ceiling and Cedar tighten-only policy evaluation
- [x] 2.2 Implement SSH-signed and interactive grant validation with canonical grant hashes
- [x] 2.3 Implement macOS Seatbelt profile generation, execution, process-group timeout, output bounds, and profile hashing
- [x] 2.4 Implement Linux bwrap command construction and Landlock enforcement classification without unsandboxed fallback
- [x] 2.5 Add sandbox escape, environment, network, timeout, output, and unsupported-platform fixtures

## 3. Durable service and sidecar API

- [x] 3.1 Implement the durable request/run ledger, same-hash replay, hash conflict, terminal commit ordering, and restart reconciliation
- [x] 3.2 Implement ordered durable events and response-loss retrieval in the transport-independent service
- [x] 3.3 Implement health-first mode-0600 UDS binding, same-UID peer credentials, readiness state, and REST/SSE routes
- [x] 3.4 Extend `prometheus-exec` with daemon, run, status, and non-mutating doctor commands

## 4. Local certification and handoff

- [x] 4.1 Run local format, warnings-denied Clippy, unit/property/integration, sandbox, restart, API, and false-green doctor tests
- [x] 4.2 Record macOS runtime evidence and explicit Linux/Windows dispositions without using hosted CI
- [x] 4.3 Update phase evidence/progress and prepare the dependency-ordered Tier P local commit
