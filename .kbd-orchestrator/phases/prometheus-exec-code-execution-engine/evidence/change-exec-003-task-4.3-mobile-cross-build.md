# Change EXEC-003 task 4.3 — mobile cross-build and size evidence

Date: 2026-08-04

Host: macOS 26, x86_64

Rust: 1.97.1

Current source: `9371c01` plus the task 4.3 build-contract change

Baseline source: `6c5edc7` (before the `skill-ffi` execution surface)

## Scope and claim boundary

The repository does not contain a package or artifact literally named
`gen_ui_core`. Its owned mobile execution boundary is `skill-ffi`, which is the
library linked by the host runtime described as `gen_ui_core` in the OpenSpec.
This evidence therefore measures the release `skill-ffi` artifact for each
available arm64 mobile ABI. It does not claim a final downstream application
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

ANDROID_NDK_HOME=/Users/gqadonis/Library/Android/sdk/ndk/28.0.12433566 \
CARGO_TARGET_DIR=/tmp/prometheus-exec-current \
  substrate/skill-ffi/build-mobile.sh all

git worktree add --detach /tmp/prometheus-exec-baseline-worktree 6c5edc7
CARGO_TARGET_DIR=/tmp/prometheus-exec-baseline \
  cargo build --manifest-path substrate/skill-ffi/Cargo.toml \
  --release --target aarch64-apple-ios

ANDROID_NDK_HOME=/Users/gqadonis/Library/Android/sdk/ndk/28.0.12433566 \
CARGO_TARGET_DIR=/tmp/prometheus-exec-baseline \
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=/Users/gqadonis/Library/Android/sdk/ndk/28.0.12433566/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android35-clang \
  cargo build --manifest-path substrate/skill-ffi/Cargo.toml \
  --release --target aarch64-linux-android
```

## Per-ABI results

| ABI | Baseline bytes | Current bytes | Delta bytes | 12 MiB gate | Result |
| --- | ---: | ---: | ---: | ---: | --- |
| iOS arm64 (`aarch64-apple-ios`) | 84,672 | 679,816 | +595,144 | 12,582,912 | PASS |
| Android arm64 (`aarch64-linux-android`) | 3,511,136 | 7,195,008 | +3,683,872 | 12,582,912 | PASS |

Artifact identities:

| Artifact | SHA-256 | File identity |
| --- | --- | --- |
| baseline iOS | `59dec8440599246c9f94d9d8e8eeb259722febf9c158955a15d9d575b92dbcdf` | Mach-O 64-bit dynamically linked shared library arm64 |
| current iOS | `4cde74cc800c19a33c73d14694e14a6374fa774ea01233ea5367a01157cc352a` | Mach-O 64-bit dynamically linked shared library arm64 |
| baseline Android | `5621a854f7bc301a0841d3ae3dcbf8cf781f48f588dd99d75f4d0d6cbeb4dedc` | ELF 64-bit LSB shared object, ARM aarch64 |
| current Android | `741d27c99e16d06cc628dec0fdea420e87bfef00b9b463e06e7f62a656556fd8` | ELF 64-bit LSB shared object, ARM aarch64 |

## Disposition

Task 4.3 passes for the two supported arm64 cross-build profiles available on
this host. Both artifacts select Pulley execution with JIT prohibited, and both
measured FFI deltas are below the 12 MiB release gate. Device execution remains
pending task 4.4 evidence.
