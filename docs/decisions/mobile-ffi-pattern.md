# Decision: follow the target — `flutter_rust_bridge` for mobile, uniffi only if a second host appears

**Status:** **accepted** · 2026-07-31 · `change-msp-007-ffi-pattern-decision`
**Phase:** mobile-skill-portability
**Supersedes:** an earlier draft of this file that chose uniffi outright. See
[What changed and why](#what-changed-and-why).

## Decision

> **Accepted 2026-07-31 by `change-msp-009`, after both load-bearing
> assumptions were tested rather than assumed.** It was recorded *provisional*
> until then, because review objected — correctly — to accepting a decision
> while cheap checks were outstanding.
>
> | Check | Result |
> |---|---|
> | expresses `run(string) -> Result<String, E>` | **passed** — that exact signature compiles for both mobile targets |
> | consumes a crate **outside** the app's Cargo workspace, without vendoring | **passed** — `substrate/skill-ffi` has no relationship to KnowMe's workspace and builds for `aarch64-apple-ios` (16,408 B dylib) and `aarch64-linux-android` (454,856 B .so) |
>
> **Falsifier 3 CLOSED 2026-07-31 by `change-uhe-003` — measured, not estimated.**
>
> A fourth function (`list_skills`) was added to `substrate/skill-ffi` and the
> hand-written glue counted:
>
> | Category | Lines added |
> |---|---|
> | FFI attributes / annotations (`#[...]`) | **0** |
> | `extern "C"` / `no_mangle` / `unsafe` | **0** |
> | `Cargo.toml` | **0** |
> | `lib.rs` | **0** |
> | `build-mobile.sh` | **0** |
> | Hand-written Dart | **0** |
> | **Total hand-written glue** | **0** |
>
> The 21 lines added to `api.rs` are 9 doc comments, 3 inline comments, 1 blank,
> and **8 lines of ordinary Rust function** — the function itself, not the cost
> of exposing it. `flutter_rust_bridge` generates from the plain signature, so a
> new function needs no annotation at all.
>
> Threshold was **>~20 lines reverses the decision**. Actual: **0**. The
> decision **stands**, and it stands on a measurement rather than on the absence
> of a counter-argument. Both mobile targets still build; 8/8 tests pass.

Bind the pack's skill-invocation surface to mobile with
**`flutter_rust_bridge` 2.12.0** — the pattern **already in production at the
only mobile target that exists**.

Do **not** adopt uniffi as the primary pattern, and do **not** adopt liter-llm's
cbindgen + hand-written JNI.

## Why this reverses my first answer

The first draft framed the choice as *cbindgen (in-tree, 767 fns) vs uniffi
(generates Kotlin + Swift from one definition)* and picked uniffi on a
maintenance-cost measurement. Adversarial review returned **CRITICAL**: the
stated mobile target is **Flutter**, and optimising for generated Kotlin/Swift
"may be solving the wrong FFI problem."

Checking rather than arguing settled it. In `know-me-system`:

| Evidence | Value |
|---|---|
| `mobile/pubspec.yaml:36` | `flutter_rust_bridge: 2.12.0` |
| `rust/crates/gen_ui_ffi/Cargo.toml` | `flutter_rust_bridge = "=2.12.0"`, `crate-type = ["cdylib", "staticlib"]` |
| generated Dart bridge files | **18** |
| hand-written provider (`mobile/lib/bridge/rust_bridge_provider.dart`) | **393 lines** |

**A third pattern was already chosen, shipped, and working at the target.**
Neither candidate in my original comparison was the incumbent. Choosing uniffi
would have added a second FFI toolchain to an app that already has one — and
made Flutter, the actual delivery surface, the one platform needing a bridge to
the bridge.

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

1. **A second, non-Flutter mobile host ships.** Not "uniffi would cover Dart" —
   it does not, as the table below says plainly, and an earlier version of this
   falsifier contradicted its own evidence on that point. The real trade is:
   *one* Flutter-native binding versus *two* bindings (uniffi for the native
   host, `flutter_rust_bridge` for Flutter). Reverse only if maintaining both is
   worse than routing Flutter through uniffi's C ABI.

   **Operational test:** a native Kotlin or Swift host exists as a buildable
   target in a repo we own — a `build.gradle` or `.xcodeproj` that compiles and
   calls the pack — **not** a stated intention or a roadmap entry. Check by
   building it.
2. **`flutter_rust_bridge` cannot consume a crate from outside the app's Cargo
   workspace** without vendoring this pack into KnowMe. Test: build
   `gen_ui_ffi`-style bindings against a path dependency on a pack crate. If it
   requires vendoring, the coupling is worse than a second toolchain.
3. **Adding one skill function costs hand-written work.** Line count measures
   the wrong thing — review flagged this, and it is right: 393 lines written
   once is cheap; 40 lines per function forever is not. What actually falsifies
   the pattern is *marginal* cost.

   **Operational test:** add a second function to the surface and count the
   hand-written Dart, Rust-side annotation, and build-config lines the addition
   requires. **Reverse if adding one function needs more than ~20 lines of
   hand-written glue** — at that rate the generator is not generating, and the
   pattern's whole justification is gone.

   (The 393-line provider stays in the table as context for total size. It is
   not the threshold; the first draft's "300 lines" was arbitrary and the
   revised total-size threshold was measuring the wrong quantity.)

## What was measured, and what it does and does not show

| | `flutter_rust_bridge` (KnowMe) | uniffi (`frf-ffi`) | cbindgen + JNI (`liter-llm`) |
|---|---|---|---|
| Rust FFI source | (in `gen_ui_ffi`) | 258 lines | 19,435 lines |
| Generated bindings | **18 Dart files** | 1 Kotlin + 2 Swift | — |
| Hand-written bindings | 393-line provider | — | 150 Java + 125 other |
| Reaches Flutter | **natively** | via a bridge | via `packages/dart/` |

> **This is not a like-for-like comparison and must not be read as one.**
> Review flagged that too. liter-llm exposes 767 C functions; `frf-ffi` exposes
> a small surface. The 19,435-vs-258 ratio measures *surface size*, not pattern
> efficiency. What the table honestly shows is which patterns **reach Flutter
> without an extra layer** — and only one does.

## What this decision does not do

- **No FFI code is written.** `change-msp-009` builds the bindings and tests
  falsifiers 2 and 3 before writing much of anything.
- **It does not touch `flint-realtime-fabric`, `tools/liter-llm`, or
  `know-me-system`.** Each keeps its pattern; this governs only what *this pack*
  writes.
- **It does not retire uniffi as an option.** Falsifier 1 is a live path, and
  `frf-ffi` remains the reference if it fires.

## What changed and why

An earlier version of this file chose uniffi. It was recorded via
`decision-log.sh`, then rewritten in place after adversarial review returned
CRITICAL. The wiki entry
(`decision-uniffi-is-the-mobile-ffi-pattern-not-cbindgen-hand-written-jni.md`)
carries the superseded title and is **the record of the first answer**; this
document is the second. Both are kept deliberately — a decision log that hides
its reversals is not a log.

## Adversarial review record

Two rounds, judge `kbd-judge` via `rest-gateway:http://localhost:8181/v1`,
`cross_model_check: verified-distinct`, producer `claude-opus-5`.

**Round 1 — BLOCK (1 CRITICAL, 5 WARNING).** The CRITICAL was that optimising
for generated Kotlin/Swift "may be solving the wrong FFI problem" when the
stated target is Flutter. Checking `know-me-system` proved it: `flutter_rust_bridge`
2.12.0 was already in production there. **The decision was reversed**, not
defended.

**Round 2 — BLOCK (1 CRITICAL, 4 WARNING), all addressed:**

| Finding | Response |
|---|---|
| Falsifier 1 contradicted its own evidence — claimed uniffi would cover Dart while the table says it does not | **Accepted.** Rewritten as one Flutter-native binding vs two bindings, with a buildable-target test rather than a stated intention. |
| Accepted while two load-bearing assumptions are unverified and cheap to test | **Accepted.** Status downgraded to **provisional**; `change-msp-009` must confirm both **before** writing bindings. |
| Falsifier 1 had no measurable authority or artifact | **Accepted.** Now requires a `build.gradle` or `.xcodeproj` that compiles and calls the pack. |
| Falsifier 3's line count does not falsify maintenance cost | **Accepted.** Replaced with *marginal* cost: >~20 hand-written lines to add one function. 393 lines written once is cheap; 40 per function forever is not. |
| Prior-decision entries failed to parse, so "already settled?" is unreliable | **Acknowledged, not fixed here.** The malformed entries are the pk wiki's, not this document's; the reversal is recorded in [What changed and why](#what-changed-and-why) so the history is legible regardless. |

Stopping at the 2-round cap. No finding was rejected. The round-1 CRITICAL is
the one worth remembering: **I compared the two patterns I had found and never
asked what the target already used.** A third pattern was shipping in production
and was in neither column.
