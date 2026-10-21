# shellcheck shell=bash
# phase-disposition.sh — the one reader of a phase's canonical closure state
# (issue #192). The interpretation lives in `prometheus kbd phase disposition`;
# this library only fetches it and matches whole tokens, never substrings of
# decision text.
#
#   kbd_phase_disposition_json <root> <phase>   full kbd.phase-disposition/v1 JSON
#   kbd_phase_disposition      <root> <phase>   one token:
#       UNBOUND | OPEN | CLOSED | REFUSED_BLOCKED | REFUSED_FAIL
#
# Both return 1 (printing nothing) when the disposition cannot be read; a
# caller under canonical runtime authority must treat that as BLOCKED.
# KBD_PHASE_DISPOSITION_FILE substitutes a fixture JSON (disclosed test seam).

kbd_phase_disposition_json() { # <root> <phase>
  local root="$1" phase="$2" out
  if [ -n "${KBD_PHASE_DISPOSITION_FILE:-}" ]; then
    out="$(cat "$KBD_PHASE_DISPOSITION_FILE" 2>/dev/null)" || return 1
  else
    command -v prometheus >/dev/null 2>&1 || return 1
    out="$(prometheus kbd --path "$root" phase disposition --id "$phase" --json 2>/dev/null)" || return 1
  fi
  printf '%s' "$out" | jq -e '.schema == "kbd.phase-disposition/v1"' >/dev/null 2>&1 || return 1
  printf '%s\n' "$out"
}

kbd_phase_disposition() { # <root> <phase>
  local json token
  json="$(kbd_phase_disposition_json "$1" "$2")" || return 1
  token="$(printf '%s' "$json" | jq -r '.disposition // empty')"
  case "$token" in
    UNBOUND|OPEN|CLOSED|REFUSED_BLOCKED|REFUSED_FAIL) printf '%s\n' "$token" ;;
    *) return 1 ;;
  esac
}
