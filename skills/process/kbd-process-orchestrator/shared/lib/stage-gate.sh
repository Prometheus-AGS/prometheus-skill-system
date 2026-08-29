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

# Refuse when the phase being worked is not the phase canonical state considers
# active. Returns 0 when they agree OR when the question is genuinely
# unanswerable (no waypoint at all — a pre-KBD tree), 2 on a real mismatch.
#
# Unanswerable and disagreeing are deliberately different: blocking every repo
# that has never run KBD would make the gate unusable, while passing a real
# mismatch is the bug this exists to stop.
_sg_assert_canonical_phase() { # <stage> <phase_dir>
  local stage="$1" phase_dir="$2" root wp active worked

  # Derive the root FROM THE PHASE DIR, not from $PWD.
  #
  # `_sg_root` walks up from the working directory. When KBD_PHASE_DIR points at
  # a sandbox outside the cwd — which is exactly how the existing suite drives
  # this — that walk escapes the sandbox and finds an unrelated repo's waypoint,
  # producing a phantom mismatch against a phase the caller never named.
  # Anchoring to the phase dir keeps the comparison within one tree.
  root="${phase_dir%/.kbd-orchestrator/phases/*}"
  [ "$root" = "$phase_dir" ] && root="$(_sg_root)"
  [ -n "$root" ] || return 0

  wp="$root/.kbd-orchestrator/current-waypoint.json"
  [ -f "$wp" ] || return 0

  active="$(jq -r '.activePhaseId // empty' "$wp" 2>/dev/null)"
  [ -n "$active" ] || return 0          # projection predates activePhaseId

  worked=""
  if [ -f "$phase_dir/progress.json" ]; then
    if ! worked="$(jq -r '.phaseId // .phase // empty' "$phase_dir/progress.json" 2>/dev/null)"; then
      printf 'kbd_stage_gate: %s blocked — %s/progress.json is unreadable.\n' \
        "$stage" "$phase_dir" >&2
      return 2
    fi
  fi
  [ -n "$worked" ] || worked="$(basename "$phase_dir")"
  [ "$worked" = "$active" ] && return 0

  printf 'kbd_stage_gate: %s blocked — canonical state disagrees.\n' "$stage" >&2
  printf '  working phase : %s\n  canonical     : %s\n' "$worked" "$active" >&2
  printf 'Artifacts written now would belong to a phase the runtime does not consider active.\n' >&2
  printf 'Remediation:\n  prometheus kbd --path . phase activate --command-id "activate-%s:$(date +%%Y%%m%%d)" --id %s\n' \
    "$worked" "$worked" >&2
  return 2
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
    # WAS: warn + return 0. A gate whose failure branch passes is not a gate.
    printf 'kbd_stage_gate: %s blocked — no phase directory resolvable.\n' "$stage" >&2
    printf 'Remediation: activate a phase first:\n  prometheus kbd --path . phase activate --command-id "activate-<phase>:$(date +%%Y%%m%%d)" --id <phase>\n' >&2
    return 2
  }

  # Canonical state must agree with the phase being worked.
  #
  # This check did not exist before 2026-08-12. On that date an agent authored a
  # whole phase — assess, analyze, spec, plan — while `activePhaseId` still named
  # a DIFFERENT, already-closed phase. Every stage passed. A second harness then
  # read the stale position projection and stalled for three cycles.
  #
  # `_sg_phase_dir` resolves from the waypoint's `.phase`, so without this the
  # gate reads whatever the projection says and never asks whether canonical
  # state agrees. Note this runs BEFORE the index-0 shortcut: opening a phase is
  # precisely when canonical state must exist, so `assess` is not exempt.
  _sg_assert_canonical_phase "$stage" "$phase_dir" || return 2

  [ "$idx" -eq 0 ] && return 0

  # A missing handoffs/ used to disable the gate as "legacy". But every NEW phase
  # also lacks handoffs/ until something writes one, so that exempted exactly the
  # phases most at risk. Create it and continue: absence is not evidence of a
  # pre-handoff phase, and the per-stage checks below still apply.
  if [ ! -d "$phase_dir/handoffs" ]; then
    mkdir -p "$phase_dir/handoffs" 2>/dev/null || {
      printf 'kbd_stage_gate: %s blocked — cannot create %s/handoffs\n' "$stage" "$phase_dir" >&2
      return 2
    }
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
