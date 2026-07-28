#!/usr/bin/env bash
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HOOK="$SCRIPT_DIR/../position-on-prompt.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PASS=0
FAIL=0
ok() { PASS=$((PASS + 1)); }
bad() { FAIL=$((FAIL + 1)); printf 'FAIL: %s\n%s\n' "$1" "${2:-}" >&2; }

mkdir -p "$TMP/repo/.kbd-orchestrator"
cat > "$TMP/repo/.kbd-orchestrator/current-waypoint.json" <<'JSON'
{"phase":"phase-x","status":"running","exactNextCommand":"/kbd-apply c1"}
JSON

OUT="$(cd "$TMP/repo" && printf '{}' | bash "$HOOK")"
printf '%s' "$OUT" | grep -q 'POSITION ADVISORY' && ok || bad "running state has advisory" "$OUT"
if printf '%s' "$OUT" | grep -q 'MANDATORY:'; then bad "running advisory must not mandate continuation" "$OUT"; else ok; fi

jq '.status = "paused"' "$TMP/repo/.kbd-orchestrator/current-waypoint.json" > "$TMP/wp.tmp"
mv "$TMP/wp.tmp" "$TMP/repo/.kbd-orchestrator/current-waypoint.json"
OUT="$(cd "$TMP/repo" && printf '{}' | bash "$HOOK")"
printf '%s' "$OUT" | grep -q 'execution is suspended' && ok || bad "paused state suppresses steering" "$OUT"

jq '.status = "running"' "$TMP/repo/.kbd-orchestrator/current-waypoint.json" > "$TMP/wp.tmp"
mv "$TMP/wp.tmp" "$TMP/repo/.kbd-orchestrator/current-waypoint.json"
: > "$TMP/repo/.kbd-orchestrator/PAUSE"
OUT="$(cd "$TMP/repo" && printf '{}' | bash "$HOOK")"
printf '%s' "$OUT" | grep -q 'execution is suspended' && ok || bad "emergency PAUSE suppresses steering" "$OUT"

echo "---"
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
