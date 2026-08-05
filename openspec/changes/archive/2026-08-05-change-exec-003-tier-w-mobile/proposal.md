## Why

Prometheus has portable signed execution contracts and a certified native-process path, but the authored `prometheus:component@0.1.0` tier still cannot execute. Tier W is required to make deterministic, replay-verifiable execution available on desktop, Windows, iOS, and Android without introducing an unsandboxed native-process fallback.

## What Changes

- Add a Wasmtime component-model adapter with explicit Cranelift/Pulley backend selection, fuel, epoch, memory, stream, and artifact limits.
- Implement the `capabilities.wit` host boundary while permanently denying `host:exec` and `host:memory` inside Tier W.
- Load components only from an authorized signed plugin generation or an explicitly pinned standalone/bundled hash.
- Extend receipt verification with deterministic Tier W replay and cross-backend output comparison.
- Add one embedded Rust API and flutter_rust_bridge surface for run, event, receipt, artifact, and verification operations.
- Certify desktop execution locally and record physical-device/mobile and binary-size evidence honestly; missing physical devices remain pending evidence rather than a false-green result.
- Keep Tier W independent of Tier P, KBD, Sovereign Sync, and remote dispatch. R-class transport remains change 004.

## Capabilities

### New Capabilities

- `wasm-component-execution`: Deterministic execution of `prometheus:component@0.1.0` components with capability and resource enforcement across Cranelift and Pulley.
- `execution-component-provenance`: Authorization of component bytes through signed generations or explicit hash pins, bound into verified receipts.
- `execution-mobile-ffi`: A single embedded Rust/FRB execution surface with ordered events and portable receipt verification for iOS and Android consumers.

### Modified Capabilities

- `execution-artifact-cas`: Define Tier W receipt retention and the bounded mobile artifact-store profile.
- `execution-sidecar-service`: Route Tier W requests through the same durable service lifecycle and expose deterministic replay verification without weakening Tier P behavior.

## Impact

- Adds `substrate/exec-tier-w` and reference Wasm components/fixtures.
- Extends `exec-contracts`, `exec-core`, `exec-service`, and `crates/prometheus-exec` for component requests, verified receipts, and replay.
- Extends `skill-ffi`/`gen_ui_core` only through the existing single FFI boundary; no new runtime singleton or UI-owned execution logic is introduced.
- Adds pinned Wasmtime component-model/Pulley dependencies and records their versions in the repository version manifest.
- Does not add hosted product tests, invoke KBD/Sovereign services, or restrict direct Bash/Python use.
