#!/usr/bin/env bash
# skills/kbd-next-child/kbd-next-child.sh
# Advance childPointer to the next child of the active parent phase.

set -euo pipefail
die()  { printf 'kbd-next-child: %s\n' "$*" >&2; exit 1; }
warn() { printf 'kbd-next-child: warn: %s\n' "$*" >&2; }

target="${1:-}"
command -v jq >/dev/null 2>&1 || die "jq is required"

wp=".kbd-orchestrator/current-waypoint.json"
[[ -f "$wp" ]] || die "no current-waypoint.json — run /kbd-new-phase first"
jq -e . "$wp" >/dev/null 2>&1 || die "malformed waypoint at $wp"

# Runtime-authority mode selects siblings through the canonical phase graph.
KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}"
export KBD_ORCHESTRATOR_ROOT
if [[ -f "$KBD_ORCHESTRATOR_ROOT/shared/lib/runtime-authority.sh" ]]; then
  . "$KBD_ORCHESTRATOR_ROOT/shared/lib/runtime-authority.sh"
fi
if command -v kbd_runtime_authoritative >/dev/null 2>&1 && kbd_runtime_authoritative "."; then
  runtime_state="$(kbd_runtime_status_json ".")" || die "runtime status unavailable"
  active_id="$(printf '%s' "$runtime_state" | jq -r '.activePath.phaseId // empty')"
  [[ -n "$active_id" ]] || die "runtime has no active phase"
  parent_id="$(printf '%s' "$runtime_state" | jq -r --arg id "$active_id" '.phases[$id].parentPhaseId // empty')"
  prior=""
  if [[ -n "$parent_id" ]]; then
    prior="$(printf '%s' "$runtime_state" | jq -r --arg id "$active_id" '.phases[$id].slug')"
    parent_path="$(printf '%s' "$runtime_state" | jq -c '.activePath.phasePath[0:-1]')"
  else
    parent_id="$active_id"
    parent_path="$(printf '%s' "$runtime_state" | jq -c '.activePath.phasePath')"
  fi
  children="$(printf '%s' "$runtime_state" | jq -r --arg parent "$parent_id" '
    .phases | to_entries[] |
    select(.value.parentPhaseId == $parent) |
    [.key, .value.slug] | @tsv
  ')"
  [[ -n "$children" ]] || die "no children defined — run /kbd-new-child first"
  if [[ -n "$target" ]]; then
    next_row="$(printf '%s\n' "$children" | awk -F '\t' -v target="$target" '$2==target{print;exit}')"
    [[ -n "$next_row" ]] || die "no such child: $target"
  elif [[ -z "$prior" ]]; then
    next_row="$(printf '%s\n' "$children" | head -n 1)"
  else
    next_row="$(printf '%s\n' "$children" | awk -F '\t' -v prior="$prior" 'seen{print;exit} $2==prior{seen=1}')"
    [[ -n "$next_row" ]] || die "already on last child '$prior'"
  fi
  next_id="${next_row%%	*}"
  next="${next_row#*	}"
  mutation="$(kbd_runtime_mutation_args "." "phase-next-child:${parent_id}:${next_id}")" ||
    die "failed to resolve current revision"
  revision="$(printf '%s\n' "$mutation" | sed -n '1p')"
  ancestor_args=()
  while IFS= read -r ancestor; do
    [[ -n "$ancestor" ]] || continue
    ancestor_args+=(--ancestor "$ancestor")
  done < <(printf '%s' "$parent_path" | jq -r '.[]')
  prometheus kbd --path . phase activate \
    --expected-revision "$revision" \
    --command-id "phase-next-child:${parent_id}:${next_id}" \
    --id "$next_id" "${ancestor_args[@]}" \
    --exact-next-work "/kbd-assess ${next}" >/dev/null
  printf '\nCompleted kbd-next-child — now on %s\n' "$next"
  printf '  from: %s\n' "${prior:-none}"
  printf '  to:   %s\n' "$next"
  printf '  Next: /kbd-assess %s\n' "$next"
  exit 0
