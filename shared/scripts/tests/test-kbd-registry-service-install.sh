#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT

OUTPUT_ROOT="$TEST_ROOT/rendered"
bash "$REPO_ROOT/scripts/install-mcp-services.sh" \
    --render-only "$OUTPUT_ROOT"

PLIST="$OUTPUT_ROOT/ai.prometheus.sovereign-sync.plist"
SERVICE="$OUTPUT_ROOT/ai.prometheus.sovereign-sync.service"

python3 - "$PLIST" <<'PY'
import plistlib
import sys

with open(sys.argv[1], "rb") as handle:
    document = plistlib.load(handle)
environment = document["EnvironmentVariables"]
assert not any("FOCUS_PROJECT" in key for key in environment)
assert document["WorkingDirectory"] == environment["HOME"]
PY

if grep -F "FOCUS_PROJECT" "$SERVICE" >/dev/null; then
    echo "systemd definition retained obsolete focus-project configuration" >&2
    exit 1
fi

SYSTEMD_HOME="$(sed -n 's/^Environment=HOME=//p' "$SERVICE")"
grep -F "WorkingDirectory=\"$SYSTEMD_HOME\"" "$SERVICE" >/dev/null

if bash "$REPO_ROOT/scripts/install-mcp-services.sh" \
    --kbd-"focus-project" "$TEST_ROOT/obsolete" \
    --render-only "$TEST_ROOT/invalid-output" >/dev/null 2>&1; then
    echo "installer accepted the removed project-focus option" >&2
    exit 1
fi

echo "kbd registry service rendering: PASS"
