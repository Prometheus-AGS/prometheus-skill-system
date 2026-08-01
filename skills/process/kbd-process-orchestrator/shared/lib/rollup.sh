# shellcheck shell=bash
# kbd-process-orchestrator/shared/lib/rollup.sh
#
# Aggregate child-loop progress up the ancestor chain. Source this file; no
# import side effects.
#
#   . shared/lib/rollup.sh
#   kbd_rollup_children <node-dir>      # recompute one node's children{} block
#   kbd_rollup_chain <p0> [p1] ...      # roll up every ancestor along a path
#
# Each parent node's progress.json gains a `children` object keyed by child name:
#   { "<child>": { status, implementation_completed, implementation_total,
#                    certification_status, handoff, completed_at } }
# Legacy changes_completed/changes_total aliases are emitted from the same
# implementation values for backward compatibility.
# computed from each child dir's own progress.json. Non-destructive: only the
# `children` key is rewritten.

_progress_lib="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)/progress.sh"
# shellcheck source=/dev/null
[ -f "$_progress_lib" ] && . "$_progress_lib"

# kbd_rollup_children <node-dir>
kbd_rollup_children() {
  local node_dir="$1"
  command -v jq >/dev/null 2>&1 || return 0
  local prog="$node_dir/progress.json"
  [ -f "$prog" ] || return 0
  if [ "$(jq -r '.generatedBy // empty' "$prog" 2>/dev/null)" = "kbd-runtime" ]; then
    # Child summaries are reducer-derived in runtime-authority mode.
    return 0
  fi
  local children_root="$node_dir/children"
  local agg='{}'
  if [ -d "$children_root" ]; then
    local child cp
    for child in "$children_root"/*/; do
      [ -d "$child" ] || continue
      local cname; cname="$(basename "$child")"
      cp="$child/progress.json"
      [ -f "$cp" ] && jq empty "$cp" 2>/dev/null || continue
      local handoff="null"
      [ -f "$child/handoff-out.md" ] && handoff="\"children/$cname/handoff-out.md\""
      local impl_done impl_total cert_status
      impl_done="$(kbd_progress_implementation_completed "$cp")"
      impl_total="$(kbd_progress_implementation_total "$cp")"
      cert_status="$(kbd_progress_dimension_status "$cp" certification)"
      agg="$(jq -c \
        --arg name "$cname" \
        --argjson handoff "$handoff" \
        --argjson impl_done "$impl_done" \
        --argjson impl_total "$impl_total" \
        --arg cert_status "$cert_status" \
        --slurpfile c "$cp" \
        '.[$name] = {
          status: (if
            ((($c[0].completion.implementation.status // "") | ascii_upcase) == "COMPLETE") or
            ($c[0].reflect_complete == true)
          then "DONE"
          else ($c[0].active_change // null | if . then "IN_PROGRESS" else "PENDING" end)
          end),
          implementation_completed: $impl_done,
          implementation_total: $impl_total,
          changes_completed: $impl_done,
          changes_total: $impl_total,
          certification_status: $cert_status,
          handoff: $handoff,
          completed_at: ($c[0].updatedAt // $c[0].last_updated // null)
        }' <<<"$agg")"
    done
  fi
  local tmp; tmp="$(mktemp)"
  jq --argjson ch "$agg" '.children = $ch' "$prog" > "$tmp" 2>/dev/null && mv "$tmp" "$prog" || rm -f "$tmp"
}

# kbd_rollup_chain <p0> [p1] ...  — roll up every ancestor that has children,
# from the deepest parent up to the top phase. Requires kbd_node_dir (waypoint.sh).
kbd_rollup_chain() {
  command -v kbd_node_dir >/dev/null 2>&1 || return 0
  local -a toks=("$@")
  local n="${#toks[@]}"
  # Roll up each ancestor that can have children: indices 0..n-2 are parents of
  # a deeper node; also roll up the deepest node itself in case it has children.
  local i
  for (( i=n; i>=1; i-- )); do
    local -a prefix=("${toks[@]:0:$i}")
    kbd_rollup_children "$(kbd_node_dir "${prefix[@]}")"
  done
}
