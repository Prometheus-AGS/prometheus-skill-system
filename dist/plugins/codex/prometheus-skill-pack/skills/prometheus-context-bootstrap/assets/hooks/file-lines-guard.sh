#!/usr/bin/env bash
# PostToolUse:Edit|Write — a code file over 500 lines is reported to the model at once.
#
# The write has already happened, so exit 2 cannot prevent it; it feeds the message back so the next
# action is the partition, not more code on top. Every failure path exits 0: a guard that blocks because
# jq is missing is worse than one that does not fire.
set -uo pipefail

root="${CLAUDE_PROJECT_DIR:-$PWD}"
check="$root/scripts/check-file-lines.sh"
command -v jq >/dev/null 2>&1 || exit 0
[[ -x "$check" ]] || exit 0

file="$(jq -r '.tool_input.file_path // empty' 2>/dev/null)" || exit 0
[[ -n "$file" && -f "$file" ]] || exit 0

out="$("$check" "$file" 2>/dev/null)" && exit 0
[[ -n "$out" ]] || exit 0

cat >&2 <<EOF
FILE OVER 500 LINES — partition it before adding anything else.

  $out

Split by responsibility, not by line count: make a directory named after the file, one file per
responsibility (state / actions / selectors · types / operation / errors · screen / sections), and a thin
entry point (index.ts, mod.rs, a Dart barrel) that re-exports the same public surface so importers do not
change. If a responsibility belongs to another layer or feature, move it there. Do not add the file to
rules/line-limit-allowlist.txt — that list is for generated and vendored code only.
EOF
exit 2
