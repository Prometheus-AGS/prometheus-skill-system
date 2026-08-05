## Context

Change 002 established transport-independent contracts, durable lifecycle state, CAS ownership, signed receipts, and Tier P. The remaining local execution gap is Tier W: the repository describes `prometheus:component@0.1.0`, but no component adapter instantiates it. Tier W must run in sidecar, embedded desktop, Windows, and mobile forms without depending on native-process actuators, KBD, Sovereign Sync, or remote transport.

## Goals / Non-Goals

**Goals:**

- Execute component-model Wasm deterministically under explicit capabilities and hard resource ceilings.
- Produce `verified` receipts whose outputs can be replayed and compared across Pulley and Cranelift.
- Bind every component to signed-generation or explicit hash-pin provenance before compilation/interpretation.
- Expose one embedded Rust API that the existing FRB/Tauri boundaries can call without creating another runtime singleton.
- Keep Windows/mobile honest: Tier W is available; Tier P is not inferred or emulated.

**Non-Goals:**

- R-class dispatch, Sovereign Sync envelopes, MCP tools, or journal anchoring (change 004).
- Windows Tier P, mobile background execution, arbitrary WASI command modules, dynamic native plugins, or JIT on iOS.
- Making `prometheus-exec` mandatory for agent Bash/Python execution.

## Decisions

### Use Wasmtime component model with two explicit backend profiles

`exec-tier-w` owns Wasmtime configuration and the generated bindings for `prometheus:component@0.1.0`. Desktop defaults to Cranelift; iOS and the portable replay profile use Pulley; Android may select Cranelift only when the host permits executable memory and otherwise falls back to Pulley. Backend name, engine version, component hash, capability hash, and deterministic input set are receipt material. A generic WASI command adapter was rejected because it exposes a broader ambient host surface and does not implement the authored component contract.

### Keep host capabilities narrow and typed

The linker implements only declared `fs:read`, output-scoped `fs:write`, `time:now`, and `random` operations. Network, environment access, and broader writes require policy/grant approval before instantiation. `host:exec` and `host:memory` have no Tier W import implementation and fail validation before execution. Preopened ambient WASI directories and inherited environment are disabled.

### Enforce three independent resource fences

Fuel bounds deterministic compute, epoch deadlines bound wall time, and store/resource limiters bound linear memory, tables, instances, streams, and output bytes. Exhaustion becomes a deterministic failed/trapped receipt and never falls through to Tier P. Relying on only one mechanism was rejected because fuel does not bound blocked host calls and epoch interruption does not cap memory.

### Authorize bytes before engine work

Estate mode accepts a component only when its digest and signer are present in the active verified plugin-generation manifest. Standalone and bundled-mobile modes accept only exact configured digests. Authorization happens before component validation, compilation, caching, or host linking. The cache key includes engine/config identity and the authorized component digest; activation rollback cannot leave payload and index generations split.

### Extend existing lifecycle and receipts

Tier W requests use the existing durable `ExecutionService`, events, idempotency ledger, receipt log, and CAS. `exec-core` dispatches through an `ExecutionPort`; `exec-tier-w` cannot import `exec-tier-p`. Verified replay is an offline verifier option that re-instantiates the exact component/input/capability tuple and compares canonical outputs, excluding timestamps and measured usage from the deterministic comparison.

### Use one embedded API and caller-owned presentation

`skill-ffi` exposes JSON-in/JSON-out run, status/events, receipt, artifact, and verify operations over one process-global Rust runtime already owned by `gen_ui_core`. CPU-heavy compilation/replay runs under `spawn_blocking`; UI state remains in Riverpod/Zustand adapters. The same Rust API is callable by Tauri commands. A second daemon or FFI-specific execution kernel was rejected.

## Risks / Trade-offs

- **Wasmtime increases mobile binary size** → measure per-ABI delta before claiming mobile readiness; the gate is `<12 MiB`, and an overage leaves mobile Tier W pending rather than silently stripping limits.
- **Pulley/Cranelift may expose backend differences** → certify the reference component and property corpus across both, canonicalize output contracts, and reject verified status on divergence.
- **Physical devices may be unavailable on this host** → retain simulator/cross-build evidence separately and mark iPhone/Android runtime certification pending until real round trips exist.
- **Component cache can outlive trust activation** → key cache entries by immutable generation/digest and re-check active authorization on every load.
- **Host functions can reintroduce nondeterminism** → time/random are explicit granted inputs recorded in the replay material; undeclared ambient state is unavailable.

## Migration Plan

1. Add contracts/core Tier W identities without changing Tier P serialization.
2. Add `exec-tier-w` and the deterministic reference component behind an opt-in feature.
3. Wire service/CLI replay, then the embedded API and FFI bindings.
4. Enable desktop profiles after local certification; enable each mobile target only after cross-build, size, and physical-device evidence.
5. Roll back by disabling the Tier W feature/route and component generation pointer; existing contracts, Tier P records, receipts, and CAS data remain readable.

## Open Questions

- Which physical Android device will provide the release evidence remains environment-dependent.
- Hardware-backed P-256 mobile receipt keys remain deferred; schema agility already exists and software Ed25519 is the change-003 default.