fi

parent="$(jq -r '.phase // ""' "$wp")"
[[ -n "$parent" ]] || die "no active phase"

# Children as newline-separated list
children="$(jq -r '.childPhases // [] | .[]' "$wp")"
[[ -n "$children" ]] || die "no children defined — run /kbd-new-child first"
total="$(printf '%s\n' "$children" | wc -l | tr -d ' ')"

prior="$(jq -r '.childPointer // ""' "$wp")"

# Resolve next
if [[ -n "$target" ]]; then
  if ! printf '%s\n' "$children" | grep -qFx "$target"; then
    avail="$(printf '%s\n' "$children" | tr '\n' ' ')"
    die "no such child: $target (available: $avail)"
  fi
  next="$target"
else
  if [[ -z "$prior" ]]; then
    next="$(printf '%s\n' "$children" | head -n 1)"
  else
    next="$(printf '%s\n' "$children" | awk -v p="$prior" 'seen{print;exit} $0==p{seen=1}')"
    [[ -n "$next" ]] || die "already on last child '$prior' — run /kbd-reflect, then /kbd-next-phase"
  fi
fi

# Hook subsystem
hooks_avail=0
if [[ -f "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh" && -f "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh" ]]; then
  # shellcheck source=/dev/null
  . "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
  # shellcheck source=/dev/null
  . "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"
  hooks_avail=1
else
  warn "hooks subsystem unavailable (continuing without hook fires)"
fi

from_label="(none)"
if [[ -n "$prior" ]]; then
  from_label="$prior"
  if [[ "$hooks_avail" == "1" ]]; then
    prior_index="$(printf '%s\n' "$children" | grep -nFx "$prior" | head -1 | cut -d: -f1)"
    kbd_hooks_fire child after "$prior" "$prior_index" "$total" \
      || warn "child:after hook fire failed"
  fi
fi

# Atomic waypoint update. Keep path[] consistent: switching siblings at the
# active parent re-points the trailing element to the new child. We rebuild
# path[] as <parent-chain-without-trailing-pointer> + [next].
now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
new_path_json="null"
if [[ "$hooks_avail" == "1" ]]; then
  full_chain="$(_kbd_path_from_waypoint "$wp" 2>/dev/null || true)"
  if [[ -n "$full_chain" ]]; then
    # shellcheck disable=SC2206
    ft=($full_chain); fd="${#ft[@]}"
    if [[ "$fd" -gt 1 && "${ft[$((fd-1))]}" == "$prior" ]]; then
      base=("${ft[@]:0:$((fd-1))}")
    else
      base=("${ft[@]}")
    fi
    base_chain="$(printf '%s ' "${base[@]}")"; base_chain="${base_chain% }"
    new_path_json="$(jq -cn --argjson b "$(printf '%s' "$base_chain" | jq -R 'split(" ")')" --arg n "$next" '$b + [$n]')"
  fi
fi
jq --arg ptr "$next" --arg parent "$parent" --arg now "$now" --argjson np "$new_path_json" '
  .childPointer     = $ptr |
  (if $np != null then .path = $np else . end) |
  .currentTask      = ("run kbd-assess for " + $parent + "/" + $ptr) |
  .exactNextCommand = ("/kbd-assess " + $parent + "/" + $ptr) |
  .updatedAt        = $now
' "$wp" > "$wp.tmp"
mv -f "$wp.tmp" "$wp"

# child:before for new active child
if [[ "$hooks_avail" == "1" ]]; then
  next_index="$(printf '%s\n' "$children" | grep -nFx "$next" | head -1 | cut -d: -f1)"
  kbd_hooks_fire child before "$next" "$next_index" "$total" \
    || warn "child:before hook fire failed"
fi

printf '\nCompleted kbd-next-child — now on %s/%s\n' "$parent" "$next"
printf '  from: %s\n' "$from_label"
printf '  to:   %s\n' "$next"
printf '  Next: /kbd-assess %s/%s\n' "$parent" "$next"
