# The component actually executes

**Change:** `change-uhe-015-uar-wasm-execution`
**Phase:** uar-host-execution
**Goal:** S1

## Why

See `.kbd-orchestrator/phases/uar-host-execution/plan.md` for full rationale,
acceptance criteria, and the two-round adversarial review record.

## Outcome: a wasm component EXECUTED in UAR — S1 moves PARTIAL → MET

```
test result: ok. 2 passed; 0 failed

wasm component returned:
  "{\"kbd\":{\"available\":false},\"evolver\":{\"available\":false},
    \"refiner\":{\"available\":false},\"openspec\":{\"available\":false}}"
```

That is **the guest's own computed JSON**, not the placeholder. The acceptance
criterion — assert on the returned value — is what makes that distinction
visible; a test checking "the call completed" passed against the stub for as
long as the stub existed.

### What the stub was hiding

De-stubbing did not simply reveal missing bindings. The first real instantiate
failed with:

```
component imports instance `prometheus:component/kv-store@0.1.0`,
but a matching implementation was not found in the linker
  1: instance export `get` has the wrong type
  2: function implementation is missing
```

**The host never implemented a capability its own WIT declares.**
`capabilities.wit` defines `kv-store` with `get`/`set`/`delete`; the reference
component imports it; `wasm_runtime.rs` linked only WASI. Any guest using a
declared capability could never have loaded — and nothing could reveal that
while `run()` returned a string without instantiating anything.

Worth noting the negative test (`running_an_unregistered_skill_is_an_error`)
passed throughout. Positive failing while negative passes is the right shape: it
localised the fault to execution rather than to a broken harness.

### The capability, and why it is scoped this way

Three functions wired by hand rather than through bindgen, so the WIT stays the
contract: a guest whose interface has drifted fails at instantiate with a clear
"wrong type" instead of breaking generated code elsewhere.

The backing store is **per-instance and in-memory**:

- sharing one map across skills would let an untrusted guest read another's state
- persisting it would make a "portable" skill quietly stateful across calls

The WASI context is built with **no capability grants** — no preopened
directories, no inherited stdio, no environment. A skill reaches exactly what
`prometheus:component/capabilities` hands it, so "portable" cannot come to mean
"has ambient filesystem access".

### Five rounds of API correction, and the one worth keeping

Error counts: **6 → 2 → 4 → 4 → 1 → 0**. Each round was a wrong assumption about
wasmtime 46, each resolved by reading the vendored source rather than guessing
again:

| Assumed | Actual |
|---|---|
| `wasmtime_wasi::p2::{WasiCtx, IoView}` | root-level `WasiCtx`; one `WasiView` → `WasiCtxView` |
| no derive on the struct | `#[derive(Debug, Default)]` at line 32 |
| `post_return` required | deprecated, no effect |
| `func_new` takes 3 closure args | takes **4**: `(store, ty, params, results)` |
| anyhow and wasmtime errors interchangeable | **distinct types**, and this file imports wasmtime's `Context` |

The last is the one that generalises. wasmtime 46 has its own `Error` *and* its
own `Context` trait, both same-named as anyhow's, and this file sits on that
seam — its header comment says so at line 17. I hit three separate instances of
that single confusion before it stuck.

**Nothing may be described as end-to-end parity until this passes. It passes.**
