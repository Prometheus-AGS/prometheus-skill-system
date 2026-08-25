#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
FIXTURE="$(mktemp -d)"
trap 'rm -rf "$FIXTURE"' EXIT

mkdir -p "$FIXTURE/src"
cat > "$FIXTURE/Cargo.toml" <<'TOML'
[package]
name = "weather-check"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]
TOML
cat > "$FIXTURE/src/lib.rs" <<'RUST'
#[no_mangle]
pub extern "C" fn alloc(size: i32) -> i32 {
    let mut bytes = Vec::<u8>::with_capacity(size as usize);
    let pointer = bytes.as_mut_ptr() as i32;
    std::mem::forget(bytes);
    pointer
}

#[no_mangle]
pub extern "C" fn execute(_pointer: i32, _length: i32) -> i64 {
    0
}
RUST
cat > "$FIXTURE/skill.toml" <<'TOML'
[skill]
name = "weather-check"
version = "0.1.0"
description = "Hyphenated package regression"

[runtime]
type = "wasm"
entry = "weather_check.wasm"
TOML

cargo build --quiet --release --target wasm32-unknown-unknown --manifest-path "$FIXTURE/Cargo.toml"
WASM="$FIXTURE/target/wasm32-unknown-unknown/release/weather_check.wasm"
[[ -f "$WASM" ]] || {
  echo "FAIL: Cargo did not emit normalized artifact $WASM" >&2
  exit 1
}

ARCHIVE="$FIXTURE/weather-check.lf-skill.zip"
cargo run --quiet --manifest-path "$REPO_ROOT/tools/forge-rs/Cargo.toml" -p forge-cli -- \
  package-librefang "$FIXTURE" --no-build --output "$ARCHIVE"
unzip -Z1 "$ARCHIVE" | grep -Fxq 'skill.toml'
unzip -Z1 "$ARCHIVE" | grep -Fxq 'weather_check.wasm'
if unzip -Z1 "$ARCHIVE" | grep -Fxq 'weather-check.wasm'; then
  echo "FAIL: package contains the non-existent hyphenated Cargo artifact" >&2
  exit 1
fi

echo "PASS: hyphenated LibreFang skill renders, builds, and packages as weather_check.wasm"
