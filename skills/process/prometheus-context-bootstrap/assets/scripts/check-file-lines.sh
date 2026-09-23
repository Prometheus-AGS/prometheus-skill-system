#!/usr/bin/env bash
# No code file over 500 physical lines. Partition by responsibility instead (see rules: architecture.md).
#
#   scripts/check-file-lines.sh              every tracked and untracked-but-not-ignored code file
#   scripts/check-file-lines.sh <path>...    only these paths (used by the file-lines-guard hook)
#
# Exemptions: rules/line-limit-allowlist.txt, `<glob> — <reason>` per line. Generated and vendored code only.
# Exit 0 = within the limit, 1 = at least one file over it, 2 = could not run.
set -uo pipefail

LIMIT=500
EXTENSIONS='ts|tsx|mts|cts|js|jsx|mjs|cjs|rs|dart|sql|sh|bash|css|scss|py|go|kt|swift'

here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"   # the project this script belongs to, not the caller's cwd
root="$(git -C "$here" rev-parse --show-toplevel 2>/dev/null)" || root="$here"
allowlist="$root/rules/line-limit-allowlist.txt"

patterns=()
if [[ -f "$allowlist" ]]; then
  while IFS= read -r line; do
    line="${line%%#*}"
    glob="$(printf '%s' "${line%% — *}" | sed 's/[[:space:]]*$//')"
    [[ -n "$glob" ]] && patterns+=("$glob")
  done < "$allowlist"
fi

is_allowed() {
  local rel="$1" glob
  for glob in ${patterns[@]+"${patterns[@]}"}; do
    # shellcheck disable=SC2254
    case "$rel" in $glob|${glob#\*\*/}) return 0 ;; esac
  done
  return 1
}

if [[ $# -gt 0 ]]; then
  files=("$@")
else
  files=()
  while IFS= read -r f; do files+=("$f"); done < <(cd "$root" && git ls-files -co --exclude-standard 2>/dev/null)
  [[ ${#files[@]} -eq 0 ]] && { echo "check-file-lines: no files listed (not a git repository?)" >&2; exit 2; }
fi

over=0
for f in ${files[@]+"${files[@]}"}; do
  [[ "$f" = /* ]] && abs="$f" || abs="$root/$f"
  rel="${abs#"$root"/}"
  [[ -f "$abs" && ! -L "$abs" ]] || continue
  [[ "$rel" =~ \.($EXTENSIONS)$ ]] || continue
  is_allowed "$rel" && continue
  lines="$(awk 'END { print NR }' "$abs")"   # physical lines: counts a final line with no trailing newline
  if (( lines > LIMIT )); then
    printf 'OVER  %5d lines  %s\n' "$lines" "$rel"
    over=$((over + 1))
  fi
done

if (( over > 0 )); then
  cat >&2 <<EOF

check-file-lines: $over file(s) over $LIMIT lines.
Partition by responsibility, not by line count: make a directory named after the file, one file per
responsibility, and a thin entry point (index.ts / mod.rs / a Dart barrel) that re-exports the same surface.
Only generated or vendored files may be listed in rules/line-limit-allowlist.txt.
EOF
  exit 1
fi
exit 0
