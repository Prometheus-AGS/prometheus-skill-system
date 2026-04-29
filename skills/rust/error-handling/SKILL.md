---
license: MIT
name: error-handling
version: '1.0.0'
description: >
  Canonical Rust error handling for Prometheus AGS projects. Enforces the thiserror/anyhow
  boundary: thiserror for library crate errors (typed, composable), anyhow for application
  code (ergonomic, context-rich). Includes #[cold] error path annotation, no unwrap/expect
  in non-test code, and structured error propagation via the ? operator.
language: rust
---

# Error Handling — Rust

## The Library / Application Boundary

**Library crates** (`*-core`, `*-store`, `*-librarian`, etc.) define typed errors with `thiserror`.
**Application crates** (`*-cli`, `*-mcp`) use `anyhow::Result` for ergonomic propagation.

```rust
// In a library crate (forge-core, pk-store, etc.)
#[derive(thiserror::Error, Debug)]
pub enum StoreError {
    #[error("entry not found: {0}")]
    NotFound(String),

    #[error("serialization failed")]
    Serialize(#[from] serde_json::Error),

    #[error("I/O error")]
    Io(#[from] std::io::Error),
}

// In an application crate (forge-cli, pk-cherry, etc.)
use anyhow::{Context, Result};

fn load_config(path: &Path) -> Result<Config> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading config from {}", path.display()))?;
    toml::from_str(&raw).context("parsing config")
}
```

## Error Path Annotation with `#[cold]`

Mark error paths `#[cold]` and `#[inline(never)]` to push them out of the hot path
and improve instruction cache behavior. This is especially important in server code
where errors are rare but the hot path runs millions of times.

```rust
#[cold]
#[inline(never)]
fn report_store_error(e: &StoreError) {
    tracing::error!(error = %e, "store operation failed");
}

// In hot path:
match store.get(id) {
    Ok(entry) => process(entry),
    Err(e) => { report_store_error(&e); return Err(e.into()); }
}
```

## The `?` Operator and Context

Always use `?` for error propagation. Never use `unwrap()` or `expect()` outside of:
- `#[test]` functions
- `const` evaluation where panic is intentional
- Truly invariant conditions documented with `// SAFETY:` comments

When using `?`, add context to make error messages actionable:

```rust
// Bad — "No such file or directory" with no context
let content = std::fs::read_to_string(path)?;

// Good — "reading wiki entry from /path/to/file.md: No such file or directory"
let content = std::fs::read_to_string(path)
    .with_context(|| format!("reading wiki entry from {}", path.display()))?;
```

## Converting Between Error Types

Use `#[from]` on `thiserror` variants for automatic `From` impl. Use `.map_err()` when
the conversion is non-trivial or context-dependent.

```rust
#[derive(thiserror::Error, Debug)]
pub enum LibrarianError {
    // Automatic From<StoreError> via #[from]
    #[error("store error")]
    Store(#[from] StoreError),

    // Manual conversion with context
    #[error("LLM API error: {message}")]
    LlmApi { message: String },
}

// Manual conversion
async fn compile(raw: RawDoc) -> Result<WikiEntry, LibrarianError> {
    let response = call_llm(&raw.content)
        .await
        .map_err(|e| LibrarianError::LlmApi { message: e.to_string() })?;
    // ...
}
```

## Error Logging Pattern

Log errors at the boundary where they are handled — not where they originate.
Use structured fields so logs are machine-parseable.

```rust
match operation().await {
    Ok(result) => { /* happy path */ }
    Err(e) => {
        tracing::error!(
            error = %e,
            error.kind = std::any::type_name_of_val(&e),
            "operation failed"
        );
        return Err(e.into());
    }
}
```

## Forbidden Patterns

- `unwrap()` in non-test production code — use `?` or `match`
- `expect("message")` in non-test production code — add `.with_context(|| ...)` instead
- `panic!()` in library crates — return an error
- Silent error swallowing: `let _ = risky_operation();` — log or propagate
- `eprintln!` for errors — use `tracing::error!`
