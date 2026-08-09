# Example Walkthrough — Echo Skill

The complete echo skill source lives at `references/example-echo/`. This doc
walks through the four files line-by-line so an implementing agent can use it
as the reference shape when generating new skills.

## Layout

```
example-echo/
├── Cargo.toml         ← cdylib crate, wasm32-unknown-unknown-friendly profile
├── skill.toml         ← LibreFang manifest, type = "wasm"
└── src/
    ├── lib.rs         ← Guest ABI + tool dispatch
    └── host.rs        ← typed wrappers around host_call/host_log
```

## Cargo.toml

```toml
[package]
name = "echo"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]   # produces a single .wasm; do not change

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

The release profile is aggressive on size because every byte gets uploaded
to bossfang and reloaded on each agent restart. `panic = "abort"` is correct
for WASM since panics can't unwind across the host boundary anyway.

## skill.toml

```toml
[skill]
name = "echo"
version = "0.1.0"
description = "Echo a JSON payload back to the caller."
author = "prometheus-ags"
tags = ["wasm", "example"]

[runtime]
type = "wasm"
entry = "echo.wasm"

[[tools]]
name = "echo"
description = "Returns the input unchanged."
input_schema = { type = "object", properties = {
  message = { type = "string" }
}, required = ["message"] }

[requirements]
capabilities = []   # echo needs nothing — pure data transformation
```

Echo declares no capabilities because it never calls the host. A skill that
fetched a URL would add `NetConnect("...")`; a skill that read a file would
add `FileRead("...")`. See [`capability-model.md`](capability-model.md).

## src/lib.rs

```rust
mod host;

use serde::{Deserialize};
use serde_json::{json, Value};

#[derive(Deserialize)]
struct Invocation {
    tool: String,
    #[serde(default)]
    input: Value,
}

fn dispatch(inv: Invocation) -> Result<Value, String> {
    match inv.tool.as_str() {
        "echo" => tool_echo(inv.input),
        other => Err(format!("Unknown tool: {other}")),
    }
}

fn tool_echo(input: Value) -> Result<Value, String> {
    host::log_info("echo: invoked");
    Ok(json!({ "echoed": input }))
}

// === Guest ABI exports ===

#[no_mangle]
pub unsafe extern "C" fn alloc(size: i32) -> i32 {
    let mut buf: Vec<u8> = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr as i32
}

#[no_mangle]
pub unsafe extern "C" fn execute(ptr: i32, len: i32) -> i64 {
    let bytes = std::slice::from_raw_parts(ptr as *const u8, len as usize);
    let result = match serde_json::from_slice::<Invocation>(bytes) {
        Ok(inv) => match dispatch(inv) {
            Ok(v) => v,
            Err(e) => json!({ "error": e }),
        },
        Err(e) => json!({ "error": format!("Invalid invocation JSON: {e}") }),
    };
    let out = serde_json::to_vec(&result).unwrap_or_else(|_| b"{}".to_vec());
    let out_len = out.len() as i32;
    let out_ptr = alloc(out_len);
    std::ptr::copy_nonoverlapping(out.as_ptr(), out_ptr as *mut u8, out.len());
    ((out_ptr as i64) << 32) | (out_len as i64 & 0xFFFF_FFFF)
}
```

Key invariants:

1. `alloc` **must** `mem::forget` (or `Box::leak`) the allocation. Returning
   `buf.as_mut_ptr()` after `buf` drops would give the host a dangling pointer
   the next time it overwrites the freed region.
2. `execute` **must not** panic. A panic aborts the WASM instance without
   producing output bytes; the host gets `SandboxError::Execution` and the
   user sees a generic failure. Always pipe errors back as JSON.
3. The packed return is `(ptr << 32) | len`. Both halves are `i32` because
   WASM's pointer/size types are 32-bit; sign-extension is not an issue
   because `alloc` only returns positive pointers in practice.

## src/host.rs

The host-bridge module is identical in every skill — copy from
`templates/src/host.rs.tera`. Skill code only ever sees the typed
wrappers (`host::time_now()`, `host::fs_read(path)`, etc.).

## Build & Validate

```bash
cd example-echo
cargo build --target wasm32-unknown-unknown --release

# Validate the ABI
bash ../../scripts/validate-wasm-abi.sh \
  target/wasm32-unknown-unknown/release/echo.wasm

# Output: ✅ memory, alloc, execute exports present; librefang imports clean
```

## Round-Trip via LibreFang

Once `librefang start` is up:

```bash
# Package
zip echo-skill.zip echo.wasm skill.toml README.md

# Install
curl -X POST http://localhost:4545/skills/install \
  -H "Content-Type: application/zip" \
  --data-binary @echo-skill.zip

# Reload
curl -X POST http://localhost:4545/skills/reload

# Verify
curl http://localhost:4545/skills/echo
# → { "name": "echo", "runtime": { "type": "wasm", ... }, ... }
```

From an agent, the LLM can now call the `echo` tool from the `echo` skill
and receive the input back wrapped in `{ "echoed": ... }`.
