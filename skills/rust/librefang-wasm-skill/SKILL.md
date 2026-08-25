---
name: librefang-wasm-skill
description: Generate a LibreFang WASM-ABI-compliant skill in Rust. Produces a cdylib crate that exports the required Guest ABI (alloc, execute, memory) and a host_call bridge wrapping LibreFang's capability-checked host functions (fs_*, net_fetch, kv_*, agent_*, time_now, env_read, shell_exec). Includes a skill.toml manifest matching librefang-skills' SkillManifest schema. Use when building a portable, sandboxed skill that runs inside a librefang/bossfang Agent OS instance via WasmSkillSandbox, or when packaging a Prometheus native-agent for upload to a bossfang URL.
license: MIT
version: '1.0.0'
authors:
  - Prometheus AGS
language: rust
metadata:
  category: rust
  tags: [librefang, bossfang, wasm, wasmtime, wasi, agent-os, skill, packaging]
  guest_abi_version: '1.0'
  librefang_runtime_wasm_version: '>=0.1'
triggers:
  keywords:
    - librefang skill
    - bossfang skill
    - wasm skill
    - librefang wasm
    - sandboxed skill
    - capability skill
    - upload to bossfang
    - package for librefang
    - wasm32-unknown-unknown
  semantic: >
    Build a Rust crate that compiles to a WASM binary loadable into LibreFang's
    WasmSkillSandbox, with a skill.toml manifest declaring runtime, capabilities,
    and tool surface. Apply when a user asks to create a portable agent skill,
    package an agent for librefang/bossfang, or generate WASM-ABI-compliant code.
---

# LibreFang WASM Skill

Generates a Rust skill that compiles to `wasm32-unknown-unknown` and conforms to LibreFang's
WASM Guest ABI as documented in
`crates/librefang-runtime-wasm/src/sandbox.rs` of the LibreFang fork.

## When to Use

- A user wants to build a sandboxed, capability-restricted skill that runs
  inside a LibreFang or bossfang Agent OS instance.
- A user is packaging a Prometheus-generated native agent (from
  `/create-native-agent`) for upload to a bossfang URL via
  `forge package-librefang` + `/upload-to-bossfang`.
- A user asks how to call host functions (`net_fetch`, `kv_get`, `agent_send`,
  etc.) from a WASM guest.

## What This Skill Produces

When `forge enrich` resolves this skill against a task, it renders four Tera
templates plus a working "echo" example into the target project:

```
<target>/
├── Cargo.toml                ← cdylib crate, wasm32-unknown-unknown target
├── skill.toml                ← LibreFang manifest (runtime.type = "wasm")
├── README.md                 ← capability + invocation docs
└── src/
    ├── lib.rs                ← #[no_mangle] alloc, execute, plus host bridge
    └── host.rs               ← safe Rust wrappers around host_call/host_log
```

## Guest ABI (must-implement)

Per `librefang-runtime-wasm/src/sandbox.rs:8-26`, every WASM skill MUST export
exactly these symbols:

| Export | Signature | Purpose |
|---|---|---|
| `memory` | `(memory 1)` (linear memory) | Linear memory shared with host |
| `alloc` | `(func (param i32) (result i32))` | Allocate `size` bytes; return ptr |
| `execute` | `(func (param i32 i32) (result i64))` | Main entry: receives `(input_ptr, input_len)`, returns packed result. See lib.rs.tera for the bit layout. |

The host imports under module `"librefang"`:

| Import | Signature | Purpose |
|---|---|---|
| `host_call` | `(func (param i32 i32) (result i64))` | RPC dispatch: send `{"method","params"}` JSON, receive `{"ok":...}` or `{"error":"..."}` |
| `host_log` | `(func (param i32 i32 i32))` | Log a message (level, msg_ptr, msg_len). Max 4096 bytes; longer is truncated. |

The `lib.rs.tera` template generates all five correctly. Do not hand-edit
unless you are sure the resulting module still loads in `WasmSandbox::new()`.

## Host Call Surface

`librefang-runtime-wasm/src/host_functions.rs:21-47` enumerates every
allowed `method` value. The skill's `skill.toml` MUST declare a matching
`requirements.capabilities` entry for each method it calls (except
`time_now`, which requires no capability).

| Method | Required Capability | Purpose |
|---|---|---|
| `time_now` | (none) | Unix timestamp seconds |
| `fs_read` | `FileRead("<glob>")` | Read a file (canonicalized; no `..`) |
| `fs_write` | `FileWrite("<glob>")` | Write a file (parent canonicalized) |
| `fs_list` | `FileRead("<glob>")` | List a directory |
| `net_fetch` | `NetConnect("<host:port>")` | HTTP fetch with SSRF protection |
| `shell_exec` | `ShellExec("<cmd>")` | Spawn a subprocess |
| `env_read` | `EnvRead("<var>")` | Read an env var |
| `kv_get` | `MemoryRead("<key>")` | Get from kernel KV store |
| `kv_set` | `MemoryWrite("<key>")` | Set in kernel KV store |
| `agent_send` | `AgentMessage("<id>")` | Send a message to another agent |
| `agent_spawn` | `AgentSpawn` | Spawn a child agent |

