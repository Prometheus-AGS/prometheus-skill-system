#!/usr/bin/env bash
# build.sh — rebuild skill.wasm from source and place it beside SKILL.md.
#
# Exit: 0 built and verified · 1 prerequisite missing · 2 build/verification failed
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

REPO_ROOT="$(git rev-parse --show-toplevel)" || exit 1
EXPECTED="$(python3 - "$REPO_ROOT/substrate/exec-tier-w/versions.toml" <<'PY'
import sys, tomllib
with open(sys.argv[1], "rb") as stream:
    print(tomllib.load(stream)["reference_component"]["artifact_sha256"])
PY
)" || exit 1
BUILD_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/prometheus-component-build.XXXXXX")" || exit 1
trap 'rm -rf "$BUILD_ROOT"' EXIT
export SOURCE_DATE_EPOCH=0
export CARGO_TARGET_DIR="$BUILD_ROOT/target"

cargo component build --release --target wasm32-wasip2 --locked || exit 2
W="$CARGO_TARGET_DIR/wasm32-wasip2/release/entity_graph_optimize_skill.wasm"
[ -f "$W" ] || { echo "[build] no .wasm produced at $W" >&2; exit 2; }

ACTUAL="$(shasum -a 256 "$W" | awk '{print $1}')"
[ "$ACTUAL" = "$EXPECTED" ] || {
  echo "[build] deterministic hash drift: expected $EXPECTED, got $ACTUAL" >&2
  exit 2
}

cp "$W" "$HERE/../skill.wasm"
echo "[build] $ACTUAL $(wc -c < "$HERE/../skill.wasm" | tr -d ' ') bytes -> skill.wasm"
