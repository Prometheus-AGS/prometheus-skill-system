#!/usr/bin/env bash
# skills/kbd-child-exit/kbd-child-exit.sh
# Exit the active child loop: write its handoff-out, roll its progress up to the
# parent, pop path[], and restore the parent's cursor. Control returns to the
# parent loop.
#
# Companion: `enter` mode descends into the selected child (set path + clear
# childPointer) so the next /kbd-new-child nests under it. Together enter/exit
# are the child navigation pair.
#
# Usage:
#   kbd-child-exit.sh           # exit the active (deepest) child
#   kbd-child-exit.sh --enter   # descend into the selected childPointer

set -euo pipefail
die()  { printf 'kbd-child-exit: %s\n' "$*" >&2; exit 1; }
warn() { printf 'kbd-child-exit: warn: %s\n' "$*" >&2; }

mode="exit"
[[ "${1:-}" == "--enter" ]] && mode="enter"

command -v jq >/dev/null 2>&1 || die "jq is required"
wp=".kbd-orchestrator/current-waypoint.json"
[[ -f "$wp" ]] || die "no current-waypoint.json"
jq -e . "$wp" >/dev/null 2>&1 || die "malformed waypoint"

KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}"
export KBD_ORCHESTRATOR_ROOT
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
[[ -f "$KBD_ORCHESTRATOR_ROOT/shared/lib/rollup.sh" ]] && . "$KBD_ORCHESTRATOR_ROOT/shared/lib/rollup.sh"
hooks_avail=0
[[ -f "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh" ]] && { . "$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"; hooks_avail=1; }
runtime_avail=0
if [[ -f "$KBD_ORCHESTRATOR_ROOT/shared/lib/runtime-authority.sh" ]]; then
  . "$KBD_ORCHESTRATOR_ROOT/shared/lib/runtime-authority.sh"
  kbd_runtime_authoritative "." && runtime_avail=1
fi

now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

if [[ "$mode" == "enter" ]]; then
  if [[ "$runtime_avail" == "1" ]]; then
    label="$(jq -r '.path | join(" › ")' "$wp" 2>/dev/null)"
    printf '\nEntered child — runtime active path is %s\n' "$label"
    exit 0
  fi
  # Descend into the selected childPointer: path becomes <current-node>/<pointer>,
  # childPointer cleared so /kbd-new-child nests under it.
  ptr="$(jq -r '.childPointer // ""' "$wp")"
  [[ -n "$ptr" ]] || die "no childPointer selected — run /kbd-next-child <name> first"
  chain="$(_kbd_path_from_waypoint "$wp")"
  # shellcheck disable=SC2206
  toks=($chain); depth="${#toks[@]}"
  # If the trailing token already equals the pointer (selected-not-entered),
  # the entered path IS the current chain; else append the pointer.
  if [[ "$depth" -ge 1 && "${toks[$((depth-1))]}" == "$ptr" ]]; then
    entered=("${toks[@]}")
  else
    entered=("${toks[@]}" "$ptr")
  fi
  node_dir="$(kbd_node_dir "${entered[@]}")"
  [[ -d "$node_dir" ]] || die "child node dir not found: $node_dir"
  new_path="$(printf '%s\n' "${entered[@]}" | jq -R . | jq -cs .)"
  label="$(kbd_node_chain "${entered[@]}")"
  jq --argjson p "$new_path" --arg label "$label" --arg now "$now" \
    '.path = $p | .childPointer = null | .currentTask = ("work inside " + $label) |
     .exactNextCommand = ("/kbd-assess " + $label) | .updatedAt = $now' "$wp" > "$wp.tmp"
  mv -f "$wp.tmp" "$wp"
  printf '\nEntered child — now inside %s\n  /kbd-new-child here nests under this node.\n' "$label"
  exit 0
fi

# --- exit mode --------------------------------------------------------------
chain="$(_kbd_path_from_waypoint "$wp")"
# shellcheck disable=SC2206
toks=($chain); depth="${#toks[@]}"
[[ "$depth" -gt 1 ]] || die "not inside a child (path depth $depth) — nothing to exit"

child_name="${toks[$((depth-1))]}"
child_dir="$(kbd_node_dir "${toks[@]}")"
parent_tokens=("${toks[@]:0:$((depth-1))}")
parent_dir="$(kbd_node_dir "${parent_tokens[@]}")"
parent_label="$(kbd_node_chain "${parent_tokens[@]}")"

# Precondition: child must have a reflection.
[[ -f "$child_dir/reflection.md" ]] \
  || die "child '$child_name' has no reflection.md — run /kbd-reflect for the child first"

