---
paths: ['**/*.rs', '**/Cargo.toml', '**/Cargo.lock', '**/clippy.toml']
---

# Rust

Loaded when a Rust file is read. Not resident.

| Tier | Commands |
|---|---|
| T0 every edit | `cargo check -p <crate>`; `cargo clippy -p <crate> --no-deps -- -D warnings` |
| T1 unit complete | `cargo test -p <crate> <module_or_test>` — the just-written unit only |
| T2 phase complete | `cargo test --workspace`; `cargo build` (dev); doc tests if a public API changed |
| T3 milestone only | `cargo build --release`; cross-compiles; vendored native builds; feature-flag matrix; e2e |

## Hard rules

- Never `--release` during implementation; never cross-compile before T2 passes; one build profile per session.
- Scope T0 to the touched crate. Workspace-wide checks on every edit are waste.
- Never `panic = "abort"` on a profile that ships through `flutter_rust_bridge` — it needs unwinding.

## Build concurrency (A-10, G-4)

Within one target directory, single-writer — serialize; the `build-guard` hook blocks a second cargo
process on the same manifest. Across worktrees with separate `CARGO_TARGET_DIR` and a **shared**
`CARGO_HOME`, run check, build, test and clippy in parallel; serialize only `cargo fetch`, `update`, `add`.
Never give an agent its own `CARGO_HOME`: the fingerprint includes that path and forces full recompiles.

## Structure — feature-based, inside Rust too

- **A bounded context is a crate.** Anything that must never be violated becomes a crate boundary, because
  the dependency graph is the only enforcement Rust gives for free. A domain crate is pure; nothing with
  I/O may become its dependency to silence a trait or borrow error.
- A library crate is organised by capability, never by technical kind: `orders/`, `pricing/`,
  `inventory/` — not `models.rs`, `utils.rs`, `helpers.rs`.
- A service crate: `src/features/<feature>/{domain,application,infrastructure,interface}`, plus
  `src/core/` (config, error, session) and `src/shared/`.
  `interface → application → domain ← infrastructure`. `domain` imports no axum, reqwest, sqlx, tokio or
  serde_json I/O. `interface` holds handlers and DTOs only. Features never import each other; shared code
  moves to `core/` or `shared/`.
- `mod.rs` files re-export and wire; they hold no logic.

## File size

No `.rs` file over 500 lines. Before it gets there, turn `foo.rs` into `foo/{mod.rs,…}` split by
responsibility (types, the operation, its errors, its tests), keeping the public path stable through
re-exports. Inline `#[cfg(test)]` modules count; move large ones to `foo/tests.rs`. `clippy::too_many_lines`
covers functions; `scripts/check-file-lines.sh` covers files.
