# Change EXEC-003 task 4.3 — mobile cross-build and size evidence

Date: 2026-08-04

Host: macOS 26, x86_64

Rust: 1.97.1

Current source: `4da9748` plus the task 5.2 review-remediation change

Baseline source: `6c5edc7` (before the `skill-ffi` execution surface)

## Scope and claim boundary

The repository does not contain a package or artifact literally named
`gen_ui_core`. Its owned mobile execution boundary is `skill-ffi`, which is the
library linked by the host runtime described as `gen_ui_core` in the OpenSpec.
This evidence therefore measures the release `skill-ffi` artifact for each
available arm64 mobile ABI. Both the baseline and current artifacts contain a
real generated Flutter Rust Bridge dispatcher, so exported API roots retain
their dependency graphs. It does not claim a final downstream application
bundle delta.

Cross-build evidence is compile-only evidence. Physical-device runtime evidence
is tracked separately by task 4.4 and is not implied by this record.

## Backend contract

For both `aarch64-apple-ios` and `aarch64-linux-android`, the resolved feature
graph selects:

- `prometheus-exec-embedded/mobile`
- `prometheus-exec-tier-w/bundled-mobile`
- `prometheus-exec-tier-w/mobile`
- `prometheus-exec-tier-w/pulley`

It does not select the `prometheus-exec-tier-w/cranelift` native-execution
profile. `EngineProfile::ios()` and `EngineProfile::android()` both bind
`backend = pulley` and `jit_permitted = false`; the crate rejects a build that
combines its mobile and native-execution profiles.

Wasmtime's `cranelift` dependency feature remains compiled because Wasmtime 46
uses the compiler to translate source components into Pulley bytecode.
Execution still targets the Pulley interpreter. Removing the compiler feature
eliminates `Config::strategy` and `Component::from_binary`, so dependency-feature
absence is not a valid no-JIT test. The local build entry point instead rejects
the Tier W native-execution profile and requires the Pulley mobile profile.

## Commands

```bash
cargo tree --manifest-path substrate/skill-ffi/Cargo.toml \
  --target aarch64-apple-ios -e features -i wasmtime
cargo tree --manifest-path substrate/skill-ffi/Cargo.toml \
  --target aarch64-linux-android -e features -i wasmtime

ANDROID_NDK_HOME=<ANDROID_NDK> \
CARGO_TARGET_DIR=<current-frb-target-dir> \
  substrate/skill-ffi/build-mobile.sh all

git worktree add --detach <baseline-worktree> 6c5edc7
# Add `mod frb_generated;` beside `mod api;` in the disposable baseline.
# Create a temporary Dart root whose pubspec name is prometheus_skill_ffi and
# whose flutter_rust_bridge dependency is exactly 2.12.0, then run:
flutter_rust_bridge_codegen generate \
  --rust-root <baseline-worktree>/substrate/skill-ffi \
  --rust-input crate::api \
  --rust-output <baseline-worktree>/substrate/skill-ffi/src/frb_generated.rs \
  --dart-output <temporary-dart-root>/lib \
  --dart-root <temporary-dart-root> \
  --c-output <temporary-dart-root>/frb_generated.h \
  --no-add-mod-to-lib --no-auto-upgrade-dependency --no-build-runner \
  --no-dart-format --no-web --stop-on-error

CARGO_TARGET_DIR=<baseline-frb-target-dir> \
  cargo build --manifest-path substrate/skill-ffi/Cargo.toml \
  --release --target aarch64-apple-ios

ANDROID_NDK_HOME=<ANDROID_NDK> \
CARGO_TARGET_DIR=<baseline-frb-target-dir> \
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=<ANDROID_NDK>/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android35-clang \
  cargo build --manifest-path substrate/skill-ffi/Cargo.toml \
  --release --target aarch64-linux-android

nm -gU <ios-artifact> | grep frb_pde_ffi_dispatcher_primary
<ANDROID_NDK>/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-nm \
  -g <android-artifact> | grep frb_pde_ffi_dispatcher_primary
```

The disposable baseline `frb_generated.rs` SHA-256 was
`2035a3a287226aaac5d67c966a219d44e00ae07a7b7ea3ec6ac56a90d62da6b0`.
It contains the baseline API dispatcher but no exec cases. The checked current
dispatcher SHA-256 is
`62d8031f74e6362b1dd0cf07210255fb6be8c919f5404f88895ea20e27941226`;
it contains `wire__crate__api__exec_run_impl` and the other exec cases.

## Per-ABI results

| ABI | Baseline bytes | Current bytes | Delta bytes | 12 MiB gate | Result |
| --- | ---: | ---: | ---: | ---: | --- |
| iOS arm64 (`aarch64-apple-ios`) | 8,106,168 | 34,041,468 | +25,935,300 | 12,582,912 | **FAIL** |
| Android arm64 (`aarch64-linux-android`) | 11,818,664 | 43,520,400 | +31,701,736 | 12,582,912 | **FAIL** |

Artifact identities:

| Artifact | SHA-256 | File identity |
| --- | --- | --- |
| baseline iOS | `8452c0557929fdfd88fb2dad30bb50c9a85747fd80ccff66669787156cf91141` | Mach-O 64-bit dynamically linked shared library arm64; FRB dispatcher exported |
| current iOS | `38173e44074a858361c5a39df526959084dc2ea96ac48e732876401525307ca0` | Mach-O 64-bit dynamically linked shared library arm64; FRB dispatcher exported and exec cases generated |
| baseline Android | `248cc23a4d35328094097eb08cde5785d96f06041cab1bcb45658d02f2d7fdba` | ELF 64-bit LSB shared object, ARM aarch64; FRB dispatcher exported |
| current Android | `50a7ffdd424ef495a9877ba39746ad2011ceb08bf67eb6e5f496d21a92bc3754` | ELF 64-bit LSB shared object, ARM aarch64; FRB dispatcher exported and exec cases generated |

## Disposition

Both supported arm64 cross-build profiles compile and retain their generated
dispatcher, but task 4.3's size gate **fails**. iOS exceeds the limit by
13,352,388 bytes and Android exceeds it by 19,118,824 bytes. The earlier small
deltas were invalid because no generated dispatcher retained the exec graph.

Desktop Tier W remains certified. Mobile Tier W remains `pending_evidence` and
must not be declared release-ready until the retained per-ABI delta is below
12 MiB and physical-device task 4.4 evidence exists.
