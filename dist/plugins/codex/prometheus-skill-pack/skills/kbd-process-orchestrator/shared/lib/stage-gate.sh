# shellcheck shell=bash
# kbd-process-orchestrator/shared/lib/stage-gate.sh
#
# Stage precondition gates and handoff artifacts for the KBD lifecycle.
# Source this file; it does not run anything on import.
#
#   . shared/lib/stage-gate.sh
#
# Canonical stage order:
#
#   assess → analyze → spec → plan → execute → reflect
#
# `analyze` and `spec` are OPTIONAL stages: when no handoff exists for them
# the gate walks back to the nearest earlier stage instead of failing. This
# keeps pre-analyze projects working while reserving the slots.
#
# Functions:
#
#   kbd_stage_gate <stage> [phase-dir]
#       Verify the previous stage completed (handoff exists, possibly
#       skipped:true). Return 0 when satisfied or in legacy mode (phase has
#       no handoffs/ directory at all — warns once to stderr). Return 2 with
#       an exact remediation command on stderr when a required handoff is
#       missing.
#
#   kbd_stage_handoff_write <stage> <summary> [output-file ...] [phase-dir]
#       Atomically write handoffs/<stage>.handoff.json for the active phase.
#       Output files are phase-dir-relative or repo-relative paths; a trailing
#       argument that is an existing DIRECTORY is treated as the phase dir.
#
#   kbd_stage_handoff_skip <stage> <reason> [phase-dir]
#       Record an explicit skip (skipped:true) so the gate passes
#       deliberately rather than by drift.
#
# Phase dir resolution: explicit argument wins; else $KBD_PHASE_DIR; else
# derived from the nearest .kbd-orchestrator/current-waypoint.json
# (phase + childPointer).

_SG_ORDER=(assess analyze spec plan execute reflect)
_SG_OPTIONAL=" analyze spec "

_sg_root() {
  local dir="$PWD"
  while [ -n "$dir" ] && [ "$dir" != "/" ]; do
    [ -d "$dir/.kbd-orchestrator" ] && { printf '%s' "$dir"; return 0; }
    dir="$(dirname "$dir")"
  done
  return 1
}

_sg_phase_dir() {
  local explicit="${1:-}"
  if [ -n "$explicit" ]; then
    printf '%s' "$explicit"
    return 0
  fi
  if [ -n "${KBD_PHASE_DIR:-}" ]; then
    printf '%s' "$KBD_PHASE_DIR"
    return 0
  fi
  local root wp phase ptr
  root="$(_sg_root)" || return 1
  wp="$root/.kbd-orchestrator/current-waypoint.json"
  [ -f "$wp" ] || return 1
  phase="$(jq -r '.phase // empty' "$wp" 2>/dev/null)"
  [ -n "$phase" ] || return 1
  ptr="$(jq -r '.childPointer // empty' "$wp" 2>/dev/null)"
  local dir="$root/.kbd-orchestrator/phases/$phase"
  [ -n "$ptr" ] && [ -d "$dir/children/$ptr" ] && dir="$dir/children/$ptr"
  printf '%s' "$dir"
}

_sg_index() {
  local stage="$1" i
  for i in "${!_SG_ORDER[@]}"; do
    [ "${_SG_ORDER[$i]}" = "$stage" ] && { printf '%s' "$i"; return 0; }
  done
  return 1
}

kbd_stage_gate() {
  local stage="${1:-}"
  local idx
  idx="$(_sg_index "$stage")" || {
    printf 'kbd_stage_gate: unknown stage %q (order: %s)\n' "$stage" "${_SG_ORDER[*]}" >&2
    return 2
  }
  local phase_dir
  phase_dir="$(_sg_phase_dir "${2:-}")" || {
    printf 'kbd_stage_gate: warn: no phase dir resolvable — gate skipped\n' >&2
    return 0
  }
  [ "$idx" -eq 0 ] && return 0

  # Legacy mode: phase predates handoffs entirely.
  if [ ! -d "$phase_dir/handoffs" ]; then
    printf 'kbd_stage_gate: warn: %s has no handoffs/ — legacy phase, gate disabled\n' "$phase_dir" >&2
    return 0
  fi

  local i prev handoff
  i=$((idx - 1))
  while [ "$i" -ge 0 ]; do
    prev="${_SG_ORDER[$i]}"
    handoff="$phase_dir/handoffs/$prev.handoff.json"
    if [ -f "$handoff" ] && jq empty "$handoff" 2>/dev/null; then
      return 0
    fi
    case "$_SG_OPTIONAL" in
      *" $prev "*) i=$((i - 1)); continue ;;
    esac
    printf 'kbd_stage_gate: %s blocked — %s handoff missing.\n' "$stage" "$prev" >&2
    printf 'Remediation: run /kbd-%s first (or record an explicit skip with kbd_stage_handoff_skip %s "<reason>").\n' "$prev" "$prev" >&2
    return 2
  done
  return 0
}

_sg_handoff_emit() { # <stage> <summary> <skipped> <skipReason> <phase_dir> [outputs...]
  local stage="$1" summary="$2" skipped="$3" skip_reason="$4" phase_dir="$5"
  shift 5
  local idx next="null"
  idx="$(_sg_index "$stage")" || {
    printf 'stage-gate: unknown stage %q\n' "$stage" >&2
    return 1
  }
  if [ "$idx" -lt $((${#_SG_ORDER[@]} - 1)) ]; then
    next="${_SG_ORDER[$((idx + 1))]}"
  fi
  mkdir -p "$phase_dir/handoffs" || return 1
  local outputs_json tmp
  outputs_json="$(printf '%s\n' "$@" | jq -R . | jq -s 'map(select(length > 0))')"
  tmp="$(mktemp "$phase_dir/handoffs/.$stage.XXXXXX")" || return 1
  jq -n \
    --arg stage "$stage" \
    --arg next "$next" \
    --arg summary "$summary" \
    --arg skipReason "$skip_reason" \
    --argjson outputs "$outputs_json" \
    --argjson skipped "$skipped" \
    '{
      stage: $stage,
      completedAt: (now | todate),
      outputs: $outputs,
      nextStage: (if $next == "null" then null else $next end),
      summaryForNext: $summary,
      skipped: $skipped,
      skipReason: (if $skipReason == "" then null else $skipReason end)
    }' > "$tmp" || { rm -f "$tmp"; return 1; }
  mv "$tmp" "$phase_dir/handoffs/$stage.handoff.json"
}

kbd_stage_handoff_write() {
  local stage="${1:-}" summary="${2:-}"
  shift 2 || true
  local phase_dir="" outputs=()
  local arg
  for arg in "$@"; do
    if [ -d "$arg" ]; then phase_dir="$arg"; else outputs+=("$arg"); fi
  done
  phase_dir="$(_sg_phase_dir "$phase_dir")" || {
    printf 'kbd_stage_handoff_write: no phase dir resolvable\n' >&2
    return 1
  }
  _sg_handoff_emit "$stage" "$summary" false "" "$phase_dir" "${outputs[@]:-}"
}

kbd_stage_handoff_skip() {
  local stage="${1:-}" reason="${2:-unspecified}"
  local phase_dir
  phase_dir="$(_sg_phase_dir "${3:-}")" || {
    printf 'kbd_stage_handoff_skip: no phase dir resolvable\n' >&2
    return 1
  }
  _sg_handoff_emit "$stage" "skipped: $reason" true "$reason" "$phase_dir"
}
