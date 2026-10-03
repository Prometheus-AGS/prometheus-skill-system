#!/usr/bin/env bash
# test-scope-guard.sh — fixture tests for scope-guard.sh + scope-record.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GUARD="$SCRIPT_DIR/../scope-guard.sh"
RECORD="$SCRIPT_DIR/../scope-record.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); }
bad() { FAIL=$((FAIL+1)); printf 'FAIL: %s — %s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
ROOT="$TMP/proj"
mkdir -p "$ROOT/.kbd-orchestrator" "$ROOT/src" "$ROOT/other"
cat > "$ROOT/.kbd-orchestrator/current-waypoint.json" <<'JSON'
{ "phase": "p", "change": "change-x", "scoped_paths": ["src/**", "docs/api.md"] }
JSON

# guard <mode> <abs-path> → prints guard stdout plus a trailing rc= line
guard() {
  local mode="$1" path="$2" out rc
  out="$(cd "$ROOT" && printf '{"tool_input":{"file_path":"%s"}}' "$path" \
    | PROMETHEUS_SCOPE_ENFORCE="$mode" bash "$GUARD" 2>/dev/null)"
  rc=$?
  printf '%s\nrc=%s\n' "$out" "$rc"
}

# record <abs-path> — run scope-record from within ROOT
record() {
  ( cd "$ROOT" && printf '{"tool_input":{"file_path":"%s"}}' "$1" | bash "$RECORD" >/dev/null 2>&1 )
}

# 1. In-scope (src/**) → exit 0, no permissionDecision/NOTICE
mkdir -p "$ROOT/src"
out="$(guard warn "$ROOT/src/app.ts")"
echo "$out" | grep -q '^rc=0$' \
  && ! echo "$out" | grep -qE 'permissionDecision|NOTICE' \
  && ok || bad "in-scope should pass silently" "$out"

# 2. Out-of-scope, warn mode → exit 0 (non-blocking), but no JSON on stdout
out="$(guard warn "$ROOT/other/x.ts")"
echo "$out" | grep -q '^rc=0$' || bad "warn out-of-scope must not block" "$out"
echo "$out" | grep -q 'permissionDecision' && bad "warn mode must not emit ask-JSON" "$out" || ok

# 3. Out-of-scope, ask mode → emits ask-JSON on stdout, exit 0
out="$(guard ask "$ROOT/other/x.ts")"
echo "$out" | grep -q '"permissionDecision":"ask"' && ok || bad "ask mode should emit ask-JSON" "$out"

# 4. off mode → exit 0 always, no notice
out="$(guard off "$ROOT/other/x.ts")"
echo "$out" | grep -q '^rc=0$' && ! echo "$out" | grep -q 'NOTICE' && ok || bad "off mode silent pass" "$out"

# 5. Always-allowed: .kbd-orchestrator/** even when out of scoped_paths
out="$(guard ask "$ROOT/.kbd-orchestrator/foo.json")"
echo "$out" | grep -q 'permissionDecision' && bad "orchestrator path must be allowed" "$out" || ok

# 6. No active change → pass
cat > "$ROOT/.kbd-orchestrator/current-waypoint.json" <<'JSON'
{ "phase": "p", "scoped_paths": ["src/**"] }
JSON
out="$(guard ask "$ROOT/other/x.ts")"
echo "$out" | grep -q 'permissionDecision' && bad "no-change must pass" "$out" || ok

# 7. scope-record adds an override for an out-of-scope write, idempotently
cat > "$ROOT/.kbd-orchestrator/current-waypoint.json" <<'JSON'
{ "phase": "p", "change": "change-x", "scoped_paths": ["src/**"] }
JSON
record "$ROOT/other/x.ts"
jq -e '.scope_overrides | length == 1 and .[0].path == "other/x.ts"' "$ROOT/.kbd-orchestrator/current-waypoint.json" >/dev/null \
  && ok || bad "scope-record should add one override" "$(jq -c .scope_overrides "$ROOT/.kbd-orchestrator/current-waypoint.json")"
# idempotent: second record does not duplicate
record "$ROOT/other/x.ts"
jq -e '.scope_overrides | length == 1' "$ROOT/.kbd-orchestrator/current-waypoint.json" >/dev/null \
  && ok || bad "scope-record must be idempotent" "$(jq -c .scope_overrides "$ROOT/.kbd-orchestrator/current-waypoint.json")"

# 8. After recording, guard treats the path as allowed (ask mode → no JSON)
out="$(guard ask "$ROOT/other/x.ts")"
echo "$out" | grep -q 'permissionDecision' && bad "recorded override should silence guard" "$out" || ok

echo "---"; echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