Full documentation: [`references/librefang-host-abi.md`](references/librefang-host-abi.md).

## Skill Manifest (skill.toml)

`librefang-skills/src/lib.rs` defines the `SkillManifest` struct that
`skill.toml` deserializes into. The minimum WASM manifest is:

```toml
[skill]
name = "{{ skill_name }}"
version = "{{ skill_version | default(value="0.1.0") }}"
description = "{{ skill_description }}"
author = "{{ skill_author | default(value="your-name") }}"
tags = ["{{ skill_tag | default(value="example") }}"]

[runtime]
type = "wasm"
entry = "{{ skill_name | replace(from="-", to="_") }}.wasm"

[[tools]]
# Each tool the skill exposes to the LLM. Names must be unique within the skill.
name = "echo"
description = "Echo a JSON payload back to the caller."
input_schema = { type = "object", properties = { message = { type = "string" } }, required = ["message"] }

[requirements]
# String form of librefang_types::Capability variants. See references/capability-model.md.
tools = []
capabilities = []
```

Full manifest reference with all optional sections: [`references/skill-toml-reference.md`](references/skill-toml-reference.md).

## Build & Validate

The included `scripts/validate-wasm-abi.sh` uses `wasm-tools` to check the
four required exports and absence of forbidden imports. CI should run it
after every `cargo build --target wasm32-unknown-unknown --release`.

```bash
cargo build --target wasm32-unknown-unknown --release
bash scripts/validate-wasm-abi.sh target/wasm32-unknown-unknown/release/{{ skill_name | replace(from="-", to="_") }}.wasm
```

Cargo converts hyphens in package names to underscores in the emitted library
artifact. A skill named `weather-check` therefore builds and packages
`weather_check.wasm`; keep the manifest `entry` aligned with that filename.

If `wasm-tools` is not installed: `cargo install wasm-tools` (or `brew install wasm-tools`).

## Working Example

A complete, runnable echo skill is bundled at
[`references/example-echo/`](references/example-echo/). It demonstrates the
full ABI, manifest, and host-call usage in ~80 lines of Rust. Use it as the
canonical reference when reviewing generated skills.

```bash
cd references/example-echo
cargo build --target wasm32-unknown-unknown --release
# Then load into LibreFang:
#   curl -X POST http://localhost:4545/skills/install \
#        -H "Content-Type: application/zip" \
#        --data-binary @echo-skill.zip
```

## Templates

| Template | Renders To | Variables |
|---|---|---|
| [`templates/Cargo.toml.tera`](templates/Cargo.toml.tera) | `Cargo.toml` | `skill_name`, `skill_version`, `skill_description`, `crate_authors` |
| [`templates/lib.rs.tera`](templates/lib.rs.tera) | `src/lib.rs` | `skill_name`, `tools_json` (array of tool definitions) |
| [`templates/host.rs.tera`](templates/host.rs.tera) | `src/host.rs` | (none — boilerplate, identical for every skill) |
| [`templates/skill.toml.tera`](templates/skill.toml.tera) | `skill.toml` | `skill_name`, `skill_version`, `skill_description`, `skill_author`, `skill_tags`, `tools`, `capabilities` |

## References

- [Guest ABI walkthrough](references/example-walkthrough.md) — annotated tour of the echo example
- [Host ABI](references/librefang-host-abi.md) — every `host_call` method, params, and capability
- [Capability model](references/capability-model.md) — every `Capability` variant, glob semantics
- [skill.toml reference](references/skill-toml-reference.md) — full SkillManifest schema

## Constraints

- **WASM target**: `wasm32-unknown-unknown` is required. LibreFang's
  `WasmSandbox` uses `wasmtime::Module` + `Linker` (core wasmtime), not the
  Component Model — so `wasm32-wasip1` and `wasm32-wasip2` produce modules
  with WASI-imports the host doesn't satisfy. The host imports are bound by
  module name `librefang`, not by component-model interfaces.
- **No `panic!` across FFI**: panics in `execute` abort the WASM instance
  without returning JSON, surfacing as `SandboxError::Execution` on the host.
  Always `Result`-pipe errors back as `{"error": "..."}` JSON.
- **No `std::net`, `std::fs`, `std::process`**: all I/O must go through
  `host_call`. The host enforces SSRF, path traversal, and capability checks
  that are bypassed by direct WASI calls.
- **Allocator stays put**: `alloc` MUST `Box::leak` the allocation; otherwise
  the host reads freed memory.
