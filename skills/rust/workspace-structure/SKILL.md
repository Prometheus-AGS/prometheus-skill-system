---
license: MIT
name: workspace-structure
version: '1.0.0'
description: >
  Canonical Rust workspace layout for Prometheus AGS multi-crate projects. Enforces
  resolver=2, workspace-level dependency versions, domain-driven crate separation
  (*-core / *-store / *-librarian / *-mcp / *-cli), feature flag discipline, and
  release profile settings. Use when scaffolding any new Prometheus Rust workspace.
language: rust
---

# Workspace Structure — Rust

## Canonical Layout

Every Prometheus AGS Rust project follows domain-driven workspace decomposition.
Crates are separated by concern, not by layer.

```
my-project/
├── Cargo.toml              ← workspace manifest
├── Cargo.lock              ← always committed (application); gitignored (library-only)
├── CLAUDE.md               ← agent development guide
├── README.md
├── .env.example
├── .gitignore
└── crates/
    ├── my-core/            ← domain types: structs, enums, traits — no I/O
    │   ├── Cargo.toml
    │   └── src/lib.rs
    ├── my-store/           ← persistence: flat-file, SurrealDB, PostgreSQL
    ├── my-librarian/       ← orchestration: business logic, LLM calls
    ├── my-mcp/             ← Axum MCP server (JSON-RPC 2.0 + SSE)
    ├── my-uar/             ← UAR integration (optional)
    └── my-cli/             ← binary entry point
        ├── Cargo.toml
        └── src/main.rs
```

**Rule**: `*-core` must never depend on any other workspace crate. It is the root of
the dependency graph. All other crates may depend on `*-core`.

## Workspace Cargo.toml

```toml
[workspace]
members  = ["crates/my-core", "crates/my-store", "crates/my-librarian", "crates/my-mcp", "crates/my-cli"]
resolver = "2"  # Always resolver=2 for feature unification correctness

[workspace.package]
version    = "0.1.0"
edition    = "2021"
authors    = ["Travis James <travis@prometheusags.ai>"]
license    = "MIT"

[workspace.dependencies]
# Pin ALL external deps at the workspace level. Crate Cargo.toml only uses
# { workspace = true } — no version strings in crate manifests.
tokio        = { version = "1", features = ["full"] }
axum         = { version = "0.8", features = ["macros"] }
serde        = { version = "1", features = ["derive"] }
serde_json   = "1"
anyhow       = "1"
thiserror    = "2"
tracing      = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
uuid         = { version = "1", features = ["v4", "serde"] }
chrono       = { version = "0.4", features = ["serde"] }

# Internal crates — always path deps
my-core      = { path = "crates/my-core" }
my-store     = { path = "crates/my-store" }
my-librarian = { path = "crates/my-librarian" }
my-mcp       = { path = "crates/my-mcp" }

[profile.release]
strip         = true
lto           = true
codegen-units = 1
```

## Crate Cargo.toml Pattern

```toml
[package]
name        = "my-store"
# No version/edition/authors — all inherited from workspace
version.workspace   = true
edition.workspace   = true
authors.workspace   = true
license.workspace   = true

[dependencies]
my-core     = { workspace = true }
tokio       = { workspace = true }
serde       = { workspace = true }
serde_json  = { workspace = true }
anyhow      = { workspace = true }
thiserror   = { workspace = true }
```

**Never** put version strings in a crate's `[dependencies]` when the dep is declared
at workspace level. Use `{ workspace = true }` exclusively.

## Feature Flags

Use feature flags only for genuinely optional capabilities. Don't use them to split
code that always runs together — that's what crate separation is for.

```toml
[features]
default    = []
jemalloc   = ["dep:tikv-jemallocator"]  # Optional allocator
mcp        = ["dep:axum", "dep:tower"]  # Enable MCP server in library crates

[dependencies]
tikv-jemallocator = { workspace = true, optional = true }
axum              = { workspace = true, optional = true }
```

Apply `jemalloc` only in the binary entry point (`*-cli`), never in library crates.

## Dependency Graph Rules

```
my-core        ← no deps on other workspace crates
my-store       ← depends on: my-core
my-librarian   ← depends on: my-core, my-store
my-mcp         ← depends on: my-core, my-librarian
my-cli         ← depends on: my-core, my-librarian, my-mcp
```

Circular dependencies are a compile error. If a circular dep is tempting, the design
has a layering problem — introduce a trait in `*-core` and implement it in a separate crate.

## Git and Cargo.lock

- **Application workspaces** (has a binary): commit `Cargo.lock`
- **Library-only workspaces**: gitignore `Cargo.lock`, but check it in for reproducible CI

## Forbidden Patterns

- Version strings in crate `[dependencies]` when dep is declared at workspace level
- `resolver = "1"` — always use `resolver = "2"`
- Circular workspace crate dependencies
- `*-core` depending on any other workspace crate
- Feature flags that are always enabled (just add the dep unconditionally)
