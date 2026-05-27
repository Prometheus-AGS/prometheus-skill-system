# shellcheck shell=bash
# kbd-process-orchestrator/shared/lib/waypoint.sh
#
# Pure-bash + jq helpers for reading the KBD waypoint, rendering the phase
# chain, and validating worktree-root membership.
#
# Source this file; it does not run anything on import.
#
#   . shared/lib/waypoint.sh
#
# Functions (POSIX-bash 3.2 compatible):
#
#   waypoint_load <path>
#       Emit each documented field on stdout as `key=value` lines, applying
#       documented defaults for absent fields.
#
#   chain_separator
#       Echo the active separator: U+203A `›` by default, or ` > ` when the
#       locale is POSIX/C.
#
#   waypoint_chain <parent-or-empty> <phase> <pointer-or-empty>
#       Echo the rendered phase chain ("parent › phase › pointer") with empty
#       slots elided.
#
#   expand_kbd_path <literal>
#       Echo the input string with ${HOME} and ${USER} expanded against the
#       current environment.
#
#   is_descendant <child-path> <parent-path>
#       Exit 0 if <child-path> is the same as <parent-path>/<...>, after
#       canonicalising both with `cd … && pwd -P`. Exit 1 otherwise.

waypoint_load() {
  local path="$1"
  [[ -n "$path" && -f "$path" ]] || { printf 'waypoint_load: missing file: %s\n' "$path" >&2; return 1; }
  command -v jq >/dev/null 2>&1 || { printf 'waypoint_load: jq is required\n' >&2; return 1; }

  jq -r '
    def s(d): if . == null then d else . end;
    [
      "phase="              + (.phase              // ""),
      "previousPhase="      + (.previousPhase      // ""),
      "change="             + (.change             // ""),
      "status="             + (.status             // ""),
      "currentTask="        + (.currentTask        // ""),
      "nextPendingChange="  + (.nextPendingChange  // ""),
      "sourceTool="         + (.sourceTool         // ""),
      "exactNextCommand="   + (.exactNextCommand   // ""),
      "parentPhase="        + (.parentPhase        // ""),
      "childPhases="        + ((.childPhases       // []) | join(",")),
      "childPointer="       + (.childPointer       // ""),
      "backend="            + (.backend            // ""),
      "wave="               + (.wave               // ""),
      "lastCompletedChange="+ (.lastCompletedChange // ""),
      "updatedAt="          + (.updatedAt          // "")
    ] | .[]
  ' "$path"
}

chain_separator() {
  case "${LC_ALL:-${LANG:-}}" in
    POSIX|C|C.*) printf ' > ' ;;
    *)           printf $'\xe2\x80\xba' ; printf ' ' ;;  # `›` followed by a space
  esac
}

waypoint_chain() {
  local parent="$1" phase="$2" pointer="$3"
  local sep
  sep="$(chain_separator)"
  # Trim trailing space so we can join cleanly.
  local sep_trim="${sep% }"

  local out=""
  if [[ -n "$parent" ]]; then
    out="$parent"
  fi
  if [[ -n "$phase" ]]; then
    if [[ -n "$out" ]]; then out="$out $sep_trim $phase"; else out="$phase"; fi
  fi
  if [[ -n "$pointer" ]]; then
    if [[ -n "$out" ]]; then out="$out $sep_trim $pointer"; else out="$pointer"; fi
  fi
  printf '%s' "$out"
}

expand_kbd_path() {
  # Restrict expansion to a documented set; do not eval arbitrary content.
  # Patterns are stashed in a temp var so the embedded `}` doesn't close the
  # outer parameter expansion early.
  local in="$1" pat
  pat='${HOME}'; in="${in//"$pat"/$HOME}"
  pat='${USER}'; in="${in//"$pat"/${USER:-}}"
  pat='$HOME';   in="${in//"$pat"/$HOME}"
  pat='$USER';   in="${in//"$pat"/${USER:-}}"
  printf '%s' "$in"
}

is_descendant() {
  local child="$1" parent="$2"
  [[ -n "$child" && -n "$parent" ]] || return 1

  local cchild cparent
  cchild="$(cd "$child" 2>/dev/null && pwd -P)" || return 1
  # Parent may not exist yet; canonicalise via its existing prefix.
  if [[ -d "$parent" ]]; then
    cparent="$(cd "$parent" 2>/dev/null && pwd -P)" || return 1
  else
    cparent="$parent"
  fi

  [[ "$cchild" == "$cparent" ]] && return 1   # same path is NOT a descendant
  case "$cchild" in
    "$cparent"/*) return 0 ;;
    *)            return 1 ;;
  esac
}
