# CLAUDE.md — forge-rs

Development guide for AI agents working on the forge-rs codebase.

## Architecture overview

See `README.md` for the full architecture. Key crates:
- `forge-core` — domain types only, no I/O
- `forge-skills` — SkillRegistry, Tera templates, dependency resolution
- `forge-enricher` — enrichment pipeline (reads → resolves → renders → writes)
- `forge-reflect` — reflection + pk ingest (Karpathy loop)
- `forge-mcp` — Axum MCP server (JSON-RPC 2.0 over HTTP)
- `forge-cli` — `forge` binary entry point

## Build commands

```bash
cargo build --workspace             # build all crates
cargo build --release -p forge-cli  # build forge binary
cargo test --workspace              # run all tests
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

## Key constraints (from Rust constitution)

- No `unwrap()` or `expect()` in non-test code — use `?` and `anyhow::Context`
- No `std::sync::Mutex` — use `parking_lot::Mutex` or `Arc<RwLock<T>>`
- No blocking calls in async context
- `tikv-jemallocator` at binary entry points only (forge-cli)
- `thiserror` for library errors in forge-core, `anyhow` for application code

## Testing

```bash
# Unit tests (no I/O)
cargo test -p forge-core
cargo test -p forge-skills

# Integration tests (require pk CLI in PATH)
cargo test -p forge-enricher -- --ignored
cargo test -p forge-reflect  -- --ignored
```

## Adding a new language

1. Create `constitution-templates/<language>.toml`
2. Add the `Language` variant to `forge-core/src/lib.rs`
3. Add detection logic to `forge-enricher/src/lib.rs → detect_language()`
4. Add `include_str!` branch in `forge-cli/src/main.rs → scaffold_constitution()`
5. Create `skills/<language>/` in the skill pack with at least one skill

## Adding a new MCP tool

1. Add the tool definition to the `tools/list` response in `forge-mcp/src/lib.rs`
2. Add the `tools/call` match arm
3. Wire to the appropriate crate operation
4. Document in `README.md`

## Skill manifest format (`skill.toml`)

```toml
name        = "axum-patterns"
language    = "rust"
description = "Axum 0.8 router, middleware, and extractor patterns"
version     = "1.0.0"

[[templates]]
path               = "router.rs.tera"
output_description = "Axum router scaffold with state injection"

[[templates]]
path               = "middleware.rs.tera"
output_description = "Tower middleware pattern for Axum"

[[triggers]]
type     = "AlwaysForLanguage"
language = "rust"

[[triggers]]
type     = "Keywords"
keywords = ["axum", "router", "middleware", "handler"]

depends_on = ["rust/error-handling"]
```

Templates use [Tera](https://keats.github.io/tera/) syntax. Variables available:
- `{{ task_description }}`
- `{{ task_id }}`
- `{{ task_path }}`
- `{{ acceptance_criteria }}`
- `{{ constitution_summary }}`
- `{{ karpathy_focus }}`
