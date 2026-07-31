#!/usr/bin/env bash
# build.sh — rebuild skill.wasm from source and place it beside SKILL.md.
#
# Exit: 0 built and validated · 1 prerequisite missing · 2 build/validation failed
#
# The committed skill.wasm is a build artifact. It is checked in because UAR
# discovers `skill.wasm` beside `SKILL.md` in this repo as a submodule — a
# consumer must not need a Rust toolchain to get it. This script is how it is
# regenerated, so the binary is never the only copy of the truth.
set -uo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
cd "$HERE" || exit 1

command -v cargo-component >/dev/null 2>&1 || {
  echo "[build] cargo-component required: cargo install cargo-component" >&2; exit 1; }
rustup target list --installed 2>/dev/null | grep -q wasm32-wasip2 || {
  echo "[build] rustup target add wasm32-wasip2" >&2; exit 1; }

cargo component build --release --target wasm32-wasip2 || exit 2
W="$(find target -name 'entity_graph_optimize_skill.wasm' | head -1)"
[ -n "$W" ] || { echo "[build] no .wasm produced" >&2; exit 2; }

wasm-tools validate --features component-model "$W" || {
  echo "[build] artifact is not a valid component" >&2; exit 2; }

cp "$W" "$HERE/../skill.wasm"
echo "[build] $(wc -c < "$HERE/../skill.wasm" | tr -d ' ') bytes -> skill.wasm"
