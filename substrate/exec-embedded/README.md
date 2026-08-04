# `prometheus-exec-embedded`

Estate-free Tier W execution for mobile and desktop applications. The crate
provides one process-owned API and a string/byte adapter for UI transports:

- `run` / `run_json`
- `status` / `status_json`
- cursor-based ordered `events` / `events_json`
- `receipt` / `receipt_json`
- content-addressed `artifact`
- transport-free `verify` / `verify_json`

The crate never creates a Tokio runtime. The embedding app owns one runtime;
blocking Wasmtime, ledger, and CAS work runs through `spawn_blocking`.

## Host key boundary

Construct `EmbeddedExecutionApi` in trusted Rust host code after obtaining the
device signing key from the platform secure-key provider. UI and FFI adapters
receive an already-configured API. Their methods accept no private key and can
export only `EmbeddedPublicKey` verification material.

## Tauri integration

`EmbeddedExecutionAdapter` is Tauri-compatible without importing Tauri into the
execution substrate. Manage one adapter as application state and delegate thin
commands to its async methods:

```rust,ignore
#[tauri::command]
async fn exec_run(
    state: tauri::State<'_, EmbeddedExecutionAdapter>,
    request_json: String,
    component: Vec<u8>,
    inputs: BTreeMap<String, Vec<u8>>,
) -> Result<String, String> {
    state
        .run_json(request_json, component, inputs)
        .await
        .map_err(|error| error.to_string())
}
```

Use the `standalone` feature for desktop Cranelift builds and the `mobile`
feature (with default features disabled) for Pulley-only iOS/Android builds.
