#!/usr/bin/env bash
# test-detect-toolchain-sovereign-sync.sh — sovereign-sync daemon fixtures.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DETECT="$SCRIPT_DIR/../detect-toolchain.sh"

PASS=0
FAIL=0

bad() {
    FAIL=$((FAIL + 1))
    printf 'FAIL: %s — %s\n' "$1" "${2:-}" >&2
}

ok() {
    PASS=$((PASS + 1))
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

BASE_PATH="$PATH"
BIN="$TMP/bin"
mkdir -p "$BIN"
cat > "$BIN/sovereign-sync" <<'FAKE'
#!/usr/bin/env bash
case "${SOVEREIGN_SYNC_FIXTURE:-missing}" in
  healthy)
    printf '{"status":"healthy","port":7892,"endpoint":"http://127.0.0.1:7892/health","message":"ok"}\n'
    exit 0
    ;;
  missing)
    printf '{"status":"missing","port":7892,"endpoint":"http://127.0.0.1:7892/health","message":"not listening"}\n'
    exit 1
    ;;
  occupied)
    printf '{"status":"occupied","port":7892,"endpoint":"http://127.0.0.1:7892/health","message":"different service"}\n'
    exit 2
    ;;
  *)
    exit 9
    ;;
esac
FAKE
chmod +x "$BIN/sovereign-sync"

run_json() {
    PATH="$BIN:$BASE_PATH" SOVEREIGN_SYNC_FIXTURE="$1" bash "$DETECT" --json
}

assert_status() {
    local fixture="$1"
    local expected="$2"
    local expected_version="$3"
    local output
    output="$(run_json "$fixture")"
    printf '%s' "$output" | node -e "
const fs = require('fs');
const data = JSON.parse(fs.readFileSync(0, 'utf8'));
const got = data['sovereign-sync-daemon'];
if (!got) throw new Error('missing sovereign-sync-daemon');
if (got.status !== '$expected') throw new Error('expected status $expected, got ' + got.status);
if (!got.version.includes('$expected_version')) throw new Error('expected version to include $expected_version, got ' + got.version);
" && ok || bad "$fixture fixture" "$output"
}

assert_status healthy ok "healthy"
assert_status missing missing "not listening"
assert_status occupied occupied "occupied"

text_output="$(PATH="$BIN:$BASE_PATH" SOVEREIGN_SYNC_FIXTURE=occupied bash "$DETECT")"
printf '%s' "$text_output" | grep -q 'sovereign-sync daemon (:7892)' \
    && printf '%s' "$text_output" | grep -q 'occupied by a different service' \
    && ok \
    || bad "occupied text output" "$text_output"

echo "---"
echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
