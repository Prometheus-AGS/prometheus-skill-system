#!/usr/bin/env bash
# skills/kbd-new-child/kbd-new-child.sh
# Create a child phase under the ACTIVE node (arbitrary depth, waypoint v3).
#
# The "parent" of the new child is whatever node the waypoint path[] currently
# points at — a top-level phase, a child, a grandchild, etc. This is what makes
# arbitrary-depth nesting work. On spawn the child gets its own goals.md,
# progress.json, plus a parent→child handoff (handoff-in.md) and a
# context-isolation contract (scope.json).

set -euo pipefail
die()  { printf 'kbd-new-child: %s\n' "$*" >&2; exit 1; }
warn() { printf 'kbd-new-child: warn: %s\n' "$*" >&2; }

name="${1:-}"
[[ -n "$name" ]] || die "usage: kbd-new-child.sh <child-name> [goal-1] [goal-2] …"
shift
goals=("$@")

case "$name" in
  *..*) die "invalid name: parent traversal not allowed" ;;
  */*)  die "invalid name: slashes not allowed" ;;
  .|..) die "invalid name: '$name'" ;;
esac
[[ "$name" =~ ^[a-z0-9][a-z0-9._-]*$ ]] \
  || die "invalid name '$name': must match ^[a-z0-9][a-z0-9._-]*$"

command -v jq >/dev/null 2>&1 || die "jq is required"

wp=".kbd-orchestrator/current-waypoint.json"
[[ -f "$wp" ]] || die "no current-waypoint.json — run /kbd-new-phase first"
jq -e . "$wp" >/dev/null 2>&1 || die "malformed waypoint at $wp"

# --- Resolve the active node from path[] (v3) -------------------------------
KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}"
export KBD_ORCHESTRATOR_ROOT
waypoint_lib="$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
[[ -f "$waypoint_lib" ]] || die "waypoint.sh not found at $waypoint_lib"
# shellcheck source=/dev/null
. "$waypoint_lib"

# The PARENT of the new child is determined by path[] and childPointer together:
#
#  - "Selected but not entered" — path[]'s trailing token EQUALS childPointer.
#    This is the state right after a kbd-new-child / kbd-next-child: the child is
#    selected for traversal but the active node is still its parent. A new child
#    here is a SIBLING, so strip the trailing token to get the parent.
#  - "Entered/descended" — childPointer is cleared (or differs from path's tail).
#    An outer agent has descended into the node; a new child NESTS under the
#    deepest path[] node.
#
# Descent is therefore: set path[] to the child chain AND clear childPointer.
full_chain="$(_kbd_path_from_waypoint "$wp")"
[[ -n "$full_chain" ]] || die "could not resolve current path from waypoint"
# shellcheck disable=SC2206
full_tokens=($full_chain)
full_depth="${#full_tokens[@]}"
wp_pointer="$(jq -r '.childPointer // ""' "$wp")"

if [[ "$full_depth" -gt 1 && -n "$wp_pointer" && "${full_tokens[$((full_depth-1))]}" == "$wp_pointer" ]]; then
  # Selected-but-not-entered → sibling add: strip the trailing pointer token.
  cur_tokens=("${full_tokens[@]:0:$((full_depth-1))}")
else
  # Entered/descended (pointer cleared) → nest under the deepest node.
  cur_tokens=("${full_tokens[@]}")
fi
cur_depth="${#cur_tokens[@]}"

# Depth rail (project.json maxChildDepth, default 4; top phase = depth 1).
max_depth=4
if [[ -f .kbd-orchestrator/project.json ]]; then
  md="$(jq -r '.maxChildDepth // empty' .kbd-orchestrator/project.json 2>/dev/null || true)"
  [[ -n "$md" ]] && max_depth="$md"
fi
if [[ "$((cur_depth + 1))" -gt "$max_depth" ]]; then
  die "maxChildDepth ($max_depth) reached at $(kbd_node_chain "${cur_tokens[@]}") — cannot nest deeper"
fi

parent_node_dir="$(kbd_node_dir "${cur_tokens[@]}")"
[[ -n "$parent_node_dir" ]] || die "could not resolve parent node dir"
parent_label="$(kbd_node_chain "${cur_tokens[@]}")"

# Duplicate check against the parent node's own childPhases.
parent_progress="$parent_node_dir/progress.json"
if [[ -f "$parent_progress" ]]; then
  in_list="$(jq -r --arg n "$name" '.childPhases // [] | any(. == $n)' "$parent_progress" 2>/dev/null || echo false)"
  [[ "$in_list" != "true" ]] \
    || die "child '$name' already exists under $parent_label — try /kbd-next-child $name"
fi

child_dir="$parent_node_dir/children/$name"
[[ -e "$child_dir" ]] && die "child directory already exists: $child_dir"

now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
mkdir -p "$child_dir"

# --- goals.md ---------------------------------------------------------------
{
  printf '# Goals — %s%s%s\n\n' "$parent_label" "$(chain_separator)" "$name"
  if [[ ${#goals[@]} -gt 0 ]]; then
    for g in "${goals[@]}"; do printf -- '- %s\n' "$g"; done
  else
    printf -- '<!-- TBD: enumerate child goals before /kbd-assess -->\n'
  fi
} > "$child_dir/goals.md.tmp"
mv -f "$child_dir/goals.md.tmp" "$child_dir/goals.md"

# --- handoff-in.md (parent → child contract) --------------------------------
{
  printf '# Handoff in — %s%s%s\n\n' "$parent_label" "$(chain_separator)" "$name"
  printf '**Spawned by:** %s\n\n' "$parent_label"
  printf '## Why this child was spawned\n\n<!-- TBD: the specific sub-goal the parent could not complete inline -->\n\n'
  printf '## Inputs (paths from the parent node)\n\n- %s/assessment.md\n- %s/plan.md\n\n' "$parent_node_dir" "$parent_node_dir"
  printf '## Success criteria\n\n<!-- TBD: what "done" means for this child -->\n\n'
  printf '## Expected deliverables\n\n<!-- TBD: artifacts the parent expects back via handoff-out.md -->\n'
} > "$child_dir/handoff-in.md.tmp"
mv -f "$child_dir/handoff-in.md.tmp" "$child_dir/handoff-in.md"

# --- scope.json (context-isolation contract) --------------------------------
jq -n --arg child "$child_dir" '
{
  allowedWritePaths: [ ($child + "/**") ],
  deniedPaths: [],
  inheritsConstraints: true,
  __note: "Edit allowedWritePaths to widen the child loop'\''s write surface. .kbd-orchestrator/** and SCRATCHPAD.md are always allowed. Enforced advisorily by check-child-scope.sh."
}' > "$child_dir/scope.json.tmp"
mv -f "$child_dir/scope.json.tmp" "$child_dir/scope.json"

child_label="$(kbd_node_chain "${cur_tokens[@]}" "$name")"

# Runtime-authority mode persists the child relationship and active hierarchy
# as typed events; progress and waypoint files remain generated projections.
runtime_lib="$KBD_ORCHESTRATOR_ROOT/shared/lib/runtime-authority.sh"
if [[ -f "$runtime_lib" ]]; then
  # shellcheck source=/dev/null
  . "$runtime_lib"
fi
if command -v kbd_runtime_authoritative >/dev/null 2>&1 && kbd_runtime_authoritative "."; then
  runtime_state="$(kbd_runtime_status_json ".")" || die "runtime status unavailable"
  parent_id="$(printf '%s' "$runtime_state" | jq -r '.activePath.phaseId // empty')"
  [[ -n "$parent_id" ]] || die "runtime has no active parent phase"
  child_runtime_id="${parent_id}::${name}"
  prometheus kbd --path . phase create \
    --command-id "phase-create:${child_runtime_id}" \
    --id "$child_runtime_id" --slug "$name" --title "$name" --parent "$parent_id" >/dev/null
  ancestor_args=()
  while IFS= read -r ancestor; do
    [[ -n "$ancestor" ]] || continue
    ancestor_args+=(--ancestor "$ancestor")
  done < <(printf '%s' "$runtime_state" | jq -r '.activePath.phasePath[]?')
  prometheus kbd --path . phase activate \
    --command-id "phase-activate:${child_runtime_id}" \
    --id "$child_runtime_id" "${ancestor_args[@]}" \
    --exact-next-work "/kbd-assess ${child_label}" >/dev/null
  hooks_lib="$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"
  if [[ -f "$hooks_lib" ]]; then
    # shellcheck source=/dev/null
    . "$hooks_lib"
    kbd_hooks_fire child before "$name" "$((cur_depth + 1))" "$max_depth" \
      || warn "child:before hook fire failed (child still created)"
  fi
  printf '\nCompleted kbd-new-child — %s ready for /kbd-assess\n' "$child_label"
  printf '  parent: %s\n' "$parent_label"
  printf '  child:  %s  [depth %s]\n' "$name" "$((cur_depth + 1))"
  printf '  goals:  %s\n' "$child_dir/goals.md"
  printf '  scope:  %s\n' "$child_dir/scope.json"
  printf '  Next:   /kbd-assess %s\n' "$child_label"
  exit 0
fi

# --- progress.json ----------------------------------------------------------
source_tool="$(jq -r '.sourceTool // ""' "$wp")"
[[ -n "$source_tool" ]] || source_tool="unknown"
jq -n \
  --arg phase "$name" --arg parent "${cur_tokens[$((cur_depth-1))]}" --arg src "$source_tool" --arg now "$now" '
{
  schemaVersion: "2",
  phase: $phase,
  parentPhase: $parent,
  childPhases: [],
  childPointer: null,
  assessment_complete: false,
  plan_complete: false,
  execute_complete: false,
  reflect_complete: false,
  implementation_total: 0,
  implementation_completed: 0,
  changes_total: 0,
  changes_completed: 0,
  completion: {
    primaryCounter: "implementation",
    implementation: { completed: 0, total: 0, status: "PENDING" },
    evidence: { status: "NOT_TRACKED", summary: null, blockers: [] },
    certification: { status: "NOT_TRACKED", summary: null, blockers: [] },
    publication: { status: "NOT_TRACKED", summary: null, blockers: [] }
  },
  completed_changes: [],
  active_change: null,
  blocked_changes: [],
  changes: [],
  last_updated: $now,
  last_updated_by: "kbd-new-child",
  sourceTool: $src,
  createdBy: "kbd-new-child",
  updatedAt: $now
}' > "$child_dir/progress.json.tmp"
mv -f "$child_dir/progress.json.tmp" "$child_dir/progress.json"

# --- Register the child on the PARENT node's progress.json ------------------
# The parent's childPhases/childPointer always live on its own progress.json.
# When the parent IS a child node (depth > 1) the parent has a progress.json
# under its child dir; at the top level the parent's progress.json is the
# top-level phase's. Compute the appended list once for reuse.
new_children='["'"$name"'"]'
if [[ -f "$parent_progress" ]]; then
  new_children="$(jq -c --arg n "$name" '.childPhases // [] | . + [$n]' "$parent_progress")"
  echo "$new_children" | jq -e 'length == (unique | length)' >/dev/null \
    || die "internal: would write duplicate childPhases on parent (refusing)"
  jq --argjson cp "$new_children" --arg ptr "$name" --arg now "$now" \
    '.childPhases = $cp | .childPointer = $ptr | .updatedAt = $now' \
    "$parent_progress" > "$parent_progress.tmp"
  mv -f "$parent_progress.tmp" "$parent_progress"
fi

# --- Update the waypoint ----------------------------------------------------
# Push the child onto path[] (canonical, all depths). At the TOP LEVEL
# (cur_depth == 1) ALSO maintain the v2 childPhases/childPointer on the waypoint
# for backward compatibility (existing tools/tests read them there). For deeper
# levels, childPhases lives only on the parent node's progress.json + path[].
parent_chain="$(printf '%s ' "${cur_tokens[@]}")"; parent_chain="${parent_chain% }"
new_path="$(jq -cn --argjson base "$(printf '%s' "$parent_chain" | jq -R 'split(" ")')" --arg n "$name" '$base + [$n]')"
if [[ "$cur_depth" -eq 1 ]]; then
  jq --argjson path "$new_path" --argjson cp "$new_children" --arg ptr "$name" --arg label "$child_label" --arg now "$now" '
    .path             = $path |
    .childPhases      = $cp |
    .childPointer     = $ptr |
    .currentTask      = ("run kbd-assess for " + $label) |
    .exactNextCommand = ("/kbd-assess " + $label) |
    .updatedAt        = $now
  ' "$wp" > "$wp.tmp"
else
  jq --argjson path "$new_path" --arg ptr "$name" --arg label "$child_label" --arg now "$now" '
    .path             = $path |
    .childPointer     = $ptr |
    .currentTask      = ("run kbd-assess for " + $label) |
    .exactNextCommand = ("/kbd-assess " + $label) |
    .updatedAt        = $now
  ' "$wp" > "$wp.tmp"
fi
mv -f "$wp.tmp" "$wp"

# --- Hook fire --------------------------------------------------------------
total="$cur_depth"
index="$((cur_depth + 1))"
hooks_lib="$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"
if [[ -f "$hooks_lib" ]]; then
  # shellcheck source=/dev/null
  . "$hooks_lib"
  kbd_hooks_fire child before "$name" "$index" "$max_depth" \
    || warn "child:before hook fire failed (child still created)"
else
  warn "hooks subsystem unavailable (child still created)"
fi

printf '\nCompleted kbd-new-child — %s ready for /kbd-assess\n' "$child_label"
printf '  parent: %s\n' "$parent_label"
printf '  child:  %s  [depth %s]\n' "$name" "$index"
printf '  goals:  %s\n' "$child_dir/goals.md"
printf '  scope:  %s\n' "$child_dir/scope.json"
printf '  Next:   /kbd-assess %s\n' "$child_label"
