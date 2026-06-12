#!/usr/bin/env bash
# protect-tests.sh — PreToolUse(Write|Edit|MultiEdit) guard for the BDD
# Immutable-Tests Rule (CLAUDE.md → "BDD Immutable-Tests Rule"; BDD-006).
#
# Code-generation agents may NOT edit existing Cucumber/BDD step definitions,
# support files, or feature files to make failing tests pass. They MAY add new
# feature files under tests/features/drafts/ and brand-new step files.
#
# Contract: reads PreToolUse JSON from stdin. Exit 2 (with stderr guidance)
# blocks a forbidden edit; exit 0 allows. Degrades to exit 0 when the project
# has no tests/features/ directory or the input is unreadable.
set -uo pipefail

HOOK_LOG_LIB="$(cd "$(dirname "$0")" && pwd)/lib/hook-log.sh"
[ -f "$HOOK_LOG_LIB" ] && source "$HOOK_LOG_LIB"
hook_log_start "PreToolUse" "protect-tests.sh"

# Shared path-scope helper (canonicalize + relativize, macOS symlink-safe).
PSCOPE_LIB="$(cd "$(dirname "$0")" && pwd)/lib/path-scope.sh"
[ -f "$PSCOPE_LIB" ] && source "$PSCOPE_LIB"

finish() { hook_log_end "${1:-0}"; exit "${1:-0}"; }

# Global override.
[ "${PROMETHEUS_ALLOW_TEST_EDITS:-}" = "1" ] && {
  echo "[protect-tests] WARN: PROMETHEUS_ALLOW_TEST_EDITS=1 — guard bypassed" >&2
  finish 0
}

command -v python3 >/dev/null 2>&1 || finish 0
INPUT="$(cat 2>/dev/null || true)"
[ -n "$INPUT" ] || finish 0

# Extract tool name + target path from the PreToolUse payload.
read -r TOOL_NAME FILE_PATH <<EOF
$(printf '%s' "$INPUT" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
except Exception:
    print('  '); raise SystemExit
tool = d.get('tool_name', '')
ti = d.get('tool_input', {}) or {}
fp = ti.get('file_path') or ti.get('path') or ''
print(tool, fp)
" 2>/dev/null || printf '  ')
EOF

[ -n "${FILE_PATH:-}" ] || finish 0

# Locate the repo/project root the path belongs to (walk up for tests/features).
_find_tests_root() {
  local dir
  dir="$(cd "$(dirname "$FILE_PATH")" 2>/dev/null && pwd)" || dir="$PWD"
  while [ -n "$dir" ] && [ "$dir" != "/" ]; do
    [ -d "$dir/tests/features" ] && { printf '%s' "$dir"; return 0; }
    dir="$(dirname "$dir")"
  done
  return 1
}
ROOT="$(_find_tests_root)" || finish 0   # no BDD suite → not applicable

# Normalize the target to a path relative to ROOT (shared helper when present;
# fall back to a plain prefix strip in a stripped environment).
if command -v pscope_relativize >/dev/null 2>&1; then
  REL="$(pscope_relativize "$ROOT" "$FILE_PATH")"
else
  REL="$FILE_PATH"
  case "$FILE_PATH" in "$ROOT"/*) REL="${FILE_PATH#"$ROOT"/}" ;; esac
fi

# Drafts and brand-new files are always allowed.
case "$REL" in
  tests/features/drafts/*) finish 0 ;;
esac

# Is this a protected BDD artifact?
_is_protected() {
  case "$1" in
    tests/steps/*.steps.ts|tests/support/*.ts|tests/features/*.feature) return 0 ;;
    *) return 1 ;;
  esac
}
_is_protected "$REL" || finish 0

# Writing a NEW file (path does not yet exist) is allowed — only mutating an
# EXISTING protected file is blocked. (Write to an existing path is a mutation.)
if [ ! -e "$FILE_PATH" ]; then
  finish 0
fi

cat >&2 <<EOF

[protect-tests] BLOCKED — BDD Immutable-Tests Rule (CLAUDE.md / BDD-006).

  Target: ${REL}

  Existing step definitions, support files, and feature files may not be
  edited by code-generation agents to make failing tests pass.

  Instead:
    - Add a NEW feature under tests/features/drafts/ and matching new step defs.
    - Surface the failing test to the user rather than rewriting the step.

  To override deliberately: PROMETHEUS_ALLOW_TEST_EDITS=1

EOF
finish 2
