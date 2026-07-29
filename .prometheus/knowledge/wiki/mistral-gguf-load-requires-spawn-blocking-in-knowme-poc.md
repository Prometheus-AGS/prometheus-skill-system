---
type: Reference
id: mistral-gguf-load-requires-spawn-blocking-in-knowme-poc
title: Mistral GGUF Load Requires spawn_blocking in KnowMe PoC
tags:
- hybrid-mobile
- knowme-poc
- mistral-rs
- tokio
- spawn-blocking
- gguf
- local-inference
links:
- hybrid-mobile-poc-codegen-and-ci-verification-plan
sources:
- stdin
- manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification
timestamp: 2026-07-16T21:02:27.499491+00:00
created_at: 2026-07-16T21:02:27.499491+00:00
updated_at: 2026-07-16T21:02:27.499491+00:00
revision: 0
---

## Context

- **Project:** Hybrid Mobile Architecture
- **Phase:** `phase-codegen-and-ci-verification`
- **KBD root:** `/Users/gqadonis/Projects/hybrid-mobile-architecture-src/.claude/worktrees/compassionate-babbage-7cd4bc`
- **Captured:** `2026-07-16T21:01:49Z`
- **Position:** `phase-codegen-and-ci-verification | status: executing`
- **Related plan:** [Hybrid Mobile PoC Codegen and CI Verification Plan](/hybrid-mobile-poc-codegen-and-ci-verification-plan.md)

## Phase Goal

The revised phase goal is to build a working proof-of-concept application in `apps/<name>/`, based on KnowMe reference documentation in `docs/reference-app/`, and use it to prove the repository skill package end-to-end.

The PoC should exercise:

- Streaming `ContentBlock` chat
- PEM entity management
- SurrealDB graph-RAG memory
- Local-first sync
- Cross-platform Flutter/Tauri/web surfaces from one Rust core
- Codegen and CI verification as supporting objectives

Supporting verification goals include:

- Run the real codegen pipeline:
  - `flutter_rust_bridge_codegen generate`
  - `dart run build_runner build`
  - `flutter pub get`
  - `pnpm install`
- Resolve or work around the PEM workspace dependency blocker: `@prometheus-ags/entity-graph-core@workspace:*` outside the PEM monorepo.
- Verify at least one target per surface:
  - macOS Tauri desktop
  - iOS simulator or Android emulator for Flutter
- Wire CI to run:
  - `cargo clippy --workspace`
  - `audit.sh all`
  - Boundary test suites against the PoC on every push

## T12d Result: Real Runtime-Blocking Bug Found

T12d verified the Mistral local inference assumptions against source and an isolated Tokio repro. The result split into two findings:

| Path | Original assumption | Result |
|---|---|---|
| `generate` / streaming inference | Safe to avoid `spawn_blocking` | Correct |
| `load` / GGUF model loading | Safe to avoid `spawn_blocking` because loader manages its own threading | Incorrect; bug fixed |

## `generate` Path Was Correct

`mistral.rs` already handles inference without blocking the application runtime:

- `mistralrs-core/src/lib.rs:813` spawns its own OS thread with a private Tokio runtime for the engine.
- `stream_chat_request` sends a request through a channel and returns a `Stream` over responses.
- Inference does not run on the caller's Tokio runtime.

Conclusion: wrapping the streaming inference path in `spawn_blocking` would have been wrong.

## `load` Path Was Blocking the Tokio Runtime

`GgufModelBuilder::build()` is declared as `async fn`, but its body calls `Loader::load_model_from_hf`, which is a synchronous `fn`, not an `async fn`, and is invoked with no `.await`.

Implication:

- Multi-GB model download and dequantization run inline on whichever thread polls the future.
- `spawn_blocking` was absent from the GGUF load path.
- The existing code comment claiming the loader “drives its own internal threading” was wrong.

This was corrected in `apps/knowme-poc/rust/crates/gen_ui_inference/src/mistral.rs`.

## Verification Evidence

A 4-worker Tokio repro matching the load-path shape measured runtime responsiveness:

| Scenario | Heartbeats | Verdict |
|---|---:|---|
| `.await build()` | `1 / ~150` | Runtime stalled |
| `spawn_blocking(build)` | `128 / ~150` | Runtime responsive, same wall-clock duration |

The stall was proven by:

1. Reading the Mistral/GGUF source path.
2. Reproducing the same blocking-future shape in isolation.

Limit: no real 1GB+ model load was timed directly. The fix is still correct because the synchronous blocking call is confirmed in the load path.

## Fix

`load()` now runs `build()` on Tokio's blocking pool. Because the future is non-`Send`, the implementation drives it from a current-thread Tokio runtime created inside the `spawn_blocking` closure.

Important invariant preserved:

- The process still has one multi-thread Tokio runtime.
- The nested runtime is current-thread and scoped to the blocking closure.

Validation performed:

- `cargo clippy` is clean under `--features local-mistral`.
- The module was confirmed to compile under that feature with a deliberate-error probe, avoiding a false-positive empty pass.

## Commit

- Commit: `0123c51`
- Primary file: `apps/knowme-poc/rust/crates/gen_ui_inference/src/mistral.rs`
- `design.md` was corrected because it had marked the loader-threading assumption as unverified.
- `.prometheus/` logs were included under standing authorization.

## C-105 Status

C-105 verification tasks are now complete:

- T12 verified
- T12b verified
- T12c verified
- T12d verified

Pattern observed across T12 lanes: each path appeared correct by inspection, but real execution or runtime-shape verification exposed hidden issues.

Remaining phase work is the broader codegen/CI verification unless the fresh-clone `pnpm dev` bootstrap gap from T12c is prioritized first.

# Citations

1. stdin
2. manual:Hybrid Mobile Architecture/phase-codegen-and-ci-verification