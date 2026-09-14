#!/usr/bin/env bash
# build-graph.sh — stage 07 offline graph builder (change-rah-010).
#
# Emits graph.json in the shape of references/schemas/research-graph.schema.json:
# {topics[], claims[], relations[]}. Claim ids are content-addressed,
# `claim-` + sha256("<scope>:<normalised text>")[:16], so the same sentence
# from two artifacts collapses to ONE claim whose label is the higher of the
# two (verified > inferred > unverified > blocked) and whose sources are the
# union. Relations are `cites` (claim → source) and `contradicts` (claim ↔
# claim, from contradictions.json).
#
# Usage:
#   bash build-graph.sh <package_dir> [--out graph.json]
#       reads sources/registry.json, sources/credibility.json (labels, evidence,
#       scores), contradictions.json (optional), checkpoint.json (package_id scope)
#   bash build-graph.sh --registry R --credibility C [--contradictions X]
#                       --package-id ID [--critical FILE] [--out graph.json]
#
# --critical FILE: one claim text per line to mark `critical: true` (stage 09
# may only promote a claim that is already critical here).
#
# The claim-id function is duplicated verbatim in detect-contradictions.sh;
# tests/scoring-graph.sh asserts the two agree. bash 3.2 compatible (C-05).
set -euo pipefail

REGISTRY=""; CRED=""; CONTRA=""; PKG_ID=""; CRITICAL=""; OUT=""
if [ $# -gt 0 ] && [ -d "$1" ]; then
  PKG="$(cd "$1" && pwd)"; shift
  REGISTRY="$PKG/sources/registry.json"; CRED="$PKG/sources/credibility.json"
  [ -f "$PKG/contradictions.json" ] && CONTRA="$PKG/contradictions.json"
  if [ -f "$PKG/checkpoint.json" ] && command -v jq >/dev/null 2>&1; then
    PKG_ID="$(jq -r '.package_id // ""' "$PKG/checkpoint.json" 2>/dev/null || true)"
  fi
  [ -n "$PKG_ID" ] || PKG_ID="$(basename "$PKG")"
fi
while [ $# -gt 0 ]; do
  case "$1" in
    --registry) REGISTRY="${2:-}"; shift 2 ;;
    --credibility) CRED="${2:-}"; shift 2 ;;
    --contradictions) CONTRA="${2:-}"; shift 2 ;;
    --package-id) PKG_ID="${2:-}"; shift 2 ;;
    --critical) CRITICAL="${2:-}"; shift 2 ;;
    --out) OUT="${2:-}"; shift 2 ;;
    -h|--help) sed -n '2,24p' "$0"; exit 0 ;;
    *) echo "[build-graph] unknown argument: $1" >&2; exit 1 ;;
  esac
done
[ -n "$REGISTRY" ] && [ -f "$REGISTRY" ] || { echo "[build-graph] registry required (package dir or --registry)" >&2; exit 1; }
[ -n "$PKG_ID" ] || { echo "[build-graph] --package-id required (claim ids are scoped by package)" >&2; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "[build-graph] python3 is required" >&2; exit 1; }

# The program lives beside this script rather than inside a heredoc. The
# heredoc form hangs indefinitely on some hosts (observed 2026-09-09: this
# exact program blocked past 60s inline and ran in under a second from a
# file), and a stage that hangs is worse than one that fails. `exec` replaces
# the shell so the exit code passes through unchanged — the same convention
# score-sources.py / verify-sources.sh already use in this directory.
HERE="$(cd "$(dirname "$0")" && pwd)"
PROG="$HERE/build-graph.py"
[ -f "$PROG" ] || { echo "[build-graph] build-graph.py missing beside this script" >&2; exit 1; }

exec env REGISTRY="$REGISTRY" CRED="$CRED" CONTRA="$CONTRA" PKG_ID="$PKG_ID" \
         CRITICAL="$CRITICAL" OUT="$OUT" python3 "$PROG"
