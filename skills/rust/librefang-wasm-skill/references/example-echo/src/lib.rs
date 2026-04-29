//! Echo — the canonical LibreFang WASM skill example.
//!
//! Builds with: cargo build --target wasm32-unknown-unknown --release
//! Validates with: bash ../../scripts/validate-wasm-abi.sh \
//!                   target/wasm32-unknown-unknown/release/echo.wasm

#![allow(clippy::missing_safety_doc)]

mod host;

use serde::Deserialize;
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
