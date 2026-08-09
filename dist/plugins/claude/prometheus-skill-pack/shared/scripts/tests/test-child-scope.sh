#!/usr/bin/env bash
# test-child-scope.sh — fixture tests for check-child-scope.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
GUARD="$REPO_ROOT/skills/process/kbd-process-orchestrator/shared/lib/check-child-scope.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); }
bad() { FAIL=$((FAIL+1)); printf 'FAIL: %s — %s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
ROOT="$TMP/proj"
CHILD=".kbd-orchestrator/phases/p/children/alpha"
mkdir -p "$ROOT/$CHILD" "$ROOT/src/feature" "$ROOT/src/other"
# Child scope: only allowed to write under src/feature/**
cat > "$ROOT/$CHILD/scope.json" <<JSON
{ "allowedWritePaths": ["src/feature/**", "$CHILD/**"], "deniedPaths": [], "inheritsConstraints": true }
JSON

set_waypoint() { # <path-json>
  cat > "$ROOT/.kbd-orchestrator/current-waypoint.json" <<JSON
{ "phase": "p", "path": $1, "childPointer": null }
JSON
}

guard() { # <mode> <abs-path>
  local mode="$1" path="$2" out rc
  out="$(cd "$ROOT" && printf '{"tool_input":{"file_path":"%s"}}' "$path" \
    | PROMETHEUS_CHILD_SCOPE_ENFORCE="$mode" bash "$GUARD" 2>/dev/null)"
  rc=$?
  printf '%s\nrc=%s\n' "$out" "$rc"
}

# --- Inside child alpha (path depth 2) ---
set_waypoint '["p","alpha"]'

# 1. In-scope write (src/feature) → silent pass
out="$(guard warn "$ROOT/src/feature/x.ts")"
echo "$out" | grep -q '^rc=0$' && ! echo "$out" | grep -qE 'NOTICE|permissionDecision' && ok \
  || bad "in-scope should pass silently" "$out"

# 2. Out-of-scope (src/other), warn → notice, exit 0, no JSON
out="$(guard warn "$ROOT/src/other/y.ts")"
echo "$out" | grep -q '^rc=0$' || bad "warn must not block" "$out"
echo "$out" | grep -q 'permissionDecision' && bad "warn must not emit ask-JSON" "$out" || ok

# 3. Out-of-scope, ask → ask-JSON
out="$(guard ask "$ROOT/src/other/y.ts")"
echo "$out" | grep -q '"permissionDecision":"ask"' && ok || bad "ask should emit ask-JSON" "$out"

# 4. Child's own dir always allowed
out="$(guard ask "$ROOT/$CHILD/notes.md")"
echo "$out" | grep -q 'permissionDecision' && bad "child's own dir must be allowed" "$out" || ok

# 5. .kbd-orchestrator/** always allowed
out="$(guard ask "$ROOT/.kbd-orchestrator/foo.json")"
echo "$out" | grep -q 'permissionDecision' && bad "orchestrator path must be allowed" "$out" || ok

# 6. off mode → never acts
out="$(guard off "$ROOT/src/other/y.ts")"
echo "$out" | grep -qE 'NOTICE|permissionDecision' && bad "off mode must be silent" "$out" || ok

# --- At top level (path depth 1) → hook is a no-op ---
set_waypoint '["p"]'
out="$(guard ask "$ROOT/src/other/y.ts")"
echo "$out" | grep -q 'permissionDecision' && bad "top-level writes must never be flagged" "$out" || ok

echo "---"; echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
