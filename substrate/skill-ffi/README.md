# `skill-ffi` — the mobile FFI boundary

Exposes the `prometheus:component/skill` invocation surface —
`run(string) -> result<string, error>` plus discovery — across the FFI boundary,
so a mobile caller sees the same contract a Wasm host does. If the two diverge,
a skill behaves differently depending on how it was reached, which is the
failure this phase exists to prevent.

Pattern: **`flutter_rust_bridge` 2.12.0**, per
[`docs/decisions/mobile-ffi-pattern.md`](../../docs/decisions/mobile-ffi-pattern.md).

## Status: builds and round-trips; **does not execute skills yet**

| Claim | Status |
|---|---|
| builds for `aarch64-apple-ios` | **yes** — 16,408-byte arm64 Mach-O dylib |
| builds for `aarch64-linux-android` | **yes** — 454,856-byte arm64 ELF `.so` |
| round-trip tests assert on returned values | **yes** — 7 passing |
| `crate-type` matches KnowMe's `gen_ui_ffi` | **yes** — `cdylib`, `staticlib`, `rlib` |
| **actually invokes a skill** | **NO** |

`run_skill` returns `Unsupported` with `"no host bound"` rather than a result.
That is the truthful answer while UAR's Wasm runtime is a stub
(`change-msp-008`): returning `Ok` would make a mobile caller believe a skill
ran when nothing did. One test asserts exactly this, so a future change cannot
quietly make it fake success.

## Falsifiers from the decision — both tested

The pattern decision was recorded **provisional** pending two checks, to be run
*before* writing bindings rather than after:

| Falsifier | Result |
|---|---|
| **2 — can `flutter_rust_bridge` consume a crate outside the app's Cargo workspace?** | **Passed.** This crate is in `substrate/`, has no relationship to KnowMe's workspace, and builds for both mobile targets with `flutter_rust_bridge` as a plain dependency. No vendoring. |
| **assumption — does it express `run(string) -> Result<String, E>`?** | **Passed.** `run_skill` has exactly that signature with a structured error, and compiles for both targets. |

Falsifier 3 (marginal cost per added function) is **not yet testable** — it
needs a second function added over time, not at authoring. Recorded as open.

## Build

```bash
bash build-mobile.sh all      # ios + android
bash build-mobile.sh ios
RUSTUP_TOOLCHAIN=stable cargo test
```

Two environment facts the script encodes, both of which cost time to discover:

- **The NDK's clang wrappers are API-suffixed.** There is no bare
  `aarch64-linux-android-clang`; the NDK ships `aarch64-linux-android24-clang`
  and friends. cc-rs looks for the bare name and fails inside `dart-sys` with an
  error naming neither the NDK nor the API level.
- **Cross-builds use stable.** The local nightly ICEs compiling `tokio` for the
  host target. Unrelated to this crate, but it makes a bare `cargo test` fail
  confusingly.

## What is deliberately absent

- **No generated Dart yet.** `flutter_rust_bridge_codegen` output belongs with
  the consuming app; generating it here would commit bindings nothing imports.
- **No host dispatch.** Wiring one now would produce a call that silently
  returns UAR's placeholder string — worse than the honest `Unsupported`.
