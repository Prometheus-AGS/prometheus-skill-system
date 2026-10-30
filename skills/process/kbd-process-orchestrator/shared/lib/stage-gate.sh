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
#       skipped:true). Return 0 when satisfied. Return 2 with remediation
#       on stderr when the phase cannot be resolved, canonical phase views
#       disagree, or a required predecessor handoff is missing. A missing
#       handoffs/ directory does not bypass required predecessors.
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
#   _sg_validate_handoff <file> <expected-stage> <phase-dir> <for-stage>
#       Semantic handoff validation (replaces bare `jq empty`): valid JSON,
#       required schema fields, stage-name match, identity.phaseId agreement
#       when identity is present. At the Plan→Execute boundary the plan
#       handoff MUST carry full identity (projectId, runId, phaseId,
#       planRevision, contractDigest) bound to the adopted contract — an
#       identity-less or foreign plan handoff is rejected, never dispatched.
#
#   Handoff writes merge $KBD_HANDOFF_IDENTITY_JSON (a JSON identity object)
#   into the emitted handoff when the variable is set — this is how the plan
#   stage binds its handoff to the adopted contract.
#
# Phase dir resolution: explicit argument wins; else $KBD_PHASE_DIR; else
# derived from the nearest .kbd-orchestrator/current-waypoint.json
# (phase + childPointer).

. "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)/package-root.sh"
# shellcheck source=/dev/null
. "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)/phase-disposition.sh"

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

