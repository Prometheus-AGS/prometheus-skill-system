// TJ-ARCH-MOB-001 compliant
//! Rust↔Swift FFI for the iOS MLX bridge.
//!
//! The Swift side registers a C vtable into Rust at app launch — the inverse of
//! Android's `set_java_vm`. Everything is `Option<vtable>` at runtime, so host
//! and simulator builds (where no bridge registers) degrade to a clean "not
//! available" rather than an undefined-symbol link error.

use std::ffi::{c_char, c_void, CStr, CString};
use std::path::Path;
use std::sync::OnceLock;

use gen_ui_types::error::{CoreError, CoreResult};

use crate::mlx::{MlxBridge, MlxBridgeEvent};

/// C function table the Swift bridge fills in. All pointers are plain
/// C-convention functions, so the table is trivially `Send + Sync`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MlxBridgeVTable {
    /// Blocking load. `model_dir`/`config_json` are NUL-terminated UTF-8. On
    /// failure, `on_error(ctx, message)` is invoked once and a non-zero code is
    /// returned; `message` is valid only for that call.
    pub load: extern "C" fn(
        model_dir: *const c_char,
        config_json: *const c_char,
        ctx: *mut c_void,
        on_error: extern "C" fn(*mut c_void, *const c_char),
    ) -> i32,
    /// Blocking generate. `on_event(ctx, json)` is invoked per streamed event;
    /// `json` is a NUL-terminated UTF-8 event valid only for that call.
    pub generate: extern "C" fn(
        request_json: *const c_char,
        ctx: *mut c_void,
        on_event: extern "C" fn(*mut c_void, *const c_char),
    ) -> i32,
    pub cancel: extern "C" fn(),
    pub close: extern "C" fn(),
}

// SAFETY: the vtable is a table of C function pointers with no interior
// mutability; sharing it across threads is sound. The Swift implementations
// behind the pointers are responsible for their own thread-safety, which the
// bridge enforces by serializing generation.
unsafe impl Send for MlxBridgeVTable {}
unsafe impl Sync for MlxBridgeVTable {}

static MLX_BRIDGE: OnceLock<MlxBridgeVTable> = OnceLock::new();

/// Register the Swift bridge. Called once from `AppDelegate` at launch, before
/// Flutter boot triggers model loading. Idempotent (later calls are ignored).
///
/// # Safety
/// `vtable` must point to a valid, fully-initialized `MlxBridgeVTable` whose
/// function pointers remain valid for the process lifetime.
#[no_mangle]
pub unsafe extern "C" fn __APP_NAME___mlx_register_bridge(vtable: *const MlxBridgeVTable) -> i32 {
    if vtable.is_null() {
        return -1;
    }
    // SAFETY: caller guarantees `vtable` is a valid pointer per the contract.
    let table = unsafe { *vtable };
    match MLX_BRIDGE.set(table) {
        Ok(()) => 0,
        // Already registered — not an error, just a no-op.
        Err(_) => 1,
    }
}

pub(crate) fn bridge_available() -> bool {
    MLX_BRIDGE.get().is_some()
}

/// Collected error message from the load callback.
struct LoadErrorSink {
    message: Option<String>,
}

extern "C" fn load_error_trampoline(ctx: *mut c_void, message: *const c_char) {
    if ctx.is_null() || message.is_null() {
        return;
    }
    // SAFETY: ctx is the Box<LoadErrorSink> we passed to `load`; message is a
    // valid NUL-terminated C string for the duration of this call.
    let sink = unsafe { &mut *(ctx as *mut LoadErrorSink) };
    let text = unsafe { CStr::from_ptr(message) }
        .to_string_lossy()
        .into_owned();
    sink.message = Some(text);
}

/// Trampoline state for `generate`: the Rust closure receiving parsed events.
struct GenerateSink<'a> {
    on_event: &'a mut dyn FnMut(MlxBridgeEvent),
}

extern "C" fn generate_event_trampoline(ctx: *mut c_void, json: *const c_char) {
    if ctx.is_null() || json.is_null() {
        return;
    }
    // Panicking across the C boundary is UB — contain it.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // SAFETY: ctx is the GenerateSink we passed to `generate`; json is a
        // valid NUL-terminated C string for the duration of this call.
        let sink = unsafe { &mut *(ctx as *mut GenerateSink) };
        let raw = unsafe { CStr::from_ptr(json) }.to_string_lossy();
        match serde_json::from_str::<MlxBridgeEvent>(&raw) {
            Ok(event) => (sink.on_event)(event),
            Err(error) => (sink.on_event)(MlxBridgeEvent::Error {
                message: format!("MLX bridge sent an unparseable event: {error}"),
            }),
        }
    }));
}

/// The production `MlxBridge` backed by the registered Swift vtable.
pub(crate) struct VtableBridge;

impl MlxBridge for VtableBridge {
    fn load(&self, model_dir: &Path, config_json: &str) -> CoreResult<()> {
        let vtable = MLX_BRIDGE
            .get()
            .ok_or_else(|| CoreError::Terminal("MLX bridge is not registered".into()))?;
        let dir = CString::new(model_dir.to_string_lossy().as_bytes())
            .map_err(|error| CoreError::Terminal(format!("MLX model dir C string: {error}")))?;
        let config = CString::new(config_json)
            .map_err(|error| CoreError::Terminal(format!("MLX config C string: {error}")))?;
        let mut sink = LoadErrorSink { message: None };
        let code = (vtable.load)(
            dir.as_ptr(),
            config.as_ptr(),
            &mut sink as *mut LoadErrorSink as *mut c_void,
            load_error_trampoline,
        );
        if code == 0 {
            Ok(())
        } else {
            Err(CoreError::Terminal(sink.message.unwrap_or_else(|| {
                format!("MLX bridge load failed with code {code}")
            })))
        }
    }

    fn generate(
        &self,
        request_json: &str,
        on_event: &mut dyn FnMut(MlxBridgeEvent),
    ) -> CoreResult<()> {
        let vtable = MLX_BRIDGE
            .get()
            .ok_or_else(|| CoreError::Terminal("MLX bridge is not registered".into()))?;
        let request = CString::new(request_json)
            .map_err(|error| CoreError::Terminal(format!("MLX request C string: {error}")))?;
        let mut sink = GenerateSink { on_event };
        let code = (vtable.generate)(
            request.as_ptr(),
            &mut sink as *mut GenerateSink as *mut c_void,
            generate_event_trampoline,
        );
        if code == 0 {
            Ok(())
        } else {
            Err(CoreError::Terminal(format!(
                "MLX bridge generate failed with code {code}"
            )))
        }
    }

    fn cancel(&self) {
        if let Some(vtable) = MLX_BRIDGE.get() {
            (vtable.cancel)();
        }
    }

    fn close(&self) {
        if let Some(vtable) = MLX_BRIDGE.get() {
            (vtable.close)();
        }
    }

    fn available(&self) -> bool {
        bridge_available()
    }
}
