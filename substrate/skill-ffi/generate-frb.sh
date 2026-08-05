#!/usr/bin/env bash
# Regenerate the checked-in Rust dispatcher that retains the complete mobile API.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
CODEGEN_VERSION="2.12.0"
MODE="${1:-write}"
[ "$MODE" = "write" ] || [ "$MODE" = "--check" ] || {
  echo "usage: $0 [--check]" >&2
  exit 1
}

command -v flutter_rust_bridge_codegen >/dev/null 2>&1 || {
  echo "[ffi] flutter_rust_bridge_codegen $CODEGEN_VERSION is required" >&2
  exit 1
}
actual_version="$(flutter_rust_bridge_codegen --version | awk '{print $2}')"
[ "$actual_version" = "$CODEGEN_VERSION" ] || {
  echo "[ffi] expected flutter_rust_bridge_codegen $CODEGEN_VERSION, found $actual_version" >&2
  exit 1
}

scratch="$(mktemp -d "${TMPDIR:-/tmp}/prometheus-skill-ffi.XXXXXX")"
generated="$HERE/src/frb_generated.rs"
if [ "$MODE" = "--check" ]; then
  generated_output="$HERE/src/frb_generated_check.rs"
else
  generated_output="$generated"
fi
cleanup() {
  if [ "$MODE" = "--check" ]; then
    rm -f "$generated_output"
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT
mkdir -p "$scratch/lib"
cat >"$scratch/pubspec.yaml" <<'YAML'
name: prometheus_skill_ffi
environment:
  sdk: ">=3.0.0 <4.0.0"
dependencies:
  flutter_rust_bridge: 2.12.0
YAML

flutter_rust_bridge_codegen generate \
  --rust-root "$HERE" \
  --rust-input crate::api \
  --rust-output "$generated_output" \
  --dart-output "$scratch/lib" \
  --dart-root "$scratch" \
  --c-output "$scratch/frb_generated.h" \
  --no-add-mod-to-lib \
  --no-auto-upgrade-dependency \
  --no-build-runner \
  --no-dart-format \
  --no-web \
  --stop-on-error

if [ "$MODE" = "--check" ]; then
  # A sibling output file makes codegen emit `use crate::*`; normalize only
  # that derived module path before comparing the otherwise identical output.
  sed 's/use crate::\*;/use crate::api::*;/' "$generated_output" \
    > "$scratch/normalized.rs"
  mv "$scratch/normalized.rs" "$generated_output"
  cmp -s "$generated" "$generated_output" || {
    echo "[ffi] src/frb_generated.rs differs from deterministic codegen output" >&2
    diff -u "$generated" "$generated_output" || true
    exit 2
  }
  echo "[ffi] checked-in Rust dispatcher matches codegen $CODEGEN_VERSION"
else
  echo "[ffi] regenerated src/frb_generated.rs with flutter_rust_bridge_codegen $CODEGEN_VERSION"
fi
