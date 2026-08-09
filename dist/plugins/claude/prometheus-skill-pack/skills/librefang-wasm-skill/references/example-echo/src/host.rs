//! Safe wrappers around LibreFang's host imports.
//!
//! Each wrapper performs JSON serialization on the way out, deserialization
//! on the way back, and surfaces the host's `{"error": "..."}` payload as a
//! `Result::Err`. Skill code only ever sees typed Rust.

use serde_json::{json, Value};

#[link(wasm_import_module = "librefang")]
extern "C" {
    fn host_call(req_ptr: i32, req_len: i32) -> i64;
    fn host_log(level: i32, msg_ptr: i32, msg_len: i32);
}

fn call(method: &str, params: Value) -> Result<Value, String> {
    let req = json!({ "method": method, "params": params });
    let req_bytes = serde_json::to_vec(&req).map_err(|e| e.to_string())?;
    let packed = unsafe { host_call(req_bytes.as_ptr() as i32, req_bytes.len() as i32) };
    let out_ptr = (packed >> 32) as i32;
    let out_len = (packed & 0xFFFF_FFFF) as i32;
    let resp_bytes =
        unsafe { std::slice::from_raw_parts(out_ptr as *const u8, out_len as usize) };
    let resp: Value = serde_json::from_slice(resp_bytes).map_err(|e| e.to_string())?;
    if let Some(err) = resp.get("error").and_then(|e| e.as_str()) {
        return Err(err.to_string());
    }
    resp.get("ok")
        .cloned()
        .ok_or_else(|| "Host response missing 'ok' field".into())
}

fn log(level: i32, msg: &str) {
    let bytes = msg.as_bytes();
    unsafe { host_log(level, bytes.as_ptr() as i32, bytes.len() as i32) };
}

#[allow(dead_code)]
pub fn log_info(msg: &str) { log(2, msg) }

// Echo doesn't need any other host calls. A real skill would re-export the
// full set documented in references/librefang-host-abi.md.
#[allow(dead_code)]
pub fn time_now() -> Result<u64, String> {
    let v = call("time_now", json!({}))?;
    v.as_u64().ok_or_else(|| "time_now: expected u64".into())
}
