#!/usr/bin/env bash
# path-scope.sh — ONE verified path-scope-matching helper for all guard hooks.
# Source this file; no import side effects.
#
#   source "$(dirname "$0")/lib/path-scope.sh"
#   rel="$(pscope_relativize "$ROOT" "$FILE_PATH")"
#   pscope_always_allowed "$rel" && allow
#   [ "$(pscope_match "$rel" "$GLOBS_JSON")" = "in" ] && allow
#
# Consolidates the canonicalize + repo-relativize + fnmatch logic that
# scope-guard, check-child-scope, and protect-tests each re-implemented (and
# each got subtly wrong — macOS /var symlinks, glob over-broadening). Centralize
# it once, test it once.

# pscope_relativize <root> <file>
# Canonicalize both via `cd && pwd -P` (macOS /var vs /private/var safe) and
# echo <file> made relative to <root>. When <file> is outside <root> (or
# already relative), echo it unchanged.
pscope_relativize() {
  local root="$1" file="$2"
  _pscope_canon() { ( cd "$1" 2>/dev/null && pwd -P ) || printf '%s' "$1"; }
  local root_real fp_real
  root_real="$(_pscope_canon "$root")"
  case "$file" in
    /*) fp_real="$(_pscope_canon "$(dirname "$file")")/$(basename "$file")" ;;
    *)  printf '%s' "$file"; return 0 ;;   # already relative
  esac
  case "$fp_real" in
    "$root_real"/*) printf '%s' "${fp_real#"$root_real"/}" ;;
    *)              printf '%s' "$fp_real" ;;
  esac
}

# pscope_always_allowed <rel>
# True (exit 0) for paths every guard treats as in-scope: orchestrator state
# and the session scratchpad.
pscope_always_allowed() {
  case "$1" in
    .kbd-orchestrator/*|SCRATCHPAD.md) return 0 ;;
    *) return 1 ;;
  esac
}

# pscope_match <rel> <globs-json>
# Echo "in" when <rel> matches any glob in the JSON array, else "out". Uses
# python fnmatch over the globs AS-IS (no prefix-stripping — that is the
# over-broadening bug that turned an allowed path into "**"). Fails open to
# "in" only on a hard interpreter error (guards are advisory; a crash must not
# block flow), but a well-formed empty/no-match returns "out".
pscope_match() {
  local rel="$1" globs="$2"
  command -v python3 >/dev/null 2>&1 || { printf 'in'; return 0; }
  REL="$rel" GLOBS="$globs" python3 -c '
import os, json, fnmatch
rel = os.environ.get("REL", "")
try:
    globs = json.loads(os.environ.get("GLOBS", "[]"))
except Exception:
    print("out"); raise SystemExit
for g in globs:
    if fnmatch.fnmatch(rel, g):
        print("in"); raise SystemExit
print("out")
' 2>/dev/null || printf 'in'
}
