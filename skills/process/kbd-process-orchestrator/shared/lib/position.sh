# shellcheck shell=bash
# kbd-process-orchestrator/shared/lib/position.sh
#
# Unified, DERIVED position model. `kbd_position_sync` rebuilds
# .kbd-orchestrator/position.json from the waypoint plus each node's
# progress.json on every call — it derives rather than accumulates, so the
# file can be stale by at most one write but can never diverge from the
# source state. Schema: references/schemas/position.schema.json.
#
# Source this file; it does not run anything on import.
#
#   . shared/lib/position.sh
#   kbd_position_sync            # writes <root>/.kbd-orchestrator/position.json
#
# Foreign state (.evolver/, .zeespec/) is ingested READ-ONLY as annotations —
# this lib never writes outside .kbd-orchestrator/position.json.

_pos_root() {
  local dir="$PWD"
  while [ -n "$dir" ] && [ "$dir" != "/" ]; do
    [ -d "$dir/.kbd-orchestrator" ] && { printf '%s' "$dir"; return 0; }
    dir="$(dirname "$dir")"
  done
  return 1
}

# _pos_node <node-dir> <id> <waypoint-file> <is-active 0|1>
# Emits one phase-node JSON object (without children) on stdout.
_pos_node() {
  local node_dir="$1" id="$2" wp="$3" is_active="$4"
  local prog="$node_dir/progress.json"
  local done=0 total=0 status="unknown"
  if [ -f "$prog" ] && jq empty "$prog" 2>/dev/null; then
    done="$(jq -r '.changes_completed // 0' "$prog")"
    total="$(jq -r '.changes_total // 0' "$prog")"
  fi
  if [ "$is_active" = "1" ]; then
    status="$(jq -r '.status // .stage // "unknown"' "$wp" 2>/dev/null)"
  else
    status="inactive"
  fi

  local change_json="null"
  if [ "$is_active" = "1" ]; then
    local change
    change="$(jq -r '.change // .active_change // empty' "$wp" 2>/dev/null)"
    if [ -n "$change" ] && [ -f "$prog" ]; then
      change_json="$(jq -c --arg id "$change" '
        (.changes[]? | select(.id == $id)) as $c |
        {
          type: "change",
          id: $id,
          status: ($c.status // "PENDING"),
          progress: { done: ($c.tasks_done // 0), total: ($c.tasks_total // 0) }
        }' "$prog" 2>/dev/null)"
      [ -n "$change_json" ] || change_json="$(jq -cn --arg id "$change" \
        '{type:"change", id:$id, status:"PENDING", progress:{done:0,total:0}}')"
    fi
  fi

  jq -cn \
    --arg id "$id" \
    --arg status "$status" \
    --argjson done "${done:-0}" \
    --argjson total "${total:-0}" \
    --argjson change "$change_json" \
    '{
      type: "phase",
      id: $id,
      status: $status,
      progress: { done: $done, total: $total },
      children: (if $change == null then [] else [$change] end)
    }'
}

_pos_annotations() { # <root>
  local root="$1" ann="[]"
  if [ -d "$root/.evolver" ]; then
    local evos
    evos="$(ls "$root/.evolver/evolutions" 2>/dev/null | head -3 | tr '\n' ',' | sed 's/,$//')"
    ann="$(printf '%s' "$ann" | jq -c --arg ref ".evolver/" --arg s "evolutions: ${evos:-none}" \
      '. + [{source:"evolver", ref:$ref, summary:$s}]')"
  fi
  if [ -d "$root/.zeespec" ]; then
    local subjects
    subjects="$(ls "$root/.zeespec" 2>/dev/null | head -3 | tr '\n' ',' | sed 's/,$//')"
    ann="$(printf '%s' "$ann" | jq -c --arg ref ".zeespec/" --arg s "subjects: ${subjects:-none}" \
      '. + [{source:"zeespec", ref:$ref, summary:$s}]')"
  fi
  printf '%s' "$ann"
}

kbd_position_sync() {
  command -v jq >/dev/null 2>&1 || return 0
  local root
  root="$(_pos_root)" || return 0
  local wp="$root/.kbd-orchestrator/current-waypoint.json"
  [ -f "$wp" ] && jq empty "$wp" 2>/dev/null || return 0

  local phase ptr
  phase="$(jq -r '.phase // empty' "$wp")"
  [ -n "$phase" ] || return 0
  ptr="$(jq -r '.childPointer // empty' "$wp")"

  local phase_dir="$root/.kbd-orchestrator/phases/$phase"

  # Resolve the full position chain from path[] (synthesized from v2 when
  # absent), supporting arbitrary nesting depth.
  local chain
  chain="$(jq -r '
    if (.path | type) == "array" and (.path | length) > 0 then .path | join(" ")
    else ([ (.phase // empty), (.childPointer // empty) ]
          | map(select(. != "" and . != null)) | join(" ")) end
  ' "$wp" 2>/dev/null)"
  # shellcheck disable=SC2206
  local toks=($chain)
  local cdepth="${#toks[@]}"

  local root_node cursor active_node_dir
  if [ "$cdepth" -le 1 ]; then
    root_node="$(_pos_node "$phase_dir" "$phase" "$wp" 1)"
    cursor="$(jq -cn --arg a "$phase" '[$a]')"
    active_node_dir="$phase_dir"
  else
    # Build the tree from the deepest active node outward; the root node wraps
    # its descendant chain. Cursor = the full path tokens.
    local i node_dir="$phase_dir"
    # Deepest node (active).
    local deepest="$phase_dir"
    for ((i=1; i<cdepth; i++)); do deepest="$deepest/children/${toks[$i]}"; done
    active_node_dir="$deepest"
    # Innermost node object, then wrap outward as nested children.
    local acc; acc="$(_pos_node "$deepest" "${toks[$((cdepth-1))]}" "$wp" 1)"
    local pdir="$deepest"
    for ((i=cdepth-2; i>=0; i--)); do
      pdir="$(dirname "$(dirname "$pdir")")"   # strip /children/<name>
      local wrap; wrap="$(_pos_node "$pdir" "${toks[$i]}" "$wp" 0)"
      acc="$(jq -cn --argjson p "$wrap" --argjson c "$acc" \
        '$p | .status = "active-parent" | .children = [$c]')"
    done
    root_node="$acc"
    cursor="$(printf '%s\n' "${toks[@]}" | jq -R . | jq -cs .)"
  fi

  # Extend cursor with active change and task position when present.
  local change
  change="$(jq -r '.change // .active_change // empty' "$wp")"
  if [ -n "$change" ]; then
    cursor="$(printf '%s' "$cursor" | jq -c --arg c "$change" '. + [$c]')"
    local node_dir="$active_node_dir"
    local tdone ttotal
    tdone="$(jq -r --arg id "$change" '.changes[]? | select(.id==$id) | .tasks_done // 0' "$node_dir/progress.json" 2>/dev/null)"
    ttotal="$(jq -r --arg id "$change" '.changes[]? | select(.id==$id) | .tasks_total // 0' "$node_dir/progress.json" 2>/dev/null)"
    if [ -n "${ttotal:-}" ] && [ "${ttotal:-0}" != "0" ]; then
      cursor="$(printf '%s' "$cursor" | jq -c --arg t "task:${tdone:-0}/${ttotal}" '. + [$t]')"
    fi
  fi

  local annotations tmp out="$root/.kbd-orchestrator/position.json"
  annotations="$(_pos_annotations "$root")"
  tmp="$(mktemp "$root/.kbd-orchestrator/.position.XXXXXX")" || return 0
  jq -n \
    --argjson cursor "$cursor" \
    --argjson rootNode "$root_node" \
    --argjson annotations "$annotations" \
    '{
      schemaVersion: "1",
      updatedAt: (now | todate),
      cursor: $cursor,
      root: ($rootNode + { annotations: $annotations })
    }' > "$tmp" 2>/dev/null || { rm -f "$tmp"; return 0; }
  mv "$tmp" "$out"
}
