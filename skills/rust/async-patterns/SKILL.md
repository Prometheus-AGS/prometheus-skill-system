---
license: MIT
name: async-patterns
version: '1.0.0'
description: >
  Canonical async Rust patterns for Prometheus AGS projects. Covers tokio task
  spawning, Arc<RwLock<T>> vs parking_lot::Mutex selection, blocking guard prevention,
  broadcast channels, graceful shutdown, and structured concurrency. Use whenever
  writing async code in any Prometheus Fabric crate.
language: rust
---

# Async Patterns — Rust

## Shared State: Arc Rules

Wrap all shared state in `Arc`. Choose the synchronization primitive based on contention:

| Pattern | When to Use |
|---|---|
| `Arc<RwLock<T>>` | Read-heavy, infrequent writes (e.g., config, registry) |
| `Arc<parking_lot::Mutex<T>>` | Write-heavy or hot-path shared state (no `std::sync::Mutex`) |
| `Arc<tokio::sync::RwLock<T>>` | Lock must be held across `.await` points |
| `Arc<AtomicU64>` / atomic types | Single-value counters, flags, metrics |

**Never hold a `parking_lot::Mutex` guard across an `.await` point.** The guard is `!Send`.
Restructure to release the guard before the await:

```rust
// ❌ Wrong — guard held across await
async fn bad(state: Arc<Mutex<State>>) {
    let guard = state.lock();
    some_async_call().await; // compile error: guard is !Send
}

// ✅ Correct — extract value, drop guard, then await
async fn good(state: Arc<Mutex<State>>) {
    let value = { state.lock().get_value() }; // guard dropped here
    some_async_call_with(value).await;
}
```

## Task Spawning

Use `tokio::spawn` for truly independent tasks. Use `tokio::task::spawn_blocking` for
CPU-bound or blocking operations that must not block the async runtime.

```rust
// Background task — independent lifecycle
let handle = tokio::spawn(async move {
    run_background_watcher(rx).await
});

// CPU-heavy work (e.g., serialization, hashing)
let result = tokio::task::spawn_blocking(move || {
    heavy_computation(data)
}).await?;
```

Always join or abort spawned tasks on shutdown. Detached tasks are a resource leak.

## Broadcast Channels

Use `tokio::sync::broadcast` for fan-out event streams (e.g., `LibrarianEvent`, AG-UI
events). Receivers lag if they fall behind — handle `RecvError::Lagged` explicitly.

```rust
use tokio::sync::broadcast;

let (tx, _) = broadcast::channel::<Event>(256);

// Subscriber
let mut rx = tx.subscribe();
tokio::spawn(async move {
    loop {
        match rx.recv().await {
            Ok(event) => handle_event(event),
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("subscriber lagged, skipped {n} events");
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }
});
```

## Graceful Shutdown

Always implement graceful shutdown. Use `tokio::signal` for SIGTERM/SIGINT and a
`CancellationToken` (from `tokio-util`) to propagate shutdown to all tasks.

```rust
use tokio_util::sync::CancellationToken;

async fn shutdown_signal(token: CancellationToken) {
    let ctrl_c = tokio::signal::ctrl_c();
    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("signal handler")
            .recv()
            .await
    };
    tokio::select! {
        _ = ctrl_c => {},
        _ = sigterm => {},
    }
    token.cancel();
}
```

For Axum servers, wire it via `.with_graceful_shutdown()`.

## Select and Timeout

Use `tokio::select!` for racing futures. Always handle the case where the losing
branch has cleanup to do.

```rust
tokio::select! {
    result = operation() => {
        result?
    }
    _ = tokio::time::sleep(Duration::from_secs(30)) => {
        return Err(anyhow::anyhow!("operation timed out after 30s"));
    }
}
```

For retry with backoff, use exponential backoff capped at a max:

```rust
let mut delay = Duration::from_millis(100);
for attempt in 0..3 {
    match operation().await {
        Ok(v) => return Ok(v),
        Err(e) if attempt < 2 => {
            tracing::warn!(attempt, error = %e, "retrying after delay");
            tokio::time::sleep(delay).await;
            delay = (delay * 2).min(Duration::from_secs(5));
        }
        Err(e) => return Err(e),
    }
}
```

## Forbidden Patterns

- `std::thread::sleep` in async code — use `tokio::time::sleep`
- `block_on` inside an async runtime — restructure to be fully async
- Holding `parking_lot::MutexGuard` across `.await` — extract value first
- `tokio::spawn` without storing the handle — always join or abort on shutdown
- Unbounded channels (`channel()` with no backpressure) for high-volume streams
