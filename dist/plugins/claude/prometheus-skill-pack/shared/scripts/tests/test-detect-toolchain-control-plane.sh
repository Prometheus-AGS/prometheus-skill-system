#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DETECT="$SCRIPT_DIR/../detect-toolchain.sh"
TEST_HOME="$(mktemp -d)"
trap 'rm -rf "$TEST_HOME"' EXIT

output="$(HOME="$TEST_HOME" SOVEREIGN_SYNC_SOCKET="$TEST_HOME/missing.sock" bash "$DETECT" --json)"
printf '%s' "$output" | node -e '
const fs = require("fs");
const data = JSON.parse(fs.readFileSync(0, "utf8"));
const row = data["control-plane-extension"];
if (!row) throw new Error("missing control-plane-extension row");
if (row.status !== "disabled") throw new Error(`expected disabled, got ${row.status}`);
if (data["sovereign-sync-daemon"]) throw new Error("stale sovereign-sync-daemon row");
'

text_output="$(HOME="$TEST_HOME" SOVEREIGN_SYNC_SOCKET="$TEST_HOME/missing.sock" bash "$DETECT")"
printf '%s' "$text_output" | grep -q 'control-plane extension (optional)'
if printf '%s' "$text_output" | grep -qi 'sovereign-sync daemon'; then
    echo "toolchain output retained the removed daemon label" >&2
    exit 1
fi

echo "Optional control-plane discovery reports absence without a warning: PASS"
