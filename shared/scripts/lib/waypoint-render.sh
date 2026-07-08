#!/usr/bin/env bash
# waypoint-render.sh — pure renderer for the per-turn position block.
# Source this file, then call `waypoint_render`:
#   source "$(dirname "$0")/lib/waypoint-render.sh"
#   waypoint_render            # prints the sentinel-wrapped block, or nothing
#
# Contract:
#   - No side effects on source; never writes state.
#   - Prints nothing and returns 0 when no .kbd-orchestrator is found, jq is
#     missing, or the waypoint is unreadable — callers stay degradation-safe.
#   - Waypoint keys are read camelCase-first with snake_case fallback because
#     long-lived waypoints carry both generations of keys.
#   - PROMETHEUS_POSITION_VERBOSITY=dense (default) | explain
#     (explain appends a "Why:" line sourced from waypoint next_action).

# Walk up from $PWD to the filesystem root looking for .kbd-orchestrator/.
# Also checks REPO_ROOT, CLAUDE_PLUGIN_ROOT, and common project paths.
# Prints the directory that CONTAINS .kbd-orchestrator and returns 0, else 1.
_wr_find_root() {
  local dir="$PWD"
  # Walk up from CWD
  while [ -n "$dir" ] && [ "$dir" != "/" ]; do
    if [ -d "$dir/.kbd-orchestrator" ]; then
      printf '%s' "$dir"
      return 0
    fi
    dir="$(dirname "$dir")"
  done
  [ -d "/.kbd-orchestrator" ] && { printf '/'; return 0; }
  # Explicit env var fallbacks (for hooks that run from non-project CWD)
  for candidate in "${REPO_ROOT:-}" "${CLAUDE_PLUGIN_ROOT:-}"; do
    [ -n "$candidate" ] || continue
    if [ -d "$candidate/.kbd-orchestrator" ]; then
      printf '%s' "$candidate"
      printf '[wr] resolved via %s\n' "$([ "$candidate" = "${REPO_ROOT:-}" ] && echo "REPO_ROOT" || echo "CLAUDE_PLUGIN_ROOT")" >&2
      return 0
    fi
  done
  return 1
}

# _wr_get <json-file> <camelCaseKey> <snake_case_key>
# Empty string when neither key is present or value is null.
_wr_get() {
  jq -r --arg c "$2" --arg s "$3" '.[$c] // .[$s] // empty' "$1" 2>/dev/null || true
}

# _wr_is_terminal_status <status>
# Returns 0 when the given waypoint status/stage means "this phase is finished"
# and no further work should be steered — so hooks stop nagging and next-phase
# gates pass. Broad by design: the KBD toolchain has accumulated several
# vocabularies for "done" across skills, scripts, and hand-authored project
# state (e.g. reflect writes "reflected" in project.json while older scripts
# expect "reflect_complete"). Matching only one string is the seam that made a
# completed phase re-nag /kbd-execute forever. An empty status is treated as
# terminal too: a waypoint with no status carries no active step to enforce.
# Case-insensitive; tolerates hyphen/space variants.
_wr_is_terminal_status() {
  local s
  s="$(printf '%s' "${1:-}" | tr '[:upper:] -' '[:lower:]__')"
  case "$s" in
    ""|reflected|reflect_complete|phase_complete|complete|completed|done|archived|closed|reflect_done|phase_done)
      return 0 ;;
    *)
      return 1 ;;
  esac
}

waypoint_render() {
  command -v jq >/dev/null 2>&1 || return 0

  local root
  root="$(_wr_find_root)" || return 0
  local wp="$root/.kbd-orchestrator/current-waypoint.json"
  [ -f "$wp" ] || return 0
  jq empty "$wp" 2>/dev/null || return 0

  local phase child change status last next why
  phase="$(_wr_get "$wp" phase phase)"
  [ -n "$phase" ] || return 0
  child="$(_wr_get "$wp" childPointer child_pointer)"
  change="$(_wr_get "$wp" change active_change)"
  status="$(_wr_get "$wp" status stage)"
  last="$(_wr_get "$wp" currentTask current_task)"
  [ -n "$last" ] || last="$(_wr_get "$wp" lastCompleted last_completed)"
  next="$(_wr_get "$wp" exactNextCommand exact_next_command)"
  why="$(_wr_get "$wp" nextAction next_action)"

  # Progress: prefer the phase's live progress.json ledger, fall back to the
  # waypoint's own counters.
  local node_dir="$root/.kbd-orchestrator/phases/$phase"
  [ -n "$child" ] && [ -d "$node_dir/children/$child" ] && node_dir="$node_dir/children/$child"
  local prog="$node_dir/progress.json"
  local done total stage_line="" tasks_done tasks_total
  if [ -f "$prog" ] && jq empty "$prog" 2>/dev/null; then
    done="$(jq -r '.changes_completed // empty' "$prog" 2>/dev/null)"
    total="$(jq -r '.changes_total // empty' "$prog" 2>/dev/null)"
    if [ -n "$change" ]; then
      tasks_done="$(jq -r --arg id "$change" '.changes[]? | select(.id == $id) | .tasks_done // empty' "$prog" 2>/dev/null)"
      tasks_total="$(jq -r --arg id "$change" '.changes[]? | select(.id == $id) | .tasks_total // empty' "$prog" 2>/dev/null)"
    fi
  fi
  [ -n "${done:-}" ] || done="$(_wr_get "$wp" changesCompleted changes_completed)"
  [ -n "${total:-}" ] || total="$(_wr_get "$wp" changesTotal changes_total)"

  local crumb="$phase"
  [ -n "$child" ] && crumb="$crumb › $child"
  if [ -n "$change" ]; then
    crumb="$crumb › $change"
    [ -n "${tasks_total:-}" ] && [ "${tasks_total:-0}" != "0" ] && \
      crumb="$crumb › tasks ${tasks_done:-0}/${tasks_total}"
  fi

  # Prefer the unified position model's cursor when it is at least as fresh
  # as the waypoint (position.json is derived; freshness check avoids
  # rendering a stale tree after a manual waypoint edit).
  local pos="$root/.kbd-orchestrator/position.json"
  if [ -f "$pos" ] && [ ! "$pos" -ot "$wp" ] && jq empty "$pos" 2>/dev/null; then
    local pos_crumb
    pos_crumb="$(jq -r '.cursor | select(type=="array") | join(" › ")' "$pos" 2>/dev/null)"
    [ -n "$pos_crumb" ] && crumb="$pos_crumb"
  fi

  printf '<!-- prometheus-position -->\n'
  printf 'Position: %s' "$crumb"
  [ -n "$status" ] && printf ' | status: %s' "$status"
  printf '\n'
  if [ -n "${total:-}" ] && [ "${total:-0}" != "0" ]; then
    printf 'Progress: changes %s/%s\n' "${done:-0}" "$total"
  fi
  [ -n "$last" ] && printf 'Last: %s\n' "$last"
  [ -n "$next" ] && printf 'Next: %s\n' "$next"
  if [ "${PROMETHEUS_POSITION_VERBOSITY:-dense}" = "explain" ] && [ -n "$why" ]; then
    printf 'Why: %s\n' "$why"
  fi
  printf '<!-- /prometheus-position -->\n'
  return 0
}
