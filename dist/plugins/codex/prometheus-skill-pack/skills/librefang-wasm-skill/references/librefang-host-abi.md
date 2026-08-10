# LibreFang Host-Call ABI

Authoritative source: `librefang/crates/librefang-runtime-wasm/src/host_functions.rs`
in the commit-pinned LibreFang source checkout.

Every method below is invoked from a WASM guest via the `host_call` import.
Request shape:

```json
{ "method": "<name>", "params": { ... } }
```

Response shape:

```json
{ "ok": <value> }     // success
{ "error": "<msg>" }  // failure (capability denied, IO error, etc.)
```

The host packs the response pointer into an `i64` per the Guest ABI. The
generated `src/host.rs` unpacks it transparently — skill code only sees
typed `Result<T, String>`.

## Method Table

| Method | Params | `ok` value | Required Capability | Notes |
|---|---|---|---|---|
| `time_now` | `{}` | `u64` (unix seconds) | (none) | Always allowed |
| `fs_read` | `{"path": "<abs path>"}` | `string` (file contents) | `FileRead("<glob>")` | `..` rejected, symlinks resolved |
| `fs_write` | `{"path": "...", "contents": "..."}` | `null` | `FileWrite("<glob>")` | Parent canonicalized; filename traversal rejected |
| `fs_list` | `{"path": "<abs dir>"}` | `array<string>` | `FileRead("<glob>")` | Returns just file names, not paths |
| `net_fetch` | `{"url": "...", "method": "GET", "body": ""}` | `string` (response body) | `NetConnect("<host:port>")` | SSRF-protected; private/loopback IPs blocked |
| `shell_exec` | `{"cmd": "...", "args": [...]}` | `string` (stdout) | `ShellExec("<cmd>")` | Argument validation; no shell expansion |
| `env_read` | `{"var": "<NAME>"}` | `string \| null` | `EnvRead("<var>")` | Returns null when unset |
| `kv_get` | `{"key": "..."}` | `string \| null` | `MemoryRead("<key>")` | Backed by kernel `memory_recall` |
| `kv_set` | `{"key": "...", "value": "..."}` | `null` | `MemoryWrite("<key>")` | Backed by kernel `memory_persist` |
| `agent_send` | `{"agent_id": "...", "payload": <Value>}` | `<Value>` (response) | `AgentMessage("<id>")` | Synchronous; awaits target agent reply |
| `agent_spawn` | `{"spec": <AgentSpec>}` | `<spawned-agent-record>` | `AgentSpawn` | `spec` matches kernel's `AgentSpec` |

## Error Modes

| Error | Cause | Mitigation |
|---|---|---|
| `Missing '<param>' parameter` | Required param absent | Validate JSON before calling |
| `Capability denied: <Cap>` | Skill lacks the capability | Add to `[requirements].capabilities` |
| `Path traversal denied: ...` | `path` contains `..` or resolves outside scope | Use absolute paths only |
| `SSRF target rejected: ...` | URL resolves to private/loopback IP | Use only public URLs or `127.0.0.1` if explicitly allowed |
| `No kernel handle available` | Skill running in a sandbox without a kernel attachment | Only meaningful for tests; not user-facing |
| `Unknown host method: <name>` | Typo or future method | Confirm method exists in `host_functions.rs` |

## Logging

`host_log(level, msg_ptr, msg_len)` is fire-and-forget. Levels:

| Level | Constant | Use |
|---|---|---|
| 0 | trace | Per-operation detail; usually suppressed |
| 1 | debug | Algorithmic state; visible at `RUST_LOG=debug` |
| 2 | info | High-level lifecycle events |
| 3 | warn | Recoverable problems |
| 4 | error | Failures that produce a user-visible error |

Messages longer than **4096 bytes** are truncated by the host and tagged with
the truncated byte count. Don't log raw user input or large JSON blobs.

## Performance

- Each `host_call` is a synchronous trip through the wasmtime FFI boundary
  with JSON serialize on both sides. ~10–50 µs overhead per call on M-class
  hardware. Batch where possible.
- `net_fetch` blocks the guest until the HTTP response completes; epoch-based
  timeouts apply (default 30s — see `SandboxConfig::timeout_secs`).
- `kv_*` calls bottom out in the kernel's distributed memory layer
  (surreal-memory by default); expect 1–10 ms latency per call.
- `time_now` is a single syscall on the host side; cheap.

## Examples

```rust
// Read a config file the kernel made available.
let content = host::fs_read("/etc/myskill/config.toml")?;

// Fetch a public URL (capability: NetConnect("api.example.com:443"))
let body = host::net_fetch("https://api.example.com/v1/status", "GET", "")?;

// Persist a value across runs.
host::kv_set("last_run_at", &host::time_now()?.to_string())?;

// Send a message to another agent (capability: AgentMessage("research-agent"))
let resp = host::agent_send("research-agent",
    serde_json::json!({"query": "Q4 earnings"}))?;
```
