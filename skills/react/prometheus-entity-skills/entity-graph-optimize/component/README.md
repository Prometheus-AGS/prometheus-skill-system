# `entity-graph-optimize` as a WASM component

Reference component for `prometheus:component@0.1.0` — the first real skill
built against the unified world (`change-msp-006`).

## Status: well-formed, **execution unproven**

| Claim | Status |
|---|---|
| builds with `cargo component` | **yes** |
| `wasm-tools validate --features component-model` | **passes** |
| exports `run` and `describe` from the `skill` world | **yes** |
| sits where UAR discovery looks (`skill.wasm` beside `SKILL.md`) | **yes** |
| **has actually been executed by a host** | **NO** |

That last row is the point. UAR's Wasm tier is a stub
(`wasm_runtime.rs:92-111`): it loads components and returns a placeholder string
without instantiating them. Until `change-msp-008` de-stubs it, this artifact is
**proven well-formed and nothing more**. Describing it as working end-to-end
would be the "demonstrated, not enforced" mistake this phase keeps catching.

## What it ports, and what changed

Ports `scripts/detect-orchestrators.sh` — probe for orchestrator markers, emit
JSON. Two deliberate differences:

1. **Filesystem probes go through the `kv-store` capability.** A component
   cannot call `[ -e path ]`. The guest declares what it needs and the host
   decides whether to grant it; a component importing an interface outside its
   granted set fails before any guest code runs.

2. **A denied probe is an error, not `false`.** The shell version could not
   distinguish "marker absent" from "probe failed" and reported both as absence.
   The component returns `capability-denied` for the second. Preserving the
   shell behaviour would have carried a real bug across the port.

The shell script **stays**. This does not replace it.

## Rebuild

```bash
bash build.sh          # builds, validates, copies to ../skill.wasm
```

`skill.wasm` is committed because UAR consumes this repo as a submodule and must
not need a Rust toolchain to obtain it. `build.sh` exists so the binary is never
the only copy of the truth.
