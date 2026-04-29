---
license: MIT
name: actor-model
version: '1.0.0'
description: >
  Tokio-native actor model pattern for Prometheus AGS projects. Implements actors
  as tokio tasks communicating via mpsc channels with a typed message enum, enabling
  safe concurrent state management without shared locks. Used in UAR session management,
  pk-librarian event broadcasting, and parking-lot scheduler design. Use when a
  component needs to own state exclusively while serving concurrent callers.
language: rust
---

# Actor Model — Rust

## Core Pattern

An actor is a tokio task that owns its state exclusively. External code communicates
with the actor through typed messages over an `mpsc` channel. This eliminates the
need for `Arc<Mutex<T>>` on complex stateful objects.

```rust
use tokio::sync::{mpsc, oneshot};

// 1. Define the message enum — each variant carries its reply channel
pub enum LibrarianMessage {
    Compile {
        raw: RawDoc,
        reply: oneshot::Sender<anyhow::Result<WikiEntry>>,
    },
    Lint {
        reply: oneshot::Sender<Vec<LintReport>>,
    },
    Focus {
        topic: String,
        reply: oneshot::Sender<String>,
    },
    Shutdown,
}

// 2. Define the actor handle — the public API
#[derive(Clone)]
pub struct LibrarianHandle {
    tx: mpsc::Sender<LibrarianMessage>,
}

impl LibrarianHandle {
    pub async fn compile(&self, raw: RawDoc) -> anyhow::Result<WikiEntry> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .send(LibrarianMessage::Compile { raw, reply: reply_tx })
            .await
            .map_err(|_| anyhow::anyhow!("librarian actor shut down"))?;
        reply_rx.await.map_err(|_| anyhow::anyhow!("librarian did not reply"))?
    }

    pub async fn shutdown(&self) {
        let _ = self.tx.send(LibrarianMessage::Shutdown).await;
    }
}
```

## Actor Task

```rust
// 3. Define the actor struct — owns its private state
struct LibrarianActor {
    store: MarkdownStore,
    model_router: ModelRouter,
    event_tx: tokio::sync::broadcast::Sender<LibrarianEvent>,
}

impl LibrarianActor {
    async fn run(mut self, mut rx: mpsc::Receiver<LibrarianMessage>) {
        while let Some(msg) = rx.recv().await {
            match msg {
                LibrarianMessage::Compile { raw, reply } => {
                    let result = self.compile_inner(raw).await;
                    let _ = reply.send(result); // ignore if caller dropped
                }
                LibrarianMessage::Lint { reply } => {
                    let reports = self.lint_inner().await;
                    let _ = reply.send(reports);
                }
                LibrarianMessage::Focus { topic, reply } => {
                    let focus = self.focus_inner(&topic).await;
                    let _ = reply.send(focus);
                }
                LibrarianMessage::Shutdown => break,
            }
        }
        tracing::info!("LibrarianActor shutting down");
    }

    async fn compile_inner(&mut self, raw: RawDoc) -> anyhow::Result<WikiEntry> {
        // State mutation is safe — we own `self` exclusively
        let entry = self.model_router.compile(&raw).await?;
        self.store.upsert(&entry).await?;
        let _ = self.event_tx.send(LibrarianEvent::Compiled {
            entry_id: entry.id.clone(),
        });
        Ok(entry)
    }
}
```

## Spawning the Actor

```rust
// 4. Spawn and return the handle
pub fn spawn_librarian(
    store: MarkdownStore,
    model_router: ModelRouter,
    event_tx: tokio::sync::broadcast::Sender<LibrarianEvent>,
    buffer: usize,
) -> LibrarianHandle {
    let (tx, rx) = mpsc::channel(buffer);
    let actor = LibrarianActor { store, model_router, event_tx };
    tokio::spawn(actor.run(rx));
    LibrarianHandle { tx }
}
```

## Supervision Pattern

For actors that must restart on failure, use a supervision loop:

```rust
pub async fn supervise_librarian(
    make_actor: impl Fn() -> LibrarianActor + Send + 'static,
    tx: mpsc::Sender<LibrarianMessage>,
    mut rx: mpsc::Receiver<LibrarianMessage>,
) {
    loop {
        let actor = make_actor();
        let result = tokio::spawn(actor.run_until_panic(/* ... */)).await;
        match result {
            Ok(()) => break, // clean shutdown
            Err(e) => {
                tracing::error!(error = ?e, "actor panicked, restarting in 1s");
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
```

## When to Use Actor vs Arc<RwLock<T>>

| Situation | Use |
|---|---|
| Simple data read/write, no logic | `Arc<RwLock<T>>` |
| Complex stateful logic with many operations | Actor pattern |
| State must be mutated in response to async I/O | Actor pattern |
| Multiple structs sharing the same data | `Arc<RwLock<T>>` |
| Need to serialize access to external resource | Actor pattern |

## Parking-Lot Scheduler as Actor

The UAR's parking-lot scheduler is an actor: it owns the job queue exclusively,
processes submissions via mpsc, and emits `JobEvent`s on a broadcast channel.
The scheduler never shares its queue with `Arc<Mutex<_>>`.

## Forbidden Patterns

- Calling `reply.send()` and then panicking on the error — use `let _ = reply.send(...)`
- Holding a `reply` sender beyond the handler — reply immediately or it leaks
- Putting async calls inside `Arc<Mutex<>>` when actor pattern is cleaner
- Actors that `unwrap()` — all panics escape the task and kill the actor silently
