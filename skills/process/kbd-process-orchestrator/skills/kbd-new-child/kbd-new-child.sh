#!/usr/bin/env bash
# skills/kbd-new-child/kbd-new-child.sh
# Create a child phase under the active top-level phase.

set -euo pipefail
die()  { printf 'kbd-new-child: %s\n' "$*" >&2; exit 1; }
warn() { printf 'kbd-new-child: warn: %s\n' "$*" >&2; }

name="${1:-}"
[[ -n "$name" ]] || die "usage: kbd-new-child.sh <child-name> [goal-1] [goal-2] …"
shift
goals=("$@")

# Validation
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

parent="$(jq -r '.phase // ""' "$wp")"
[[ -n "$parent" ]] || die "no active phase — run /kbd-new-phase first"

# Single-level nesting only
this_parent="$(jq -r '.parentPhase // ""' "$wp")"
[[ -z "$this_parent" ]] \
  || die "current row is itself a child (parentPhase=$this_parent); only one nesting level supported"

# Duplicate check
in_list="$(jq -r --arg n "$name" '.childPhases // [] | any(. == $n)' "$wp")"
[[ "$in_list" != "true" ]] \
  || die "child '$name' already exists in childPhases — try /kbd-next-child $name"

# On-disk collision check
child_dir=".kbd-orchestrator/phases/$parent/children/$name"
[[ -e "$child_dir" ]] && die "child directory already exists: $child_dir"

now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
mkdir -p "$child_dir"

# goals.md
{
  printf '# Goals — %s/%s\n\n' "$parent" "$name"
  if [[ ${#goals[@]} -gt 0 ]]; then
    for g in "${goals[@]}"; do printf -- '- %s\n' "$g"; done
  else
    printf -- '<!-- TBD: enumerate child goals before /kbd-assess -->\n'
  fi
} > "$child_dir/goals.md.tmp"
mv -f "$child_dir/goals.md.tmp" "$child_dir/goals.md"

# progress.json (child has parentPhase set; no further nesting)
source_tool="$(jq -r '.sourceTool // ""' "$wp")"
[[ -n "$source_tool" ]] || source_tool="unknown"

jq -n \
  --arg phase "$name" --arg parent "$parent" --arg src "$source_tool" --arg now "$now" '
{
  phase: $phase,
  parentPhase: $parent,
  childPhases: [],
  childPointer: null,
  assessment_complete: false,
  plan_complete: false,
  execute_complete: false,
  reflect_complete: false,
  changes_total: 0,
  changes_completed: 0,
  completed_changes: [],
  active_change: null,
  blocked_changes: [],
  sourceTool: $src,
  createdBy: "kbd-new-child",
  updatedAt: $now
}' > "$child_dir/progress.json.tmp"
mv -f "$child_dir/progress.json.tmp" "$child_dir/progress.json"

# Compute new childPhases + pointer; verify invariants
new_children="$(jq --arg n "$name" '.childPhases // [] | . + [$n]' "$wp")"
echo "$new_children" | jq -e 'length == (unique | length)' >/dev/null \
  || die "internal: would write duplicate childPhases (refusing)"

jq --argjson cp "$new_children" --arg ptr "$name" --arg parent "$parent" --arg now "$now" '
  .childPhases      = $cp |
  .childPointer     = $ptr |
  .currentTask      = ("run kbd-assess for " + $parent + "/" + $ptr) |
  .exactNextCommand = ("/kbd-assess " + $parent + "/" + $ptr) |
  .updatedAt        = $now
' "$wp" > "$wp.tmp"
mv -f "$wp.tmp" "$wp"

# Hook fire
total="$(echo "$new_children" | jq 'length')"
index="$total"
KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}"
export KBD_ORCHESTRATOR_ROOT
hooks_lib="$KBD_ORCHESTRATOR_ROOT/shared/lib/hooks.sh"
waypoint_lib="$KBD_ORCHESTRATOR_ROOT/shared/lib/waypoint.sh"
if [[ -f "$hooks_lib" && -f "$waypoint_lib" ]]; then
  # shellcheck source=/dev/null
  . "$waypoint_lib"
  # shellcheck source=/dev/null
  . "$hooks_lib"
  kbd_hooks_fire child before "$name" "$index" "$total" \
    || warn "child:before hook fire failed (child still created)"
else
  warn "hooks subsystem unavailable (child still created)"
fi

printf '\nCompleted kbd-new-child — %s/%s ready for /kbd-assess\n' "$parent" "$name"
printf '  parent: %s\n' "$parent"
printf '  child:  %s  [%s/%s]\n' "$name" "$index" "$total"
printf '  goals:  %s\n' "$child_dir/goals.md"
printf '  Next:   /kbd-assess %s/%s\n' "$parent" "$name"
