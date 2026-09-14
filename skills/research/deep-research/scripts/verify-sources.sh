#!/usr/bin/env bash
# verify-sources.sh — stage 05 credibility scoring. Delegates to score-sources.py
# (change-rah-010): the five documented rubric dimensions, scored only where the
# registry carries evidence, weights renormalised over the present dimensions,
# sorted, with a rank-sensitivity artifact.
#
# Usage:
#   bash verify-sources.sh <package_dir>            # reads sources/registry.json,
#                                                   # writes sources/credibility.json
#                                                   # and sensitivity.json
#   bash verify-sources.sh sources/registry.json [--out FILE] [--sensitivity FILE] [...]
#   echo -e "url1\nurl2" | bash verify-sources.sh   # legacy: prints
#                                                   # [{url, credibility_score, flags}]
#
# Every extra argument is passed to score-sources.py (--threshold, --penalties,
# --now, --weights, --profiles). bash 3.2 compatible (constraint C-05).
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
SCORER="$HERE/score-sources.py"
[ -f "$SCORER" ] || { echo "[verify-sources] score-sources.py missing beside this script" >&2; exit 1; }
command -v python3 >/dev/null 2>&1 || { echo "[verify-sources] python3 is required" >&2; exit 1; }

if [ $# -eq 0 ]; then
  # Legacy stdin mode: URLs only, no metadata, so every score rests on domain
  # authority alone and the flags say so (partial_evidence:...).
  exec python3 "$SCORER" --urls-from - --legacy-array
fi

TARGET="$1"; shift
if [ -d "$TARGET" ]; then
  REG="$TARGET/sources/registry.json"
  [ -f "$REG" ] || { echo "[verify-sources] no sources/registry.json in $TARGET" >&2; exit 1; }
  exec python3 "$SCORER" --registry "$REG" --chunks-dir "$TARGET/sources" \
    --out "$TARGET/sources/credibility.json" --sensitivity "$TARGET/sensitivity.json" "$@"
fi
[ -f "$TARGET" ] || { echo "[verify-sources] not a package directory or registry file: $TARGET" >&2; exit 1; }
exec python3 "$SCORER" --registry "$TARGET" "$@"
