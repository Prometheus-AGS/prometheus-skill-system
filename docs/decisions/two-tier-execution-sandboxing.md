# Decision: use separate native-process and portable-component execution tiers

**Status:** accepted · 2026-08-05 · release 1.7.0

## Context

Python, Node, and Bash need operating-system process isolation and real interpreter attestation. Portable WebAssembly components need deterministic typed capabilities, exact component authorization, and cross-backend replay. Treating both as one backend either weakens native isolation claims or gives components unnecessary ambient authority.

## Decision

Tier P runs native interpreters only through a supported OS sandbox and emits attested receipts. Tier W runs authorized WebAssembly components under Wasmtime with typed capabilities and emits verified receipts. Baseline policy is one-way tightening for both; Tier W never falls back to Tier P, and Tier P never launches directly when its sandbox is unavailable.

## Alternatives considered

- **One generic subprocess runner:** supports more code quickly, but cannot fail closed when sandbox support is absent.
- **WebAssembly only:** maximizes portability, but excludes existing native scripts and interpreter ecosystems.
- **Container everything:** adds packaging and daemon dependencies without providing deterministic component replay by itself.

## Consequences

Callers choose or auto-resolve a tier from the validated runtime. Platform status is explicit: macOS Seatbelt is locally runtime-certified, Linux Tier P remains runtime-evidence-pending, and Windows Tier P is unavailable. Tier W authorization, limits, and projection parity are independently testable across Cranelift and Pulley.

## Verification

Tier P fixtures execute real Python/Node/Bash and prove filesystem, environment, network, timeout, stream, and artifact fences. Tier W fixtures prove pre-compilation authorization, exact capabilities, resource failure classification, component tamper rejection, and Cranelift/Pulley deterministic projection parity.
