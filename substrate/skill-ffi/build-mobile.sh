#!/usr/bin/env bash
# build-mobile.sh — build the FFI boundary for real mobile targets.
#
# Usage: build-mobile.sh [ios|android|all]
# Exit:  0 built · 1 prerequisite missing · 2 build failed
#
# ANDROID NEEDS THE NDK ON PATH, AND THE WRAPPER NAMES ARE API-SUFFIXED.
# A bare `aarch64-linux-android-clang` does not exist — the NDK ships
# `aarch64-linux-android24-clang` and friends. cc-rs looks for the bare name and
# fails with a confusing dart-sys build error that names neither the NDK nor the
# API level. This script sets the toolchain explicitly so that failure cannot
# recur silently.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
cd "$HERE" || exit 1
WHAT="${1:-all}"
TARGET_DIR="${CARGO_TARGET_DIR:-$HERE/target}"

# Cross-builds use stable: the local nightly ICEs compiling tokio.
export RUSTUP_TOOLCHAIN="${RUSTUP_TOOLCHAIN:-stable}"

verify_mobile_profile() {
  local target="$1"
  local tree
  grep -Fq 'wire__crate__api__exec_run_impl' "$HERE/src/frb_generated.rs" || {
    echo "[ffi] generated dispatcher does not retain the exec API" >&2
    return 2
  }
  tree="$(cargo tree --target "$target" -e features -i prometheus-exec-tier-w)" || return 2

  grep -Fq 'prometheus-exec-tier-w feature "pulley"' <<<"$tree" || {
    echo "[ffi] $target does not select the Tier W Pulley backend" >&2
    return 2
  }
  grep -Fq 'prometheus-exec-embedded feature "mobile"' <<<"$tree" || {
    echo "[ffi] $target does not select the embedded mobile profile" >&2
    return 2
  }
  if grep -Fq 'prometheus-exec-tier-w feature "cranelift"' <<<"$tree"; then
    echo "[ffi] $target incorrectly selects the Tier W native-execution profile" >&2
    return 2
  fi

  # Wasmtime retains its Cranelift *compiler* support to translate source Wasm
  # into Pulley bytecode. The rejected feature above is this crate's native
  # execution profile; the selected Pulley target executes without JIT.
  echo "[ffi] $target profile: pulley execution, jit_permitted=false"
}

verify_generated_dispatcher() {
  local artifact="$1"
  shift
  "$@" "$artifact" | grep -Eq 'frb_pde_ffi_dispatcher_primary' || {
    echo "[ffi] generated Flutter Rust Bridge dispatcher is absent from $artifact" >&2
    return 2
  }
  echo "[ffi] retained generated dispatcher: $artifact"
}

build_ios() {
  rustup target list --installed | grep -q aarch64-apple-ios || {
    echo "[ffi] rustup target add aarch64-apple-ios" >&2; return 1; }
  verify_mobile_profile aarch64-apple-ios || return $?
  cargo build --release --target aarch64-apple-ios || return 2
  verify_generated_dispatcher \
    "$TARGET_DIR/aarch64-apple-ios/release/libskill_ffi.dylib" nm -gU || return $?
  echo "[ffi] ios: $(wc -c < "$TARGET_DIR/aarch64-apple-ios/release/libskill_ffi.dylib" | tr -d ' ') bytes"
}

build_android() {
  rustup target list --installed | grep -q aarch64-linux-android || {
    echo "[ffi] rustup target add aarch64-linux-android" >&2; return 1; }
  verify_mobile_profile aarch64-linux-android || return $?
  local ndk="${ANDROID_NDK_HOME:-}"
  if [ -z "$ndk" ]; then
    ndk="$(ls -d "$HOME/Library/Android/sdk/ndk/"* 2>/dev/null | sort -V | tail -1)"
  fi
  [ -n "$ndk" ] && [ -d "$ndk" ] || {
    echo "[ffi] Android NDK not found. Set ANDROID_NDK_HOME." >&2; return 1; }

  local tb="$ndk/toolchains/llvm/prebuilt/darwin-x86_64/bin"
  [ -d "$tb" ] || tb="$ndk/toolchains/llvm/prebuilt/darwin-arm64/bin"
  [ -d "$tb" ] || tb="$ndk/toolchains/llvm/prebuilt/linux-x86_64/bin"
  local cc
  cc="$(ls "$tb"/aarch64-linux-android*-clang 2>/dev/null | sort -V | tail -1)"
  [ -n "$cc" ] || { echo "[ffi] no aarch64-linux-android*-clang in $tb" >&2; return 1; }

  export PATH="$tb:$PATH"
  export CC_aarch64_linux_android="$cc"
  export AR_aarch64_linux_android="$tb/llvm-ar"
  export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$cc"
  cargo build --release --target aarch64-linux-android || return 2
  verify_generated_dispatcher \
    "$TARGET_DIR/aarch64-linux-android/release/libskill_ffi.so" \
    "$tb/llvm-nm" -g || return $?
  echo "[ffi] android: $(wc -c < "$TARGET_DIR/aarch64-linux-android/release/libskill_ffi.so" | tr -d ' ') bytes"
}

RC=0
case "$WHAT" in
  ios)     build_ios || RC=$? ;;
  android) build_android || RC=$? ;;
  all)     build_ios || RC=$?; build_android || RC=$? ;;
  *) echo "usage: $0 [ios|android|all]" >&2; exit 1 ;;
esac
exit "$RC"
