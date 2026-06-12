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
#   { "<child>": { status, changes_completed, changes_total, handoff, completed_at } }
# computed from each child dir's own progress.json. Non-destructive: only the
# `children` key is rewritten.

# kbd_rollup_children <node-dir>
kbd_rollup_children() {
  local node_dir="$1"
  command -v jq >/dev/null 2>&1 || return 0
  local prog="$node_dir/progress.json"
  [ -f "$prog" ] || return 0
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
      agg="$(jq -c \
        --arg name "$cname" \
        --argjson handoff "$handoff" \
        --slurpfile c "$cp" \
        '.[$name] = {
          status: ($c[0].reflect_complete == true | if . then "DONE" else ($c[0].active_change // null | if . then "IN_PROGRESS" else "PENDING" end) end),
          changes_completed: ($c[0].changes_completed // 0),
          changes_total: ($c[0].changes_total // 0),
          handoff: $handoff,
          completed_at: ($c[0].updatedAt // null)
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