# --- handoff-out.md ---------------------------------------------------------
status="UNKNOWN"
if [[ -f "$child_dir/progress.json" ]]; then
  rc="$(jq -r '.reflect_complete // false' "$child_dir/progress.json")"
  [[ "$rc" == "true" ]] && status="DONE" || status="INCOMPLETE"
fi
{
  printf '# Handoff out — %s\n\n' "$(kbd_node_chain "${toks[@]}")"
  printf '**Status:** %s\n\n' "$status"
  printf '## Deliverables\n\n<!-- TBD: paths to artifacts this child produced -->\n\n'
  printf '## Goal completion\n\nSee reflection.md. Status: %s.\n\n' "$status"
  printf '## Unresolved items\n\n<!-- TBD -->\n\n'
  printf '## Recommendations to the parent (%s)\n\n<!-- TBD: what the parent should do with this result -->\n' "$parent_label"
} > "$child_dir/handoff-out.md.tmp"
mv -f "$child_dir/handoff-out.md.tmp" "$child_dir/handoff-out.md"

if [[ "$runtime_avail" == "1" ]]; then
  runtime_state="$(kbd_runtime_status_json ".")" || die "runtime status unavailable"
  path_count="$(printf '%s' "$runtime_state" | jq -r '.activePath.phasePath | length')"
  [[ "$path_count" -gt 1 ]] || die "runtime is not inside a child phase"
  parent_id="$(printf '%s' "$runtime_state" | jq -r '.activePath.phasePath[-2]')"
  ancestor_args=()
  while IFS= read -r ancestor; do
    [[ -n "$ancestor" ]] || continue
    ancestor_args+=(--ancestor "$ancestor")
  done < <(printf '%s' "$runtime_state" | jq -r '.activePath.phasePath[0:-2][]?')
  mutation="$(kbd_runtime_mutation_args "." "phase-exit:${child_name}")" || die "writer lease required"
  revision="$(printf '%s\n' "$mutation" | sed -n '1p')"
  lease_id="$(printf '%s\n' "$mutation" | sed -n '3p')"
  fencing_token="$(printf '%s\n' "$mutation" | sed -n '4p')"
  prometheus kbd --path . phase activate \
    --expected-revision "$revision" --command-id "phase-exit:${child_name}" \
    --lease-id "$lease_id" --fencing-token "$fencing_token" \
    --id "$parent_id" "${ancestor_args[@]}" \
    --exact-next-work "/kbd-status" >/dev/null
  [[ "$hooks_avail" == "1" ]] &&
    kbd_hooks_fire child after "$child_name" "$depth" "$depth" ||
    warn "child:after hook fire failed"
  printf '\nCompleted kbd-child-exit — exited %s\n' "$(kbd_node_chain "${toks[@]}")"
  printf '  status:   %s\n' "$status"
  printf '  handoff:  %s\n' "$child_dir/handoff-out.md"
  printf '  resumed:  %s\n' "$parent_label"
  printf '  Next:     /kbd-status\n'
  exit 0
fi

# --- roll progress up the ancestor chain ------------------------------------
if command -v kbd_rollup_chain >/dev/null 2>&1; then
  kbd_rollup_chain "${toks[@]}" || warn "rollup failed (continuing)"
fi

# --- pop path[]; restore parent cursor --------------------------------------
new_path="$(printf '%s\n' "${parent_tokens[@]}" | jq -R . | jq -cs .)"
# Parent pointer: clear it (we are back at the parent node, no child selected).
jq --argjson p "$new_path" --arg label "$parent_label" --arg now "$now" \
  '.path = $p | .childPointer = null |
   .currentTask = ("resumed " + $label + " after child exit") |
   .exactNextCommand = ("/kbd-status") | .updatedAt = $now' "$wp" > "$wp.tmp"
mv -f "$wp.tmp" "$wp"

# --- fire child:after -------------------------------------------------------
if [[ "$hooks_avail" == "1" ]]; then
  kbd_hooks_fire child after "$child_name" "$depth" "$depth" || warn "child:after hook fire failed"
fi

printf '\nCompleted kbd-child-exit — exited %s\n' "$(kbd_node_chain "${toks[@]}")"
printf '  status:   %s\n' "$status"
printf '  handoff:  %s\n' "$child_dir/handoff-out.md"
printf '  rolled up into: %s/progress.json (children.%s)\n' "$parent_dir" "$child_name"
printf '  resumed:  %s\n' "$parent_label"
printf '  Next:     /kbd-status\n'
