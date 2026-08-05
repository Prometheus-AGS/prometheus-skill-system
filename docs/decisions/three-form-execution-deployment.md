# Decision: support estate sidecar, standalone embedded, and bundled-mobile deployment

**Status:** accepted · 2026-08-05 · release 1.7.0

## Context

Managed workstations need signed plugin-estate trust and a durable same-user service. Desktop applications need an estate-free in-process API. Mobile applications need no-JIT execution and a narrow FFI boundary. Requiring every host to run the full Prometheus estate would prevent legitimate embedded use; making estate trust optional inside the managed sidecar would weaken authorization.

## Decision

Prometheus Exec has three local deployment forms. Estate sidecar mode authorizes Tier W through the active signed generation and serves REST/MCP over local process boundaries. Standalone embedded mode uses explicit hash pins and Cranelift. Bundled-mobile embedded mode uses compiled-in pins and the Pulley no-JIT profile. All forms retain private durable ledgers, CAS, events, signed receipts, and offline verification.

## Alternatives considered

- **Sidecar everywhere:** consistent operations, but unsuitable for mobile and unnecessarily heavy for embedded desktop hosts.
- **Embedded everywhere:** simple packaging, but duplicates daemon lifecycle/identity work and loses managed generation trust.
- **One feature set with runtime switches:** risks accidentally compiling estate, native-process, or JIT authority into constrained targets.

## Consequences

Cargo features enforce deployment boundaries. The embedded API never creates a Tokio runtime, and UI/FFI adapters never accept private signing keys. Separate profiles increase build-matrix cost. Mobile cross-build success also remains distinct from size and physical-device certification.

## Verification

Dependency-direction checks prevent forbidden edges. Estate, standalone, bundled-mobile, iOS, and Android builds assert profile selection. Embedded restart/response-loss tests use the same receipt correctness, while mobile size and physical-device requirements retain their honest blocked or pending statuses.
