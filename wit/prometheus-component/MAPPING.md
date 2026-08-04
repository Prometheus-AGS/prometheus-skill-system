# `prometheus:component@0.1.0` — how the existing targets map onto it

Authored by `change-msp-005`, implementing
[`docs/decisions/wit-world-unification.md`](../../docs/decisions/wit-world-unification.md).
The decision fixed the shape and the ordering; this file records the mapping,
**including the parts that do not map**.

## The four packages this supersedes

Verified on disk 2026-07-31:

| Package | Version(s) | Location |
|---|---|---|
| `uar:skill` | 0.1.0 | `universal-agent-runtime/wit/uar-skill.wit:12` |
| `uar:plugin` | 0.1.0 | `universal-agent-runtime/wit/uar-plugin.wit:12` |
| `knowme:plugin` | **0.1.0 and 1.0.0** | `knowme_plugin_host/wit/knowme-plugin.wit:17`, `wit/v1/types.wit:14` |

Plus a fifth target that is **not WIT at all** — see below.

## `uar:skill@0.1.0` → `prometheus:component/skill`

`uar:skill`'s entire contract:

```wit
world skill {
  export run: func(input: string) -> result<string, string>;
}
```

| Element | Maps to | Note |
|---|---|---|
| `run(string) -> result<string, string>` | `run(string) -> result<string, error>` | **Superset.** Same input, same success payload. The error side gains a machine-routable `kind`; a host wanting the old shape reads `error.message`. |
| — | `describe() -> string` | **Added, optional.** A host must treat its absence as "no metadata", never as an error. |
| — | `log`, `kv-store`, `input`, `output`, `clock`, `random` | **Added, imported.** Named inputs and output paths are host-validated; clock and random values are host-supplied replay material. A guest importing none is a pure function of its input. |

**Migration cost for an existing `uar:skill` guest: the error type only.** Nothing
is removed or narrowed.

## `uar:plugin@0.1.0` and `knowme:plugin` → `prometheus:component/plugin`

Both are lifecycle-and-events shaped, so both map onto `plugin`'s
`init` / `handle-event` / `shutdown` triple.

`knowme:plugin` carries additional unstable worlds in `wit/v1/worlds-unstable.wit`
(`agent`, `provider`, `service`, `workflow`) and a `hook` world in `worlds.wit`.

> **These do NOT map, and this is the honest part of the table.**
> `prometheus:component@0.1.0` covers `skill` and `plugin` only. The four
> unstable worlds are richer host contracts — an `agent` world implies an agent
> loop, a `provider` world implies model routing — and folding them in would
> mean designing four more host ABIs on the strength of files marked
> *unstable*. They are **deliberately out of scope for 0.1.0**, not overlooked.
> A component targeting them keeps using `knowme:plugin` until they stabilise.

## The librefang core-wasm ABI → **does not map**

`skills/rust/librefang-wasm-skill/templates/` generates guests with:

```rust
#[no_mangle] pub unsafe extern "C" fn alloc(size: i32) -> i32
#[no_mangle] pub unsafe extern "C" fn execute(ptr: i32, len: i32) -> i64
extern "C" { fn host_call(...); }   // JSON-over-pointer
```

Zero `.wit` files. This is **core wasm with a hand-rolled pointer ABI**, not a
component.

**It cannot be mapped, only ported.** A component and a core module are
different binary formats: `wasmtime::component::Component::from_file` will not
load a core module, whatever its exports are named. There is no adapter that
makes `execute(i32,i32) -> i64` into `run: func(string) -> result<string,error>`
without recompiling the guest against a WIT world.

That is a real finding: the pack already ships wasm skill templates that
**cannot run in the host this phase targets**. Whether to port them, keep both
targets, or retire the librefang path is a decision for a later change — it is
not resolved by authoring this world, and this file does not pretend otherwise.

## Version pinning

`prometheus:component@0.1.0` is pinned in its package declaration, in every
file. The invariant that `knowme:plugin` violates — the same package name
declared at two versions — cannot recur here as long as all four files carry
the identical `package` line, which
`skills/devops/fabric-integration/scripts/check-invariants.sh` verifies.

## Adoption status

- `change-msp-006` ports `entity-graph-optimize` as the deterministic reference
  guest.
- `change-uhe-015` executes that guest in UAR and asserts its returned value.
- `change-exec-003` adds the independent `prometheus-exec` Tier W host. Its
  typed capability boundary rejects unsupported imports plus `host:exec` and
  `host:memory` before instantiation; runtime receipt certification remains a
  separate acceptance boundary.
- **`knowme:plugin`'s dual version is not fixed.** It stays quarantined in
  `fabric-integration`'s allowlist until migration is complete.
