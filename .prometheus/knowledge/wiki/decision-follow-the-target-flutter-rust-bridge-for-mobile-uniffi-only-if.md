---
type: Decision
id: decision-follow-the-target-flutter-rust-bridge-for-mobile-uniffi-only-if
title: Decision: follow the target — `flutter_rust_bridge` for mobile, uniffi only if a second host appears
tags:
- decision
- outcome-pending
outcome_status: pending
decided_at: 2026-07-31T11:43:02Z
links: []
sources: []
---

# Decision: follow the target — `flutter_rust_bridge` for mobile, uniffi only if a second host appears

## Decision

Bind the pack's skill-invocation surface to mobile with
**`flutter_rust_bridge` 2.12.0** — the pattern **already in production at the
only mobile target that exists**.

Do **not** adopt uniffi as the primary pattern, and do **not** adopt liter-llm's
cbindgen + hand-written JNI.

## Assumptions

- **Flutter is the mobile target.** Verified: `know-me-system/mobile/` is a
  Flutter app with `flutter_rust_bridge` wired in. If a native Kotlin or Swift
  host appears, this assumption breaks and uniffi returns as a live option.
- **`flutter_rust_bridge` 2.12.0 can express `run(string) -> Result<String, E>`.**
  Highly likely — strings and Results are its core cases — but **not verified for
  our exact signature**. `change-msp-009` verifies it first, not last.
- **A `cdylib` + `staticlib` crate in this pack can be consumed the same way
  `gen_ui_ffi` is.** Unverified: `gen_ui_ffi` lives inside KnowMe's own
  workspace, and a cross-repo crate may need packaging work that an in-workspace
  one does not.

## Falsifier

Reverse this decision if **any** of these is measured during `change-msp-009`:

1. **A second, non-Flutter mobile host is committed to.** Then one generator
   covering Kotlin + Swift + Dart beats a Flutter-specific one, and uniffi wins
   on exactly the argument the first draft made. This is the most likely
   reversal, and it is a product decision, not a technical measurement.
2. **`flutter_rust_bridge` cannot consume a crate from outside the app's Cargo
   workspace** without vendoring this pack into KnowMe. Test: build
   `gen_ui_ffi`-style bindings against a path dependency on a pack crate. If it
   requires vendoring, the coupling is worse than a second toolchain.
3. **The generated Dart surface needs more hand-written glue than the existing
   provider.** Threshold: **more than 393 lines** of new hand-written Dart for
   the skill-invocation surface — the measured size of KnowMe's current provider
   for its *whole* gen-UI surface. Needing more than that for a handful of skill
   functions means the pattern is not carrying its weight here.

Threshold 3 is grounded in a measured comparator in the same codebase, not a
round number. The first draft's "300 lines" was arbitrary — review flagged it,
correctly.

## Outcome

**Status: pending.** Nothing has been recorded yet.

A decision without a recorded outcome cannot be checked against what actually
happened — and idea rankings are known to flip after execution, so the judgement
made here is exactly the thing that needs checking later.

Record it with:

```
decision-log.sh outcome --id decision-follow-the-target-flutter-rust-bridge-for-mobile-uniffi-only-if --result -
```
