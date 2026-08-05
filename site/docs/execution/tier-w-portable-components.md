---
title: Tier W portable component operations
description: Authorize and run deterministic Prometheus WebAssembly components across desktop, embedded, replay, and mobile profiles.
---

# Tier W portable component operations

Tier W is for business logic that should be portable without receiving ambient host authority. The unit of execution is a WebAssembly component implementing the canonical `prometheus:component@0.1.0` world. Authorization, capability linking, resource limits, and deterministic host values are bound into the resulting receipt.

## The component contract

Every component exports:

```wit
export run: func(input: string) -> result<string, error>;
```

It may export `describe()` and may import only the typed capability interfaces allowed by the host:

- `log`
- `kv-store`
- `input`
- `output`
- `clock`
- `random`

The input and return value are strings so existing JSON-speaking skills can adapt without one global application schema. Named binary inputs and artifacts use typed host interfaces rather than ambient files.

## Authorization precedes compilation

Component bytes are untrusted until an authorization source binds their hash and expected capability surface:

| Deployment | Authorization source |
| --- | --- |
| Estate desktop | Active Ed25519-signed plugin generation |
| Standalone embedded | Explicit exact component hash pins |
| Bundled mobile | Compiled-in exact pins |
| Portable replay | Receipt-bound component identity plus caller-supplied exact bytes |

The host verifies authorization before component validation, compilation, cache lookup, linking, or instantiation. Rollback or trust-store changes therefore invalidate stale cached authorization instead of leaving compiled bytes implicitly trusted.

## Deterministic capability theory

A deterministic component is not simply a module that usually returns the same value. Every nondeterministic input must be explicit:

- `clock.now-ms` returns the value granted in the request;
- `random.bytes` consumes a finite replayable byte sequence;
- named inputs bind exact SHA-256 content;
- K/V state is capability-scoped and included in the evidence projection where relevant;
- stdout, environment, preopens, TCP, and UDP are closed unless represented by the contract; and
- fuel, epoch, memory, table, instance, stack, stream, and artifact limits produce stable failure classifications.

The receipt's deterministic projection binds component authorization, engine version, capability values, limits, input, output, artifacts, logs, random consumption, state, and failure. It excludes backend-specific profile identity, timestamps, wall time, and measured fuel so supported backends can be compared honestly.

## Cranelift and Pulley

Desktop estate and standalone profiles use Cranelift for native execution. Portable replay and bundled-mobile profiles use Pulley with a no-JIT policy. Both run through Wasmtime 46.0.2 in this release.

“No JIT” does not mean the dependency graph contains no Cranelift-named compiler crate. Wasmtime still needs compiler machinery to translate a component into Pulley bytecode. The enforceable claim is the selected Pulley execution profile and `jit_permitted=false`.

## Build and run path

1. Put reusable domain logic in a normal Rust library.
2. Add a component adapter implementing the Prometheus WIT world.
3. Build a WebAssembly component, normally for `wasm32-wasip2` through cargo-component.
4. Inspect imports and reproduce the build byte-for-byte.
5. Add the exact component and capability metadata to a signed plugin generation or explicit pin set.
6. Submit with `--runtime wasm-component`.
7. Preserve the request, receipt, component, input bytes, public key, environment, and artifacts for replay.

```bash
prometheus-exec run \
  --socket ./runtime/exec.sock \
  --state-dir ./exec-state \
  --identity ./exec-identity.json \
  --plugin-root "$HOME/.prometheus/plugins/prometheus-skill-pack" \
  --runtime wasm-component \
  --code ./skills/react/prometheus-entity-skills/entity-graph-optimize/skill.wasm \
  --format json
```

The reference component is authorized by the active signed plugin generation. Passing an unrelated component with the same filename or world name fails because authorization binds exact bytes.

## Portable replay

After the receipt signature and request binding verify, `verify` can re-execute Tier W through the portable Pulley profile:

```bash
prometheus-exec verify \
  --receipt ./receipt.json \
  --request ./request.json \
  --public-key '<unpadded-base64url-public-key>' \
  --component ./skill.wasm \
  --input records=./records.json \
  --format json
```

Replay compares terminal state, output and artifact hashes, failure classification, component authorization, engine version, and deterministic projection. A valid signature with mismatched component or input bytes is not a successful replay.

## Desktop, embedded, and mobile

- **Desktop estate:** signed generation, private sidecar, Cranelift, full durable service.
- **Standalone embedded:** exact pins, in-process async Rust API, private ledger/CAS, Cranelift.
- **Bundled mobile:** exact compiled-in pins, FFI, Pulley/no-JIT, private local evidence state.
- **Portable verifier:** caller-supplied receipt-bound bytes, Pulley, no daemon or network.

Mobile cross-builds exist, but fair retained-graph size deltas exceed the release requirement and no physical-device runtime evidence exists. Tier W's design supports mobile profiles; this release does not call them release-ready.

## Not the LibreFang guest ABI

The native-agent `librefang-wasm` target exports `alloc` and `execute` from a `wasm32-unknown-unknown` core module and calls LibreFang host functions. Tier W loads a component implementing Prometheus WIT. The formats, host imports, authorization, and receipt semantics differ. Share domain logic, then build two explicit adapters when both deployments are required.

Next: [Local API, CLI, and MCP](./local-api-cli-and-mcp.md).