# Semantic handoff validation. Replaces bare `jq empty`: a handoff must be
# valid JSON with the required schema fields, the stage name must match the
# predecessor being closed, a skip must carry its reason, every declared
# output must exist, and when an identity object is present its
# phaseId must match the phase directory the handoff lives in. At the
# Plan→Execute boundary the plan handoff MUST carry identity (projectId,
# runId, phaseId, planRevision, contractDigest) — an identity-less or stale
# plan handoff is rejected instead of silently dispatching (issue #186).
_sg_validate_handoff() { # <handoff-file> <expected-stage> <phase_dir> <for-stage>
  local handoff="$1" prev="$2" phase_dir="$3" for_stage="$4"
  if ! jq empty "$handoff" 2>/dev/null; then
    printf 'kbd_stage_gate: %s blocked — %s handoff is not valid JSON: %s\n' "$for_stage" "$prev" "$handoff" >&2
    printf 'Remediation: rewrite the handoff via kbd_stage_handoff_write %s (never hand-edit).\n' "$prev" >&2
    return 2
  fi
  local ok
  ok="$(jq -r --arg s "$prev" '
    (if (.stage // "") == $s then "stage-ok" else "stage-mismatch" end) + " " +
    (if ((.completedAt // "") | type) == "string" and ((.completedAt // "") | length) > 0 then "ts-ok" else "ts-missing" end) + " " +
    (if (.skipped | type) == "boolean" then "skip-ok" else "skip-bad" end)
  ' "$handoff" 2>/dev/null)" || return 2
  if [ "$ok" != "stage-ok ts-ok skip-ok" ]; then
    printf 'kbd_stage_gate: %s blocked — %s handoff fails schema check (%s): %s\n' "$for_stage" "$prev" "$ok" "$handoff" >&2
    printf 'Remediation: rewrite the handoff via kbd_stage_handoff_write %s (never hand-edit).\n' "$prev" >&2
    return 2
  fi
  # A skip is an explicit decision, so it must say why (issue #184).
  if [ "$(jq -r '.skipped' "$handoff")" = "true" ] \
     && [ -z "$(jq -r '.skipReason // empty' "$handoff" 2>/dev/null)" ]; then
    printf 'kbd_stage_gate: %s blocked — %s handoff is skipped without a skipReason: %s\n' "$for_stage" "$prev" "$handoff" >&2
    printf 'Remediation: record the skip via kbd_stage_handoff_skip %s "<reason>".\n' "$prev" >&2
    return 2
  fi
  # Every declared output must still exist (issue #184). Paths are resolved
  # against the phase directory, then the repository root; absolute paths are
  # taken as given. Content is not hashed: outputs routinely include runtime
  # projections such as progress.json that are regenerated on every transition.
  local out_root missing_outputs="" out
  out_root="${phase_dir%/.kbd-orchestrator/phases/*}"
  [ "$out_root" = "$phase_dir" ] && out_root="$(_sg_root)"
  while IFS= read -r out; do
    [ -n "$out" ] || continue
    case "$out" in
      /*) [ -e "$out" ] && continue ;;
      *) { [ -e "$phase_dir/$out" ] || { [ -n "$out_root" ] && [ -e "$out_root/$out" ]; }; } && continue ;;
    esac
    missing_outputs="${missing_outputs:+$missing_outputs, }$out"
  done < <(jq -r '.outputs[]? | strings' "$handoff" 2>/dev/null)
  if [ -n "$missing_outputs" ]; then
    printf 'kbd_stage_gate: %s blocked — %s handoff declares outputs that do not exist: %s\n' "$for_stage" "$prev" "$missing_outputs" >&2
    printf 'Remediation: restore the outputs or rerun /kbd-%s so its handoff matches what exists.\n' "$prev" >&2
    return 2
  fi
  # Identity, when present, must agree with the phase directory it lives in.
  local id_phase worked=""
  id_phase="$(jq -r '.identity.phaseId // empty' "$handoff" 2>/dev/null)"
  if [ -n "$id_phase" ]; then
    if [ -f "$phase_dir/progress.json" ]; then
      worked="$(jq -r '.phaseId // .phase // empty' "$phase_dir/progress.json" 2>/dev/null)"
    fi
    [ -n "$worked" ] || worked="$(basename "$phase_dir")"
    if [ "$id_phase" != "$worked" ]; then
      printf 'kbd_stage_gate: %s blocked — %s handoff identity.phaseId=%s disagrees with phase %s.\n' "$for_stage" "$prev" "$id_phase" "$worked" >&2
      return 2
    fi
  fi
  # Plan→Execute boundary: the plan handoff MUST carry full identity bound to
  # the adopted contract. Earlier boundaries stay optional-stage compatible.
  if [ "$for_stage" = "execute" ] && [ "$prev" = "plan" ]; then
    local id_ok
    id_ok="$(jq -r '
      if (.identity // null) == null then "missing"
      elif ((.identity.projectId // "") | length) == 0 then "missing-projectId"
      elif ((.identity.runId // "") | length) == 0 then "missing-runId"
      elif ((.identity.phaseId // "") | length) == 0 then "missing-phaseId"
      elif ((.identity.planRevision // null) | type) != "number" then "missing-planRevision"
      elif ((.identity.contractDigest // "") | test("^[0-9a-f]{64}$")) | not then "missing-contractDigest"
      else "ok" end
    ' "$handoff" 2>/dev/null)"
    if [ "$id_ok" != "ok" ]; then
      printf 'kbd_stage_gate: execute blocked — plan handoff identity %s.\n' "$id_ok" >&2
      printf 'The Plan→Execute boundary requires an adopted-contract-bound handoff (issue #186).\n' >&2
      printf 'Remediation: adopt the phase contract using the package-root command in references/templates/adopted-contract.template.md, then rewrite the plan handoff with KBD_HANDOFF_IDENTITY_JSON set.\n' >&2
      return 2
    fi
    # B2 (issue #192): a phase CLOSED in canonical state refuses further
    # execute dispatch. The disposition comes from the typed runtime authority
    # (`prometheus kbd phase disposition`), never from decision-summary text,
    # so a recorded string cannot fake or hide a closure. REFUSED_* phases
    # still admit repair dispatch. Under canonical runtime authority an
    # unreadable disposition blocks instead of failing open.
    local sg_root worked_phase sg_token
    sg_root="${phase_dir%/.kbd-orchestrator/phases/*}"
    [ "$sg_root" = "$phase_dir" ] && sg_root="$(_sg_root)"
    worked_phase="$(jq -r '.identity.phaseId // empty' "$handoff" 2>/dev/null)"
    [ -n "$worked_phase" ] || worked_phase="$(basename "$phase_dir")"
    if sg_token="$(kbd_phase_disposition "$sg_root" "$worked_phase")"; then
      if [ "$sg_token" = "CLOSED" ]; then
        printf 'kbd_stage_gate: execute blocked — phase %s is CLOSED in canonical state (typed phase disposition).\n' "$worked_phase" >&2
        printf 'A closed phase receives no further dispatch. A successor starts only with a signed start-successor authorization (kbd-next-phase --authorize-successor).\n' >&2
        return 2
      fi
    elif [ -n "${KBD_PHASE_DISPOSITION_FILE:-}" ] || { [ -n "$sg_root" ] && command -v kbd_runtime_authoritative >/dev/null 2>&1 && kbd_runtime_authoritative "$sg_root"; } \
         || { [ -n "$sg_root" ] && [ "$(jq -r '.generatedBy // empty' "$sg_root/.kbd-orchestrator/current-waypoint.json" 2>/dev/null)" = "kbd-runtime" ]; }; then
      printf 'kbd_stage_gate: execute blocked — canonical phase disposition for %s is unreadable.\n' "$worked_phase" >&2
      printf 'Remediation: install a prometheus CLI with typed phase closure (prometheus kbd phase disposition), then retry.\n' >&2
      return 2
    fi
    # B1: full canonical readiness validation at the real Plan→Execute
    # transition. Handoff identity alone is not sufficient: the adopted
    # contract, its digest, scenario coverage and contract-quality gates must
    # all validate, or the boundary does not complete and nothing dispatches.
    local pack_root checker
    pack_root="$(kbd_package_root)"
    checker="${pack_root:+$pack_root/}shared/scripts/kbd-phase-readiness-check.sh"
    if [ -z "$pack_root" ] || [ ! -f "$checker" ]; then
      printf 'kbd_stage_gate: execute blocked — readiness checker missing at %s (broken install or payload drift).\n' "$checker" >&2
      printf 'Remediation: reinstall the kbd-process-orchestrator payload; do not bypass.\n' >&2
      return 2
    fi
    local rc_out rc_ec rc_fixture=()
    [ -n "${KBD_READINESS_STATUS_FILE:-}" ] && rc_fixture+=(--status-file "$KBD_READINESS_STATUS_FILE")
    [ -n "${KBD_PHASE_DISPOSITION_FILE:-}" ] && rc_fixture+=(--disposition-file "$KBD_PHASE_DISPOSITION_FILE")
    rc_out="$(cd "$sg_root" && bash "$checker" --phase "$worked_phase" --phase-dir "$phase_dir" ${rc_fixture[@]+"${rc_fixture[@]}"} 2>&1)"; rc_ec=$?
    if [ "$rc_ec" -ne 0 ]; then
      printf 'kbd_stage_gate: execute blocked — phase readiness validation failed (exit %s).\n%s\n' "$rc_ec" "$rc_out" >&2
      printf 'Remediation: satisfy the package readiness checker (adopted contract, identity, digest, coverage, quality gates) before dispatch.\n' >&2
      return 2
    fi
  fi
  return 0
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
    if [ -f "$handoff" ]; then
      _sg_validate_handoff "$handoff" "$prev" "$phase_dir" "$stage" || return 2
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
  local outputs_json tmp identity_json="${KBD_HANDOFF_IDENTITY_JSON:-}"
  if [ -n "$identity_json" ] && ! printf '%s' "$identity_json" | jq empty 2>/dev/null; then
    printf 'stage-gate: KBD_HANDOFF_IDENTITY_JSON is not valid JSON\n' >&2
    return 1
  fi
  outputs_json="$(printf '%s\n' "$@" | jq -R . | jq -s 'map(select(length > 0))')"
  tmp="$(mktemp "$phase_dir/handoffs/.$stage.XXXXXX")" || return 1
  jq -n \
    --arg stage "$stage" \
    --arg next "$next" \
    --arg summary "$summary" \
    --arg skipReason "$skip_reason" \
    --arg identity "$identity_json" \
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
    } + (if $identity == "" then {} else {identity: ($identity | fromjson)} end)' > "$tmp" || { rm -f "$tmp"; return 1; }
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
