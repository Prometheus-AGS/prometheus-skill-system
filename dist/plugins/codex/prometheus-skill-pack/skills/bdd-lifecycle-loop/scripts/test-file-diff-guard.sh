#!/usr/bin/env bash
# Compatibility wrapper for the local, method-independent Git-state verifier.
# Agent tools remain unrestricted; approval is evaluated only at certification.
set -euo pipefail

BASE_REF="${1:?base ref required}"
HEAD_REF="${2:-HEAD}"
REPOSITORY="$(git rev-parse --show-toplevel)"

exec node "$REPOSITORY/scripts/verify-protected-tests.mjs" \
    --base "$BASE_REF" \
    --candidate "$HEAD_REF" \
    "${@:3}"
