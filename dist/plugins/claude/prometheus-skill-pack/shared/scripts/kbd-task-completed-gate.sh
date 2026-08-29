#!/usr/bin/env bash
# Claude TaskCompleted gate. It only governs tasks already tracked by the
# canonical KBD phase; unrelated Claude task-list items remain untouched.
set -uo pipefail

command -v jq >/dev/null 2>&1 || exit 0
command -v prometheus >/dev/null 2>&1 || exit 0

payload="$(cat)"
cwd="$(printf '%s' "$payload" | jq -r '.cwd // empty' 2>/dev/null)"
task_id="$(printf '%s' "$payload" | jq -r '.task_id // empty' 2>/dev/null)"
task_subject="$(printf '%s' "$payload" | jq -r '.task_subject // empty' 2>/dev/null)"
[ -n "$cwd" ] || cwd="$PWD"

find_project_root() {
  local cursor="$1"
  while [ "$cursor" != "/" ]; do
    if [ -f "$cursor/.prometheus/project.json" ]; then
      printf '%s\n' "$cursor"
      return 0
    fi
    cursor="$(dirname "$cursor")"
  done
  return 1
}

project_root="$(find_project_root "$cwd" 2>/dev/null || true)"
[ -n "$project_root" ] || exit 0
state="$(prometheus kbd --path "$project_root" status --json 2>/dev/null || true)"
[ -n "$state" ] || exit 0

result="$(printf '%s' "$state" | jq -c \
  --arg task "$task_id" --arg subject "$task_subject" '
  .activePath.phaseId as $phase
  | [.phases[$phase].changes[]?.tasks[]?
      | select(.id == $task or .title == $subject)] as $matches
  | if ($matches | length) == 0 then {tracked:false}
    elif ($matches | length) > 1 then {tracked:true, valid:false, reason:"ambiguous canonical task title"}
    else $matches[0] as $selected
      | ("task:" + ($selected.id | ascii_downcase)) as $key
      | (.latestBoundaryReceipts[$key] // null) as $receipt
      | {
          tracked: true,
          valid: (
            $selected.status == "complete"
            and $receipt != null
            and $receipt.edge == "after"
            and ($receipt.outcome == "pass" or $receipt.outcome == "repaired")
          ),
          taskId: $selected.id,
          status: $selected.status,
          receiptId: ($receipt.id // null),
          reason: "canonical completion and a valid kbd-apply after receipt are required"
        }
    end
  ' 2>/dev/null || true)"
[ -n "$result" ] || exit 0

tracked="$(printf '%s' "$result" | jq -r '.tracked // false')"
valid="$(printf '%s' "$result" | jq -r '.valid // false')"
[ "$tracked" = "true" ] || exit 0
[ "$valid" = "true" ] && exit 0

reason="$(printf '%s' "$result" | jq -r '.reason // "inconsistent canonical KBD completion"')"
canonical_id="$(printf '%s' "$result" | jq -r '.taskId // "unknown"')"
printf 'KBD completion blocked for task %s: %s. Complete it through /kbd-apply so the signed after receipt is recorded.\n' \
  "$canonical_id" "$reason" >&2
exit 2
