#!/usr/bin/env bash
# assemble-report.sh — concatenate report sections and enforce the claim-set invariant.
#
# Deterministic, no model call. It is the gate that lets stage 09 be split across
# several generation calls without accuracy drifting as the report grows:
#
#   orphan marker   a [claim-…] marker citing an id no section was assigned
#   missing ref     an assigned id that resolves to no claim in graph.json
#   claim drift     a section citing outside its assignment, or (after the
#                   editor) the report citing MORE than before
#   label misuse    prose describing a claim more strongly than its label allows
#
# Any one of these fails the build. A long report is exactly where an unsupported
# sentence is least visible — it reads fluently by then — so the check is
# mechanical rather than editorial.
#
# Usage:
#   assemble-report.sh --package <dir>              first pass, after the writers
#   assemble-report.sh --package <dir> --post-edit  re-run, after the editor
#
# Exit: 0 assembled, 1 usage/IO error, 2 invariant violation.
# bash 3.2 compatible (C-05).
set -euo pipefail

PKG=""; POST_EDIT=0
while [ $# -gt 0 ]; do
    case "$1" in
        --package)   PKG="${2:-}"; shift 2 ;;
        --post-edit) POST_EDIT=1; shift ;;
        -h|--help)   sed -n '2,20p' "$0"; exit 0 ;;
        *) echo "assemble-report: unknown argument: $1" >&2; exit 1 ;;
    esac
done

[ -n "$PKG" ] || { echo "assemble-report: --package <dir> is required" >&2; exit 1; }
[ -f "$PKG/report/outline.json" ] || { echo "assemble-report: no report/outline.json under $PKG" >&2; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "assemble-report: python3 is required" >&2; exit 1; }

# The program lives beside this script rather than inside a heredoc. The
# heredoc form hangs indefinitely on some hosts (observed 2026-09-09: this
# exact program blocked past 30s inline and ran in under a second from a
# file). `exec` replaces the shell so the exit code — 2 for an invariant
# violation — passes through unchanged.
HERE="$(cd "$(dirname "$0")" && pwd)"
PROG="$HERE/assemble-report.py"
[ -f "$PROG" ] || { echo "assemble-report: assemble-report.py missing beside this script" >&2; exit 1; }

exec python3 "$PROG" "$PKG" "$POST_EDIT"
