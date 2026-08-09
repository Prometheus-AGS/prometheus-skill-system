---
paths: ['**/*.rs', '**/Cargo.toml']
---

# Rust

Loaded when a Rust file is read. Not resident.

| Tier | Commands |
|---|---|
| T0 every edit | `cargo check -p <crate>`; `cargo clippy -p <crate> --no-deps` |
| T1 unit complete | `cargo test -p <crate> <module>` — the just-written unit only |
| T2 phase complete | `cargo test --workspace`; `cargo build` (dev); doc tests if a public API changed |
| T3 milestone only | `cargo build --release`; cross-compiles; vendored native builds; feature-flag matrix; device certification |

## Hard rules

- Never `--release` during implementation. It invalidates incremental artifacts
  and pays full optimization for code that will change.
- Never cross-compile or run a vendored native build before T2 passes.
- One build profile per session. Switching profiles thrashes the incremental cache.
- Scope T0 to the touched crate. Workspace-wide checks on every edit are waste.

## Build concurrency

Within one target directory, single-writer — serialize.

Across worktrees with separate `CARGO_TARGET_DIR` and a **shared** `CARGO_HOME`,
run check, build, test, and clippy in parallel. Serialize only
dependency-mutating commands: `cargo fetch`, `cargo update`, `cargo add`.

Do not give each agent its own `CARGO_HOME`. The fingerprint includes that path,
so a separate one breaks registry sharing and forces full recompiles.

## Capability inversion

Agent kernels do not depend on write-capable crates. Mutations live in the host
layer. The dependency graph is the enforcement point, so adding a write-capable
dependency to a kernel crate breaks the invariant at compile time — do not
add one to silence a borrow or trait error.

<!-- Replace the commands above with this project's real ones if they differ. -->
