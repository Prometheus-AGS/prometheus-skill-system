---
type: Decision
id: decision-uniffi-is-the-mobile-ffi-pattern-not-cbindgen-hand-written-jni
title: "Decision: uniffi is the mobile FFI pattern, not cbindgen + hand-written JNI"
tags:
- decision
- outcome-recorded
outcome_status: recorded
outcome_recorded_at: 2026-07-31T11:43:01Z
decided_at: 2026-07-31T11:40:57Z
links: []
sources: []
---

# Decision: uniffi is the mobile FFI pattern, not cbindgen + hand-written JNI

## Decision

Use **uniffi 0.31.2** (proc-macro mode, no `.udl`) for the pack's mobile FFI
bindings, following the `frf-ffi` pattern. Do **not** adopt liter-llm's
cbindgen + hand-written JNI pattern, despite it already being in-tree and at
far greater scale.

## Assumptions

- The pack's FFI surface is the **skill-invocation surface** —
  `run(string) -> result<string, error>` from `prometheus:component/skill`, plus
  discovery. That is a handful of functions, not hundreds. **Untested for the
  final shape**, since `change-msp-009` has not built it yet; if the surface
  turns out to be large and performance-critical, the cost comparison shifts.
- Both iOS and Android matter. If only one target ever ships, the argument for
  generating both weakens sharply.
- uniffi 0.31.2 supports the types the surface needs (strings, results, records).
  Not verified against the final signature — an unsupported type would force a
  lowering layer that erodes the advantage.

## Falsifier

**Reverse this decision if any of the following is measured:**

1. **Flutter cannot reach a uniffi cdylib without more glue than the JNI path
   would have cost.** Concretely: if a working Dart binding for the
   skill-invocation surface takes **more than 300 lines** of hand-written Dart
   plus FFI declarations, cbindgen's existing Dart bridge is the cheaper path
   and this decision was wrong.
2. **uniffi cannot express the surface.** If `run(string) -> result<string, error>`
   or the discovery functions need a type uniffi 0.31.2 does not support, forcing
   a JSON-string lowering for everything, the generation advantage collapses to
   "generates a string-passing shim" — which cbindgen also does.
3. **Binding size or startup cost is prohibitive on-device.** If the uniffi
   cdylib is more than **2×** the equivalent hand-written binding on
   `aarch64-apple-ios`, the size matters more than the maintenance saving for an
   app that ships to phones.

Each is checkable during `change-msp-009` — this decision is deliberately made
*before* the bindings exist, so it can be tested by building them rather than
ratified afterwards.

## Outcome

**Status: recorded** (2026-07-31T11:43:01Z)

Reversed within the same change. Adversarial review returned CRITICAL: the stated mobile target is Flutter, and optimising for generated Kotlin/Swift solved the wrong problem. Checking know-me-system showed flutter_rust_bridge 2.12.0 was already in production there. Superseded by the flutter_rust_bridge decision in the same file.
