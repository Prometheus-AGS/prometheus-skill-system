#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
TEST_ROOT="$(mktemp -d)"
trap 'rm -rf "$TEST_ROOT"' EXIT

PROJECT_ROOT="$TEST_ROOT/focus-project"
OUTPUT_ROOT="$TEST_ROOT/rendered"
mkdir -p "$PROJECT_ROOT/.kbd-orchestrator"

bash "$REPO_ROOT/scripts/install-mcp-services.sh" \
    --kbd-focus-project "$PROJECT_ROOT" \
    --render-only "$OUTPUT_ROOT"

CANONICAL_PROJECT_ROOT="$(cd "$PROJECT_ROOT" && pwd -P)"
grep -F "<key>KBD_FOCUS_PROJECT_PATH</key>" \
    "$OUTPUT_ROOT/ai.prometheus.sovereign-sync.plist" >/dev/null
grep -F "<string>$CANONICAL_PROJECT_ROOT</string>" \
    "$OUTPUT_ROOT/ai.prometheus.sovereign-sync.plist" >/dev/null
test "$(grep -cF "<string>$CANONICAL_PROJECT_ROOT</string>" \
    "$OUTPUT_ROOT/ai.prometheus.sovereign-sync.plist")" -eq 2
grep -F "WorkingDirectory=$CANONICAL_PROJECT_ROOT" \
    "$OUTPUT_ROOT/ai.prometheus.sovereign-sync.service" >/dev/null
grep -F "Environment=KBD_FOCUS_PROJECT_PATH=$CANONICAL_PROJECT_ROOT" \
    "$OUTPUT_ROOT/ai.prometheus.sovereign-sync.service" >/dev/null

if bash "$REPO_ROOT/scripts/install-mcp-services.sh" \
    --kbd-focus-project "$TEST_ROOT/not-a-project" \
    --render-only "$TEST_ROOT/invalid-output" >/dev/null 2>&1; then
    echo "installer accepted a focus root without .kbd-orchestrator" >&2
    exit 1
fi

echo "kbd focus service rendering: PASS"
