# Cross-repo authorisation — GRANTED 2026-07-31

The user, verbatim:

> "yes, you are authorized to write to universal-agent-runtime because I own it"

## Scope

- **Granted:** `universal-agent-runtime`
- **NOT granted, and not assumed:** `flint-realtime-fabric`, `know-me-system`.
  Both remain untouched unless separately authorised.

## What this unblocks

`change-msp-008` was archived BLOCKED for want of exactly this. Goal 1
(de-stubbing `src/uar/runtime/skills/wasm_runtime.rs:92-111`) is now deliverable,
and the four new requirements below are in scope for the same repo.
