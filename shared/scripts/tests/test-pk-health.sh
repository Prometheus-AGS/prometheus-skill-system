#!/usr/bin/env bash
# test-pk-health.sh — fixture tests for shared/scripts/pk-health.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
HEALTH="$SCRIPT_DIR/../pk-health.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); }
bad() { FAIL=$((FAIL+1)); printf 'FAIL: %s — %s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
export HOME="$TMP/home"; mkdir -p "$HOME"

# 1. No pk on PATH → silent exit 0
out="$(PATH="/usr/bin:/bin" bash "$HEALTH" 2>/dev/null; echo "rc=$?")"
echo "$out" | grep -q '^rc=0$' || bad "no pk → rc 0" "$out"
[ "$(echo "$out" | grep -vc 'rc=')" = "0" ] && ok || bad "no pk → silent" "$out"

# 2. Fake pk present → prints a health line and writes the throttle marker
BIN="$TMP/bin"; mkdir -p "$BIN"
cat > "$BIN/pk" <<'FAKE'
#!/usr/bin/env bash
[ "$1" = "lint" ] || exit 64
[ "$#" -eq 1 ] || exit 65
echo "312 articles indexed, 0 issues"
FAKE
chmod +x "$BIN/pk"
out="$(PATH="$BIN:/usr/bin:/bin" bash "$HEALTH" 2>/dev/null)"
echo "$out" | grep -q 'pk health:' && ok || bad "fake pk → prints health line" "$out"
[ -f "$HOME/.prometheus/pk-health-last-run" ] && ok || bad "throttle marker written"

# 3. 24h throttle: a second immediate run is silent (marker is fresh)
out="$(PATH="$BIN:/usr/bin:/bin" bash "$HEALTH" 2>/dev/null)"
[ -z "$out" ] && ok || bad "second run within 24h should be throttled (silent)" "$out"

# 4. Marker older than 24h → runs again
echo "$(( $(date -u +%s) - 90000 ))" > "$HOME/.prometheus/pk-health-last-run"
out="$(PATH="$BIN:/usr/bin:/bin" bash "$HEALTH" 2>/dev/null)"
echo "$out" | grep -q 'pk health:' && ok || bad "stale marker → runs again" "$out"

# 5. A failed lint command is visible and must never be translated into OK.
echo '#!/usr/bin/env bash' > "$BIN/pk"
echo 'exit 42' >> "$BIN/pk"
chmod +x "$BIN/pk"
echo "$(( $(date -u +%s) - 90000 ))" > "$HOME/.prometheus/pk-health-last-run"
out="$(PATH="$BIN:/usr/bin:/bin" bash "$HEALTH" 2>/dev/null)"
echo "$out" | grep -q 'pk health: FAIL' && ok || bad "lint failure → visible failure" "$out"
echo "$out" | grep -q 'pk health: OK' && bad "lint failure must not false-green" "$out" || ok

echo "---"; echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
