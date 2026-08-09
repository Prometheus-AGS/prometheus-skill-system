# Architectural Invariants

These invariants are checked by the AI audit loop (phases 6–9) when `--autonomous` mode is active.
Each invariant has an ID, scope, and enforcement rule.

## Actor Domain (`*-actor`, `*-supervisor`)

### ACT-01: No Shared Mutable State
Actors must own their state exclusively. No `Arc<Mutex<T>>` or `RwLock<T>` across actor boundaries.

- **Signal**: `Arc<Mutex` or `Arc<RwLock` crossing a module boundary into an actor
- **Fix**: Pass state via messages; actors own their data exclusively

### ACT-02: Message-Passing Only
Inter-actor communication must use channels or mailboxes. Direct method calls on foreign actor state are forbidden.

- **Signal**: Public mutable methods on actor structs callable from outside the actor module
- **Fix**: Replace with `send(Message)` patterns

### ACT-03: Supervision Tree Integrity
Every actor must be reachable from a root supervisor. Orphaned actors that can panic without recovery are a defect.

- **Signal**: Actor spawned without a `supervisor` or `restart_policy` in scope
- **Fix**: Register with nearest supervisor; define restart policy

## WASM Domain (`*-wasm`, `*-wasmtime`)

### WASM-01: Unsafe Confined to FFI Boundary
`unsafe` code in WASM crates must only appear in FFI shims. Business logic must be safe Rust.

- **Signal**: `unsafe` block outside of `#[no_mangle] extern "C"` or `wasm_bindgen` boundary
- **Fix**: Extract unsafe to a dedicated `ffi.rs` module; wrap in safe API

### WASM-02: No Platform Coupling
WASM crates must compile with `#![cfg(target_arch = "wasm32")]` guards. No direct OS calls.

- **Signal**: `std::fs`, `std::net`, `std::process`, or `std::thread` in WASM crate
- **Fix**: Abstract behind a trait; inject platform impl at compile time

## Async Domain (all async crates)

### ASYNC-01: Cancellation Safety
Futures held across `.await` must be cancellation-safe. Holding locks, open files, or transactions across `.await` is forbidden.

- **Signal**: `MutexGuard`, `RwLockWriteGuard`, or open `File` held across `.await`
- **Fix**: Drop guards before `.await`; use `tokio::sync::Mutex` only when truly necessary

### ASYNC-02: No Blocking in Async Context
CPU-bound or blocking I/O must use `spawn_blocking`. Never call `std::thread::sleep` or synchronous I/O in an async task.

- **Signal**: `std::thread::sleep`, `std::fs::read`, `std::io::stdin().read_line` in async fn
- **Fix**: Wrap in `tokio::task::spawn_blocking`

### ASYNC-03: Explicit Timeouts
Every `.await` on external I/O must have an explicit timeout via `tokio::time::timeout`.

- **Signal**: `client.get(url).await?` without enclosing `timeout(Duration, ...)`
- **Fix**: Wrap: `timeout(Duration::from_secs(10), client.get(url)).await??`

## Allocation Domain (performance-critical crates)

### ALLOC-01: Zero-Copy Preference
Hot paths must prefer `&[u8]` / `Cow<'_, [u8]>` over owned `Vec<u8>` allocations.

- **Signal**: `Vec::clone()` or `to_vec()` in a loop or on a hot path
- **Fix**: Use `Cow`, slices, or arena allocation

### ALLOC-02: No Heap Allocation in Interrupt Context
Crates marked `no_std` or `embedded` must not call `alloc::` in ISR context.

- **Signal**: `Box::new`, `Vec::new`, `String::new` in function marked `#[interrupt]`
- **Fix**: Use fixed-size stack buffers or pre-allocated pools

### ALLOC-03: Arena Lifetime Management
Long-lived allocations must use typed arenas, not unbounded `Vec` growth.

- **Signal**: `Vec` that grows monotonically across request lifetimes without clear eviction
- **Fix**: Use `bumpalo` or `typed-arena`; bound capacity at construction

## Core Invariants (all crates)

### CORE-01: No `unwrap` / `expect` in Production Paths
Panics are not error handling. All `unwrap()` and `expect()` calls must be behind `#[cfg(test)]` or in `main()` startup validation only.

- **Signal**: `unwrap()` or `expect()` outside `#[cfg(test)]` or `fn main()`
- **Fix**: Replace with `?`, `map_err`, or `ok_or_else`

### CORE-02: No Hardcoded Credentials or Paths
No secrets, tokens, or absolute paths in source.

- **Signal**: String literals matching `sk-`, `Bearer `, `/home/`, `/Users/`, `C:\`
- **Fix**: Use `std::env::var` or config injection
