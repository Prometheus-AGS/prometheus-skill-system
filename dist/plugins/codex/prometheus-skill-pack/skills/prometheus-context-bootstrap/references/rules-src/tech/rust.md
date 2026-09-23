---
paths: ['**/*.rs', '**/Cargo.toml', '**/Cargo.lock', '**/clippy.toml']
---

# Rust

Loaded when a Rust file is read. Not resident.

Load `prometheus-rust-workspace` before Rust implementation, review, refactoring, or
architecture work. It routes installed skills and owns Cargo timing; loading a skill
does not authorize immediate command execution.

Use `rust-best-practices` for general Rust work, `rust-async-patterns` for Tokio,
async I/O, concurrency, or cancellation, and `rust-mcp-server-generator` for Rust
MCP servers and transports. Project dependency pins and protocol versions win over
skill examples.

## Phase-gated integration verification

- Batch related edits and use static reasoning or rust-analyzer during implementation.
- Run one consolidated Cargo validation batch after a meaningful production path is
  complete at a change or phase boundary. Use a narrow compiler check earlier only
  when compiler feedback is required to unblock progress or the user requests it.
- Start with the smallest integration target that exercises the completed public
  entry point and real collaborators. Unit, module-local, mock-only, and filtered
  function tests do not count as completion evidence.
- Escalate only when the change or diagnostics cross package boundaries. Reserve
  broad, release, cross-compile, feature-matrix, Miri, fuzzing, coverage, benchmark,
  and documentation commands for a final boundary or explicit request.
- After failure, read all diagnostics, batch fixes, and rerun only the smallest
  integration command that can confirm the completed behavior.
- Run formatting once at a coherent boundary and avoid unrelated churn.
- Report commands, results, intentional deferrals, and any remaining integration gate.

## Hard rules

- Never `--release` during implementation; never cross-compile before the affected integration gate passes; one build profile per session.
- Do not use `--all-features` unless the project supports combined features or the user requests it.
- Never `panic = "abort"` on a profile that ships through `flutter_rust_bridge` — it needs unwinding.

## Build concurrency (A-10, G-4)

Within one workspace or target directory, serialize every Cargo command; the `build-guard` hook blocks a
second compile on the same manifest. Before an expensive command, account for any Cargo process already
running. Across isolated worktrees, keep separate default target directories and a shared `CARGO_HOME`;
serialize dependency-mutating commands such as `cargo fetch`, `update`, and `add`. Do not create extra
target directories merely to bypass ordinary lock contention, and never run `cargo clean` unless cleanup
is the requested task.

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
