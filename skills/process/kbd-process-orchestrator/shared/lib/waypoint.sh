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
      "path="               + (
        if (.path | type) == "array" and (.path | length) > 0 then (.path | join(","))
        else ([ (.phase // empty), (.childPointer // empty) ]
              | map(select(. != "" and . != null)) | join(",")) end
      ),
      "backend="            + (.backend            // ""),
      "wave="               + (.wave               // ""),
      "lastCompletedChange="+ (.lastCompletedChange // ""),
      "completionMetric="   + (.completionMetric   // "implementation"),
      "implementationCompleted=" + ((.implementationCompleted // .changesCompleted // 0) | tostring),
      "implementationTotal="     + ((.implementationTotal // .changesTotal // 0) | tostring),
      "certificationStatus="     + (.certificationStatus // "NOT_TRACKED"),
      "publicationStatus="       + (.publicationStatus // "NOT_TRACKED"),
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
  # waypoint_chain supplies its own surrounding spaces; normalize separators
  # such as the POSIX fallback (` > `) to avoid doubled whitespace.
  local sep_trim="${sep// /}"

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

# --- Waypoint v3+: arbitrary-depth position via path[] ---------------------
#
# path[] is the canonical position chain: path[0] = top-level phase, each
# subsequent element a nested child. The on-disk node dir interleaves
# `children/`:
#   path = [p0]            → phases/p0
#   path = [p0, c1]        → phases/p0/children/c1
#   path = [p0, c1, g2]    → phases/p0/children/c1/children/g2
#
# v3 is ADDITIVE: when a waypoint has no .path, it is synthesized from the v2
# fields ([phase] or [phase, childPointer]). parentPhase/childPointer remain
# maintained as derived (deepest-frame) fields for one release so existing
# scripts keep working unchanged.

# kbd_node_dir <p0> [p1] [p2] ...
# Echo the on-disk node dir (relative to the orchestrator root) for a path.
kbd_node_dir() {
  [[ $# -ge 1 ]] || return 1
  local out=".kbd-orchestrator/phases/$1"; shift
  local seg
  for seg in "$@"; do
    [[ -n "$seg" ]] || continue
    out="$out/children/$seg"
  done
  printf '%s' "$out"
}

# kbd_node_chain <p0> [p1] ...  — render the N-level breadcrumb.
kbd_node_chain() {
  [[ $# -ge 1 ]] || return 1
  local sep out="$1"; shift
  sep="$(chain_separator)"
  local seg
  for seg in "$@"; do
    [[ -n "$seg" ]] || continue
    out="$out$sep$seg"
  done
  printf '%s' "$out"
}

# _kbd_path_from_waypoint <waypoint-file> — echo the path as space-separated
# tokens, synthesizing from v2 fields when .path is absent.
_kbd_path_from_waypoint() {
  local wp="$1"
  command -v jq >/dev/null 2>&1 || return 1
  [[ -f "$wp" ]] || return 1
  jq -r '
    if (.path | type) == "array" and (.path | length) > 0 then
      .path | join(" ")
    else
      [ (.phase // empty), (.childPointer // empty) ]
      | map(select(. != "" and . != null)) | join(" ")
    end
  ' "$wp" 2>/dev/null
}

# kbd_current_node_dir [waypoint-file]  — resolve the active node dir from the
# waypoint's path[] (or synthesized v2 chain).
kbd_current_node_dir() {
  local wp="${1:-.kbd-orchestrator/current-waypoint.json}"
  local chain
  chain="$(_kbd_path_from_waypoint "$wp")" || return 1
  [[ -n "$chain" ]] || return 1
  # shellcheck disable=SC2086
  kbd_node_dir $chain
}
