#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
# shellcheck source=/dev/null
. "$root/shared/lib/progress.sh"

case "${1:-}" in
  --mark-implementation-complete)
    file="${2:?usage: kbd-validate-progress.sh --mark-implementation-complete <progress.json> <change-id>}"
    change_id="${3:?usage: kbd-validate-progress.sh --mark-implementation-complete <progress.json> <change-id>}"
    kbd_progress_mark_implementation_complete "$file" "$change_id"
    printf 'KBD implementation marked complete: %s (%s)\n' "$change_id" "$file"
    ;;
  *)
    file="${1:?usage: kbd-validate-progress.sh <progress.json>}"
    kbd_progress_validate "$file"
    printf 'KBD progress validation passed: %s\n' "$file"
    ;;
esac
