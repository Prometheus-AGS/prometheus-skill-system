#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT

OUTPUT_ROOT="$TEST_ROOT/rendered"
bash "$REPO_ROOT/scripts/install-mcp-services.sh" --render-only "$OUTPUT_ROOT" >/dev/null

if find "$OUTPUT_ROOT" -type f -iname '*sovereign-sync*' -print -quit | grep -q .; then
    echo "pack installer rendered a Companion-owned service definition" >&2
    exit 1
fi

if bash "$REPO_ROOT/scripts/install-mcp-services.sh" --sharing \
    --render-only "$TEST_ROOT/invalid-output" >/dev/null 2>&1; then
    echo "installer accepted the removed --sharing option" >&2
    exit 1
fi

if bash "$REPO_ROOT/scripts/install-binaries.sh" --sharing --dry-run >/dev/null 2>&1; then
    echo "binary installer accepted the removed --sharing option" >&2
    exit 1
fi

echo "Companion-owned service definitions are absent from pack installers: PASS"
