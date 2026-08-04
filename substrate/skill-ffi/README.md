# `skill-ffi` — the mobile FFI boundary

Exposes the `prometheus:component/skill` invocation surface —
`run(string) -> result<string, error>` plus discovery — across the FFI boundary,
so a mobile caller sees the same contract a Wasm host does. If the two diverge,
a skill behaves differently depending on how it was reached, which is the
failure this phase exists to prevent.

Pattern: **`flutter_rust_bridge` 2.12.0**, per
[`docs/decisions/mobile-ffi-pattern.md`](../../docs/decisions/mobile-ffi-pattern.md).

## Status: Prometheus Exec is live; legacy `run_skill` remains unbound

| Claim | Status |
|---|---|
| exec-enabled `aarch64-apple-ios` build | **builds; release pending** — retained delta 25,935,300 bytes exceeds 12 MiB |
| exec-enabled `aarch64-linux-android` build | **builds; release pending** — retained delta 31,701,736 bytes exceeds 12 MiB |
| generated FRB dispatcher | **yes** — checked in, reproducible, and export-checked for both mobile ABIs |
| round-trip tests assert on returned values | **yes** — 12 passing |
| `crate-type` matches KnowMe's `gen_ui_ffi` | **yes** — `cdylib`, `staticlib`, `rlib` |
| `exec_run` actually invokes a signed Tier W component | **yes** |
| legacy `run_skill` resolves catalog IDs automatically | **NO** |

`run_skill` returns `Unsupported` with `"no host bound"` rather than a result.
That is the truthful answer while UAR's Wasm runtime is a stub
(`change-msp-008`): returning `Ok` would make a mobile caller believe a skill
ran when nothing did. One test asserts exactly this, so a future change cannot
quietly make it fake success.

The Prometheus Exec surface is separately live. Trusted Rust host code installs
one `EmbeddedExecutionApi`; plain async `exec_run`, `exec_status`,
`exec_events`, `exec_receipt`, `exec_artifact`, and `exec_verify` functions in
`api.rs` are available to Flutter Rust Bridge. They return concrete values and
signed receipts, use the embedding app's existing Tokio runtime, and never
accept private signing keys. The same `EmbeddedExecutionAdapter` methods are
usable as thin Tauri commands.

The mobile KBD surface is implemented by `kbd-mobile` and exposes:

- restricted capability discovery;
- host-key preparation and attachment for signed commands;
- commit of prepared signed events into the local Loro authority;
- sovereign-sync-compatible delta preparation, host signing, and import.

The wire envelope and topic derivation are byte-compatible with
`sovereign-sync`. Secure device keys stay with the host application. Git,
adoption, submodule scanning, and audit-ref writes are absent by design.

## Falsifiers from the decision — both tested

The pattern decision was recorded **provisional** pending two checks, to be run
*before* writing bindings rather than after:

| Falsifier | Result |
|---|---|
| **2 — can `flutter_rust_bridge` consume a crate outside the app's Cargo workspace?** | **Passed.** This crate is in `substrate/`, has no relationship to KnowMe's workspace, and builds for both mobile targets with `flutter_rust_bridge` as a plain dependency. No vendoring. |
| **assumption — does it express `run(string) -> Result<String, E>`?** | **Passed.** `run_skill` has exactly that signature with a structured error, and compiles for both targets. |
| **3 — what does adding one function cost?** | **Passed, and it is 0.** `change-uhe-003` added `list_skills` and counted the glue: 0 FFI attributes, 0 `extern "C"`, 0 Cargo.toml, 0 build-script, 0 Dart. The 21 lines in `api.rs` are 9 doc comments, 3 inline comments, 1 blank, and 8 lines of ordinary function body. Threshold was >~20; actual 0. |

**All three falsifiers are closed and the decision stands on measurement.**
`flutter_rust_bridge` generates from the plain signature, so the marginal cost of
a new function is the function.

## Build

```bash
bash generate-frb.sh          # deterministic Rust bridge regeneration
bash generate-frb.sh --check  # fail on checked-in dispatcher drift
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

## Release boundary

- **Generated Rust glue is checked in.** It is the exported dispatcher that
  retains the complete API graph. `generate-frb.sh` pins codegen 2.12.0 and
  reproduces it byte-for-byte. Generated Dart remains owned by the consuming
  application and is intentionally not committed here.
- **Mobile Tier W is not release-ready.** Fair baseline/current measurements
  include a generated dispatcher on both sides. The retained iOS and Android
  deltas exceed the 12 MiB gate, and physical-device round trips are still
  pending. Cross-build success is not represented as mobile certification.
- **No implicit catalog-ID dispatch in `run_skill`.** Callers use the explicit
  content-addressed `exec_run` contract until catalog resolution is bound to a
  trusted host generation.
