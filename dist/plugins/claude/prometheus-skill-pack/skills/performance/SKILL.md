---
license: MIT
name: performance
version: '1.0.0'
description: >
  Production-grade Rust performance primitives for Prometheus AGS projects. Covers
  jemalloc global allocator, #[cold]/#[inline(never)] for error paths, MaybeUninit
  for zero-cost initialization, std::mem::take for ownership without clone,
  Arc placement discipline, parking_lot over std::sync, and SIMD-aware buffer
  patterns. Apply when writing hot paths, server binaries, or inference-adjacent code.
language: rust
metadata:
  tags: [rust, patterns]
---

# Performance Primitives — Rust

## Global Allocator: jemalloc

Apply `tikv-jemallocator` at every binary entry point (`*-cli`, `*-cherry`, `*-mcp` binaries).
Never apply in library crates. The allocator reduces fragmentation in long-running Tokio
servers and improves inference-adjacent workloads significantly.

```rust
// In crates/my-cli/src/main.rs (binary only)
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> { /* ... */ }
```

In `Cargo.toml` for the binary crate:
```toml
[dependencies]
tikv-jemallocator = { workspace = true }
```

## Error Path Isolation: `#[cold]` + `#[inline(never)]`

Move infrequent (error) code off the hot path. The processor's branch predictor and
instruction cache work better when errors never appear in the hot path's instruction window.

```rust
// Error handling extracted from hot path
#[cold]
#[inline(never)]
fn handle_kv_cache_miss(id: &SessionId) -> CacheEntry {
    tracing::debug!(%id, "KV cache miss — allocating new entry");
    CacheEntry::default()
}

// Hot path — branch predictor learns this is rarely taken
fn get_kv_entry(cache: &KvCache, id: &SessionId) -> CacheEntry {
    if let Some(entry) = cache.get(id) {
        return entry; // hot path: almost always taken
    }
    handle_kv_cache_miss(id) // cold: pushed out of I-cache
}
```

Apply `#[cold]` to: error constructors, logging helpers, fallback branches, retry paths.

## Zero-Cost Initialization: `MaybeUninit`

Use `MaybeUninit` for stack-allocated buffers that will be fully initialized before use.
Avoids the zeroing cost of `[0u8; N]` on hot paths (e.g., SIMD buffer setup in TurboQuant).

```rust
use std::mem::MaybeUninit;

// Stack-allocate a 256-byte FWHT buffer without zeroing
let mut buf: [MaybeUninit<f32>; 256] = MaybeUninit::uninit_array();
// SAFETY: We write every element before reading
for (i, slot) in buf.iter_mut().enumerate() {
    slot.write(input[i]);
}
// SAFETY: All elements written above
let buf = unsafe { MaybeUninit::array_assume_init(buf) };
```

Only use with a `// SAFETY:` comment documenting the invariant.

## Ownership Without Clone: `std::mem::take`

Avoid cloning when you need to move a field out of a `&mut self`. Use `std::mem::take`
to replace with the type's `Default` and take ownership of the original value.

```rust
// Avoids Vec::clone() on a potentially large job queue
fn drain_queue(&mut self) -> Vec<Job> {
    std::mem::take(&mut self.pending_jobs) // O(1) — just swaps the pointer
}

// Avoids String::clone() when building a response
fn take_buffer(&mut self) -> String {
    std::mem::take(&mut self.output_buffer)
}
```

## Arc Placement Discipline

Only clone `Arc` at the call site where the clone will be moved into a new owner.
Never clone `Arc` "just in case" or to avoid borrow checker friction.

```rust
// ✅ Correct — clone exactly where needed
let state_for_task = Arc::clone(&state);
tokio::spawn(async move { task(state_for_task).await });

// ❌ Wrong — cloning before knowing if needed
let state2 = Arc::clone(&state); // unnecessary if we only use it once
if condition {
    use_state(&state);
} else {
    drop(state2); // wasted allocation
}
```

## parking_lot over std::sync

`parking_lot::Mutex` and `parking_lot::RwLock` are always preferred over `std::sync`
equivalents. They are faster, smaller, and do not poison on panic.

```rust
// ❌ Forbidden in Prometheus projects
use std::sync::Mutex;

// ✅ Required
use parking_lot::Mutex;
use parking_lot::RwLock;
```

In hot paths where the lock is briefly held and contention is low, `parking_lot::Mutex`
outperforms `tokio::sync::Mutex` because it does not yield to the async executor.

## Slice Patterns Over Index Arithmetic

Prefer slice methods over manual index bounds checks. Use `split_at` and `chunks_exact`
to eliminate bounds check overhead in SIMD-adjacent code.

```rust
// Process 8 f32s at a time — compiler can auto-vectorize
fn apply_fwht(data: &mut [f32]) {
    assert!(data.len() % 8 == 0, "length must be multiple of 8");
    for chunk in data.chunks_exact_mut(8) {
        // chunk.len() == 8 guaranteed — no bounds checks in this loop
        let (a, b) = chunk.split_at_mut(4);
        // process a and b...
    }
}
```

## Release Profile

Always include in workspace `Cargo.toml`:

```toml
[profile.release]
strip         = true   # Remove debug symbols from binary
lto           = true   # Link-time optimization across crates
codegen-units = 1      # Single codegen unit for maximum optimization
```

For `cargo build --release`, this produces the smallest, fastest binary.
Do not use `codegen-units = 1` in development — it slows incremental compilation.

## Forbidden Patterns

- `#[allow(unused)]` on performance-critical code — all code paths should be used
- `Vec::clone()` inside a hot loop — use `drain()`, `take()`, or arena allocation
- `String::from(format!(...))` — just use `format!`
- `collect::<Vec<_>>()` on an iterator you immediately consume — iterate directly
- `std::sync::Mutex` — use `parking_lot::Mutex`
