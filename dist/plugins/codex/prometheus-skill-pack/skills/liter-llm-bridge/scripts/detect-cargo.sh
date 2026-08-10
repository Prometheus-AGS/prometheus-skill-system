#!/usr/bin/env bash
# detect-cargo.sh — verify the Rust toolchain is present.
# Skill pack precondition guarantees cargo; this is a sanity check.
# Output: JSON with `present` boolean and `version` (or null).
# Exit 0 if cargo is on PATH, exit 1 otherwise.

set -euo pipefail

if command -v cargo >/dev/null 2>&1; then
  VERSION="$(cargo --version 2>/dev/null | awk '{print $2}')"
  printf '{"present": true, "version": "%s", "path": "%s"}\n' \
    "$VERSION" "$(command -v cargo)"
  exit 0
else
  printf '{"present": false, "version": null, "path": null, "hint": "Install Rust via https://rustup.rs"}\n'
  exit 1
fi
