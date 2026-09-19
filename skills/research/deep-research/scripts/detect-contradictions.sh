#!/usr/bin/env bash
# detect-contradictions.sh — stage 06 contradiction detection (change-rah-010).
#
# Two paths, both writing the spec's contradictions.json shape
# (references/research-package-spec.md, "contradictions.json"):
#
#   numeric   the same measurable topic (percentage, latency, throughput, storage)
#             stated with values that differ by more than 2x across sources;
#             deterministic, label `inferred`.
#   semantic  candidate pairs (claims from different sources sharing content
#             words) are put to the critic model through kbd_complete; a pair
#             the judge calls contradictory is recorded with label `inferred`.
#             When no gateway is reachable every candidate pair is recorded
#             `resolved: false, label: blocked` naming the reason, so an
#             unchecked contradiction is visible (and fires on-contradiction.sh)
#             rather than silently absent.
#
# Usage:
#   bash detect-contradictions.sh <package_dir> [--semantic] [--out contradictions.json]
#   bash detect-contradictions.sh --credibility C [--registry R] --package-id ID
#                                 [--semantic] [--max-pairs 20] [--out FILE]
#
# Test seam: RESEARCH_SEMANTIC_JUDGE_CMD=<command> replaces kbd_complete; it is
# invoked as `<command> <prompt-file>` and must print the judge's JSON.
# Claim ids use the same function as build-graph.sh. bash 3.2 compatible (C-05).
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$HERE/../../../.." && pwd)"

CRED=""; REGISTRY=""; PKG_ID=""; SEMANTIC=0; MAX_PAIRS=20; OUT=""
if [ $# -gt 0 ] && [ -d "$1" ]; then
  PKG="$(cd "$1" && pwd)"; shift
  CRED="$PKG/sources/credibility.json"; REGISTRY="$PKG/sources/registry.json"
  if [ -f "$PKG/checkpoint.json" ] && command -v jq >/dev/null 2>&1; then
    PKG_ID="$(jq -r '.package_id // ""' "$PKG/checkpoint.json" 2>/dev/null || true)"
  fi
  [ -n "$PKG_ID" ] || PKG_ID="$(basename "$PKG")"
fi
while [ $# -gt 0 ]; do
  case "$1" in
    --credibility) CRED="${2:-}"; shift 2 ;;
    --registry) REGISTRY="${2:-}"; shift 2 ;;
    --package-id) PKG_ID="${2:-}"; shift 2 ;;
    --semantic) SEMANTIC=1; shift ;;
    --max-pairs) MAX_PAIRS="${2:-20}"; shift 2 ;;
    --out) OUT="${2:-}"; shift 2 ;;
    -h|--help) sed -n '2,26p' "$0"; exit 0 ;;
    *) echo "[detect-contradictions] unknown argument: $1" >&2; exit 1 ;;
  esac
done
HAVE_INPUT=0
[ -n "$CRED" ] && [ -f "$CRED" ] && HAVE_INPUT=1
[ -n "$REGISTRY" ] && [ -f "$REGISTRY" ] && HAVE_INPUT=1
[ "$HAVE_INPUT" -eq 1 ] || { echo "[detect-contradictions] --credibility or --registry (existing file) required" >&2; exit 1; }
[ -n "$PKG_ID" ] || { echo "[detect-contradictions] --package-id required" >&2; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "[detect-contradictions] python3 is required" >&2; exit 1; }

# ---- semantic judge: one function the python block calls back through a file --
# Returns 0 with the judge's text on stdout, 3 when no gateway is reachable,
# other non-zero on a gateway error.
semantic_judge() { # semantic_judge <prompt-file>
  if [ -n "${RESEARCH_SEMANTIC_JUDGE_CMD:-}" ]; then
    $RESEARCH_SEMANTIC_JUDGE_CMD "$1"; return $?
  fi
  local lib=""
  for cand in "$REPO_ROOT/shared/scripts/lib/kbd-model-resolve.sh" "${CLAUDE_PLUGIN_ROOT:-}/shared/scripts/lib/kbd-model-resolve.sh"; do
    [ -n "$cand" ] && [ -f "$cand" ] && { lib="$cand"; break; }
  done
  [ -n "$lib" ] || { echo "kbd-model-resolve.sh not found; semantic path unavailable" >&2; return 3; }
  # shellcheck source=/dev/null
  . "$lib"
  local model; model="$(kbd_resolve_role critic 2>/dev/null || echo "")"
  [ -n "$model" ] || model="kbd-critic"
  kbd_complete "$model" "You are a strict fact checker. Answer with JSON only." "$(cat "$1")" 300
}
# The python block calls the judge through a child bash that inherits this
# exported function (bash 3.2 supports export -f); nothing re-sources the script.
export -f semantic_judge
export REPO_ROOT

WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT

# The program lives beside this script rather than inside a heredoc: the
# heredoc form hangs indefinitely on some hosts (observed 2026-09-09), while the
# same program from a file runs immediately.
#
# Deliberately NOT `exec`, unlike the sibling wrappers. The semantic path calls
# back into `semantic_judge` above, which reaches the program as an *exported
# shell function* (`export -f`). `exec` replaces this shell, so the function
# would die with it and the child bash would block forever looking for it —
# which is exactly what happened when this was first written with `exec`. The
# exit code is forwarded explicitly instead.
HERE="$(cd "$(dirname "$0")" && pwd)"
PROG="$HERE/detect-contradictions.py"
[ -f "$PROG" ] || { echo "[detect-contradictions] detect-contradictions.py missing beside this script" >&2; exit 1; }

CRED="$CRED" REGISTRY="$REGISTRY" PKG_ID="$PKG_ID" SEMANTIC="$SEMANTIC" \
MAX_PAIRS="$MAX_PAIRS" OUT="$OUT" WORK="$WORK" python3 "$PROG"
exit $?
