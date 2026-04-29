---
id: change-002-toolchain-bootstrap
title: Auto-build submodule binaries + add wasm32-wasip2 + npm run doctor
phase: phase-compliance-and-power-multiplier
gaps: [F1, F2, F3, F4]
priority: P0
effort: S
agent: devops-engineer
evolver_item_id: null
status: DONE
completed: 2026-04-28
---

# change-002 — Toolchain Bootstrap

## Context

`scripts/check-prerequisites.sh` correctly detects Node + Rust and offers to install
both, but currently a user can pass the prereq check and still have no working
pipeline because the four submodule binaries (`forge`, `pk`, `liter-llm`,
`surreal-memory-server`) are never built. The WASM packaging path (change-003 → 005)
also requires the `wasm32-wasip2` rustup target which is not installed.

## Scope

In:

- Add `--build-tools` flag to `check-prerequisites.sh`. When passed alongside
  `--install`, the script:
  1. `git submodule update --init --recursive` (in case submodules are not yet present).
  2. For each of `tools/forge-rs`, `tools/prometheus-knowledge`, `tools/liter-llm`,
     `tools/surreal-memory-server`: `cargo build --release` and copy the relevant
     binary (`forge`, `pk`, `liter-llm`, `surreal-memory-server`) to `~/.local/bin`
     (creating it if missing) or to `/usr/local/bin` if writable.
  3. Idempotency: skip the build if `command -v <bin>` already resolves AND the
     binary's `--version` matches the workspace version.
- Add `rustup target add wasm32-wasip2` to the Rust detection step (only when
  `--install` is set).
- Add a Docker / Docker Desktop / Compose v2 detection block (mirror the logic from
  the `docker-detect.sh` template the native-agent generator emits).
- Add `npm run doctor` script to `package.json` that runs prereq + build-tools +
  smoke tests (`forge --version`, `pk --version`, `liter-llm --version`,
  `surreal-memory-server --version`).

Out:

- Anything Windows-specific. `~/.local/bin` is the macOS/Linux convention; Windows
  users get a clear error message asking them to use WSL.
- Network/tunneling setup for surreal-memory-server clustering.

## Deliverables

1. `scripts/check-prerequisites.sh` with `--build-tools` flag and Docker detection.
2. `package.json` with `"doctor": "bash scripts/check-prerequisites.sh --install --build-tools"`.
3. `scripts/smoke-test.sh` (new) — runs `--version` on each binary and exits non-zero
   if any are missing or fail.

## Acceptance Criteria

- On a clean Mac without any submodule binaries: `npm run doctor` exits 0 and all
  four binaries are in `$PATH`.
- `rustup target list --installed | grep wasm32-wasip2` returns a hit after running.
- Re-running `npm run doctor` on an already-bootstrapped machine completes in
  under 5 seconds (idempotency check passes).
- Script handles a partial failure (e.g. `liter-llm` build fails) by reporting which
  binary failed and continuing with the others, not aborting the whole script.

## Files to Touch

- `scripts/check-prerequisites.sh`
- `scripts/smoke-test.sh` (new)
- `package.json`
- `README.md` — document `npm run doctor` in the Getting Started section

## Test Plan

- On a clean Linux container: `npm run doctor` from a fresh clone, no binaries.
- On the same container after first run: `npm run doctor` should be idempotent.
- Simulated failure: temporarily break `tools/forge-rs/Cargo.toml`, run doctor,
  confirm the script reports forge failure but builds the others.
