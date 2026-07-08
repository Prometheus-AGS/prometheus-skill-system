# Tasks — change-prb-002-cli-subcommands

- [x] Create `src/job/mod.rs` with module pub re-exports
- [x] Create `src/job/checkpoint.rs` with `JobCheckpoint` serde struct, `read()`, `write()`, `update_status()`
- [x] Create `src/job/spawn.rs` with `spawn_job()` — self-re-exec + PID write
- [x] Create `src/job/cancel.rs` with `cancel_job()` — read PID, send SIGTERM, update checkpoint
- [x] Add `Start`, `Status`, `Cancel` subcommands to `Cli` in `main.rs`
- [x] Wire subcommand handlers in `main()` match block
- [x] Write unit tests in `#[cfg(test)]` mod for checkpoint read/write round-trip
- [x] Run `cargo build --release` — 0 errors
- [x] Manual smoke: `prometheus-research start "hello world"` → prints job ID
