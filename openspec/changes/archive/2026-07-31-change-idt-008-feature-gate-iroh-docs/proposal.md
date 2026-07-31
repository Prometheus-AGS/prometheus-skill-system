# Feature-gate iroh-docs so wasm32 is reachable

**Change:** `change-idt-008-feature-gate-iroh-docs`
**Phase:** ideation-and-decision-tools
**Goal:** enables added scope · **Library:** cand-003 (adapt)

## Why

substrate/storage-provider/Cargo.toml:19 pins iroh-docs unconditionally with
fs-store. iroh-docs is not wasm-compatible, so the crate cannot build for
wasm32 at all today.

## What

See `.kbd-orchestrator/phases/ideation-and-decision-tools/plan.md` for full
rationale, acceptance criteria, and the adversarial review record.
