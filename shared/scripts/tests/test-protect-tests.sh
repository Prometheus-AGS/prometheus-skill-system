#!/usr/bin/env bash
# test-protect-tests.sh — fixture tests for shared/scripts/protect-tests.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
GUARD="$SCRIPT_DIR/../protect-tests.sh"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); }
bad() { FAIL=$((FAIL+1)); printf 'FAIL: %s (rc=%s)\n%s\n' "$1" "${2:-}" "${3:-}" >&2; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
unset PROMETHEUS_ALLOW_TEST_EDITS

# --- Fixture project with a BDD suite ---
ROOT="$TMP/proj"
mkdir -p "$ROOT/tests/steps" "$ROOT/tests/support" "$ROOT/tests/features/drafts"
echo "Feature: existing" > "$ROOT/tests/features/login.feature"
echo "export {}" > "$ROOT/tests/steps/login.steps.ts"
echo "export {}" > "$ROOT/tests/support/world.ts"

call() { # <tool> <path>  → prints rc
  printf '{"tool_name":"%s","tool_input":{"file_path":"%s"}}' "$1" "$2" | bash "$GUARD" >/dev/null 2>&1
  echo $?
}

# 1. Edit existing .feature → blocked (rc 2)
[ "$(call Edit "$ROOT/tests/features/login.feature")" = "2" ] && ok || bad "edit existing feature should block"
# 2. Edit existing .steps.ts → blocked
[ "$(call Edit "$ROOT/tests/steps/login.steps.ts")" = "2" ] && ok || bad "edit existing steps should block"
# 3. Edit existing support → blocked
[ "$(call MultiEdit "$ROOT/tests/support/world.ts")" = "2" ] && ok || bad "edit existing support should block"
# 4. New draft feature → allowed
[ "$(call Write "$ROOT/tests/features/drafts/new.feature")" = "0" ] && ok || bad "draft feature should pass"
# 5. New (non-existent) step file → allowed
[ "$(call Write "$ROOT/tests/steps/brand-new.steps.ts")" = "0" ] && ok || bad "new step file should pass"
# 6. Non-test file → allowed
[ "$(call Edit "$ROOT/src/app.ts")" = "0" ] && ok || bad "non-test edit should pass"
# 7. Override env bypasses
rc="$(PROMETHEUS_ALLOW_TEST_EDITS=1 bash -c 'printf "{\"tool_name\":\"Edit\",\"tool_input\":{\"file_path\":\"'"$ROOT"'/tests/features/login.feature\"}}" | bash "'"$GUARD"'" >/dev/null 2>&1; echo $?')"
[ "$rc" = "0" ] && ok || bad "override env should bypass" "$rc"
# 8. Project without tests/features → no-op pass
mkdir -p "$TMP/nobdd/src"
[ "$(call Edit "$TMP/nobdd/src/x.ts")" = "0" ] && ok || bad "no-bdd project should pass"
# 9. Empty stdin → pass
rc="$(printf '' | bash "$GUARD" >/dev/null 2>&1; echo $?)"
[ "$rc" = "0" ] && ok || bad "empty stdin should pass" "$rc"

echo "---"; echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
