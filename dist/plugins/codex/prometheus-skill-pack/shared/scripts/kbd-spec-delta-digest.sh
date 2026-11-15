#!/usr/bin/env bash
# kbd-spec-delta-digest.sh — SHA-256 over a phase's OpenSpec change deltas (VLBV CR-04).
#
# Usage: kbd-spec-delta-digest.sh --phase <id> [--root <repo>] [--status-file <status.json>]
#
# The digest covers every file under openspec/changes/<change>/specs/ for each
# change registered to the phase in canonical KBD state (archived copies under
# openspec/changes/archive/*-<change>/specs/ are used once a change is archived).
# Lines "<change>/<path under specs/><TAB>sha256" are sorted (archive-location independent) and hashed, so the value is
# independent of file-system order. Authors stamp it into the v2 contract as
# method.specDeltaDigest before adoption; kbd-phase-readiness-check.sh recomputes
# it and returns REJECT-STALE when a bound requirement was edited.
# Prints the 64-hex digest, or nothing (exit 3) when the phase has no OpenSpec
# deltas. Bash 3.2 compatible.
set -euo pipefail
PHASE=""; ROOT="$PWD"; STATUS_FILE=""
while [ $# -gt 0 ]; do
  case "$1" in
    --phase) PHASE="${2:?}"; shift 2 ;;
    --root) ROOT="${2:?}"; shift 2 ;;
    --status-file) STATUS_FILE="${2:?}"; shift 2 ;;
    *) printf 'kbd-spec-delta-digest: unknown argument %s\n' "$1" >&2; exit 1 ;;
  esac
done
[ -n "$PHASE" ] || { printf 'usage: kbd-spec-delta-digest.sh --phase <id>\n' >&2; exit 1; }
if [ -n "$STATUS_FILE" ]; then STATUS_JSON="$(cat "$STATUS_FILE")"
else STATUS_JSON="$(prometheus kbd --path "$ROOT" status --json 2>/dev/null)" || { printf 'kbd-spec-delta-digest: canonical status unavailable\n' >&2; exit 2; }
fi
CHANGES="$(printf '%s' "$STATUS_JSON" | jq -r --arg p "$PHASE" '.phases[$p].changes // {} | keys[]')"
LINES=""
for change in $CHANGES; do
  dir="$ROOT/openspec/changes/$change/specs"
  if [ ! -d "$dir" ]; then
    dir="$(ls -d "$ROOT"/openspec/changes/archive/*-"$change"/specs 2>/dev/null | tail -1 || true)"
  fi
  [ -n "$dir" ] && [ -d "$dir" ] || continue
  while IFS= read -r file; do
    [ -n "$file" ] || continue
    LINES="$LINES$change/${file#"$dir"/}	$(shasum -a 256 "$file" | awk '{print $1}')
"
  done <<EOF
$(find "$dir" -type f | LC_ALL=C sort)
EOF
done
[ -n "$LINES" ] || exit 3
printf '%s' "$LINES" | LC_ALL=C sort | shasum -a 256 | awk '{print $1}'
