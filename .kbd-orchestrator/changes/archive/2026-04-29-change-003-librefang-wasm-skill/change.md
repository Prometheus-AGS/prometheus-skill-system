---
id: change-003-librefang-wasm-skill
title: New skills/rust/librefang-wasm-skill/ with WASM-ABI templates
phase: phase-compliance-and-power-multiplier
gaps: [G1]
priority: P0
effort: M
agent: rust-skills:rust-skill-creator
evolver_item_id: null
status: DONE
completed: 2026-04-29
target_correction: "wasm32-wasip2 → wasm32-unknown-unknown (LibreFang uses core wasmtime, not Component Model)"
---

# change-003 — LibreFang WASM Skill Templates

## Context

This is the foundation of the WASM packaging path. LibreFang's
`crates/librefang-runtime-wasm/src/sandbox.rs` documents a precise Guest ABI that
WASM skills must export:

- `memory` — linear memory.
- `alloc(size: i32) -> i32` — allocate `size` bytes, return a pointer.
- `execute(input_ptr: i32, input_len: i32) -> i64` — packed `(ptr<<32)|len` of
  JSON output.

And imports under module `"librefang"`:

- `host_call(req_ptr: i32, req_len: i32) -> i64` — RPC dispatch.
- `host_log(level: i32, msg_ptr: i32, msg_len: i32)` — logging (max 4096 bytes).

Today the skill pack has no skill that teaches an AI agent how to *generate*
this. The forge-rs `template new skill` flow can scaffold any Rust skill, but
the templates would have to be built from scratch every time. This change makes
"produce a LibreFang-WASM-ABI-compliant skill" a single command.

## Scope

In:

- New skill at `skills/rust/librefang-wasm-skill/` containing:
  - `SKILL.md` — frontmatter (≤200 char description) + body that explains the
    LibreFang WASM Guest ABI, capability model, and host-call surface.
  - `templates/Cargo.toml.tera` — declares `crate-type = ["cdylib"]`, adds the
    minimal deps (`serde`, `serde_json`, plus a `host` module), targets
    `wasm32-wasip2`.
  - `templates/lib.rs.tera` — `#[no_mangle] pub extern "C" fn alloc(size: i32) -> i32`
    using `Vec::with_capacity` + `Box::leak` + `Box::into_raw` patterns,
    `#[no_mangle] pub extern "C" fn execute(...)` reading JSON in / writing JSON
    out, and the `extern "C"` import block for the `librefang` module.
  - `templates/skill.toml.tera` — LibreFang manifest with `[skill]`, `[runtime]
    type = "wasm" entry = "<name>.wasm"`, `[tools]`, `[requirements]
    capabilities = [...]`.
  - `templates/host_bridge.rs.tera` — safe Rust wrapper around `host_call`
    that takes a JSON method/params and returns `Result<Value, HostError>`.
  - `references/librefang-host-abi.md` — exhaustive list of every `host_call`
    method, keyed off `librefang-runtime-wasm/src/host_functions.rs` (Gap G5).
  - `references/capability-model.md` — explains the `Capability` enum from
    `librefang-types::capability` and how to declare them in `skill.toml`.
  - `references/example-walkthrough.md` — annotated build of a tiny "echo"
    WASM skill, end-to-end.
  - `scripts/validate-wasm-abi.sh` — given a built `.wasm` file, uses
    `wasm-tools` to verify the four required exports are present.
- A `skill.toml` at the new skill's root declaring its templates so forge-rs's
  `SkillRegistry.resolve()` finds them.

Out:

- The `forge package-librefang` packaging command — that lives in change-005.
- Native-agent integration — that lives in change-004.

## Deliverables

1. Complete skill directory at `skills/rust/librefang-wasm-skill/`.
2. Tera templates that, when rendered + compiled, produce a `.wasm` that loads
   successfully into `librefang-runtime-wasm`'s `WasmSkillSandbox`.
3. A working "echo" example checked in under `references/example/`.
4. References to the LibreFang fork at `/Users/gqadonis/Projects/references/librefang`
   for ABI ground-truth.

## Acceptance Criteria

- `forge template validate skills/rust/librefang-wasm-skill/` reports clean.
- Rendering the templates with a sample `task_description` produces compilable
  Rust (CI test).
- Building the rendered output with `cargo build --target wasm32-wasip2 --release`
  produces a `.wasm` whose exports include `memory`, `alloc`, `execute`
  (verified by `scripts/validate-wasm-abi.sh`).
- The "echo" example skill, when loaded into a local LibreFang via
  `POST /skills/install`, is callable and round-trips a JSON payload.

## Files to Touch (all new)

- `skills/rust/librefang-wasm-skill/SKILL.md`
- `skills/rust/librefang-wasm-skill/skill.toml`
- `skills/rust/librefang-wasm-skill/templates/{Cargo.toml,lib.rs,skill.toml,host_bridge.rs}.tera`
- `skills/rust/librefang-wasm-skill/references/{librefang-host-abi,capability-model,example-walkthrough}.md`
- `skills/rust/librefang-wasm-skill/references/example/` (full echo skill source)
- `skills/rust/librefang-wasm-skill/scripts/validate-wasm-abi.sh`

## Test Plan

- Unit: render templates, compile with cargo, validate exports.
- Integration: spin up `librefang start` (port 4545), POST the example zip,
  call the skill, expect echoed JSON back.
- Failure mode: render with malformed `task_description`, confirm Tera reports
  the error rather than producing broken Rust.
