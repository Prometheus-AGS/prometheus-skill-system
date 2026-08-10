#!/usr/bin/env bash
# test-path-scope.sh — tests for the shared path-scope helper.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
LIB="$SCRIPT_DIR/../lib/path-scope.sh"
# shellcheck source=/dev/null
source "$LIB"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); }
bad() { FAIL=$((FAIL+1)); printf 'FAIL: %s — %s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
ROOT="$TMP/proj"; mkdir -p "$ROOT/src/feature" "$ROOT/src/other"

# 1. relativize: absolute path under root → repo-relative (macOS /var-safe)
rel="$(pscope_relativize "$ROOT" "$ROOT/src/feature/x.ts")"
[ "$rel" = "src/feature/x.ts" ] && ok || bad "relativize under root" "$rel"

# 2. relativize: already-relative path passes through
rel="$(pscope_relativize "$ROOT" "src/already/rel.ts")"
[ "$rel" = "src/already/rel.ts" ] && ok || bad "relativize relative passthrough" "$rel"

# 3. relativize: path outside root → canonical absolute (not relativized)
rel="$(pscope_relativize "$ROOT" "/etc/hosts")"
case "$rel" in /*) ok ;; *) bad "outside-root should stay absolute" "$rel" ;; esac

# 4. match: exact glob match, NO over-broadening (the recurring bug)
[ "$(pscope_match "src/feature/x.ts" '["src/feature/**"]')" = "in" ] && ok || bad "feature glob matches feature path"
[ "$(pscope_match "src/other/y.ts" '["src/feature/**"]')" = "out" ] && ok || bad "feature glob must NOT match other path"

# 5. match: multiple globs, any match → in
[ "$(pscope_match "docs/api.md" '["src/**","docs/api.md"]')" = "in" ] && ok || bad "exact file glob matches"
[ "$(pscope_match "lib/z.ts" '["src/**","docs/api.md"]')" = "out" ] && ok || bad "no glob matches → out"

# 6. match: empty globs → out
[ "$(pscope_match "anything" '[]')" = "out" ] && ok || bad "empty globs → out"

# 7. always_allowed
pscope_always_allowed ".kbd-orchestrator/x.json" && ok || bad "orchestrator path allowed"
pscope_always_allowed "SCRATCHPAD.md" && ok || bad "scratchpad allowed"
pscope_always_allowed "src/app.ts" && bad "src must NOT be always-allowed" || ok

echo "---"; echo "PASS=$PASS FAIL=$FAIL"
[ "$FAIL" -eq 0 ]
