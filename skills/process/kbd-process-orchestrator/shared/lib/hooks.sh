# shellcheck shell=bash
# kbd-process-orchestrator/shared/lib/hooks.sh
#
# KBD hooks dispatcher. Source this file; it does not run anything on import.
#
#   . shared/lib/waypoint.sh   # required for chain_separator + waypoint_load
#   . shared/lib/hooks.sh
#   kbd_hooks_fire phase before "$phase" 1 1
#
# Public:
#   kbd_hooks_fire <kind> <edge> <name> [<index>=1] [<total>=1]
#
# Layers (loaded in order, last wins on override):
#   builtin  — $KBD_ORCHESTRATOR_ROOT/hooks/hooks.json
#   user     — $KBD_ORCHESTRATOR_ROOT/hooks/user.json (optional)
#   project  — $PWD/.kbd-orchestrator/hooks-config.json (optional)
#
# Bash 3.2 compatible. Requires `jq`. Soft-fails when jq is absent.

# Be self-sufficient: hooks.sh uses chain_separator/waypoint_chain/waypoint_load
# from waypoint.sh. The call sites guard with `declare -F`, but that guard is not
# reliable across shells (notably zsh), so source waypoint.sh here when its
# functions are absent. Best-effort: never abort import if it can't be found.
if ! command -v waypoint_load >/dev/null 2>&1; then
  _kbd_hooks_self_dir="$(cd "$(dirname "${BASH_SOURCE[0]:-$0}")" 2>/dev/null && pwd -P)"
  if [ -n "${_kbd_hooks_self_dir:-}" ] && [ -f "$_kbd_hooks_self_dir/waypoint.sh" ]; then
    # shellcheck source=/dev/null
    . "$_kbd_hooks_self_dir/waypoint.sh"
  fi
  unset _kbd_hooks_self_dir
fi

_kbd_hooks_debug() {
  [[ "${KBD_HOOK_DEBUG:-}" == "1" ]] && printf '[hooks] %s\n' "$*" >&2
  :
}

_kbd_hooks_warn() {
  printf 'hooks: %s\n' "$*" >&2
}

# Map a legacy event name to canonical <kind>:<edge>.
# Echoes the canonical form, or the input unchanged if no mapping applies.
# Returns 1 for situational events that intentionally don't map (caller keeps original).
_kbd_hooks_normalise_event() {
  local ev="$1"
  case "$ev" in
    on_phase_complete)      echo "phase:after"   ;;
    on_plan_complete)       echo "plan:after"    ;;
    on_reflection_complete) echo "reflect:after" ;;
    on_assessment_complete) echo "assess:after"  ;;
    on_change_complete)     echo "on_change_complete" ;;   # sentinel — runtime check in fire()
    on_blocker_detected|on_cross_tool_handoff) echo "$ev" ;;
    *:begin) echo "${ev%:begin}:before" ;;
    *:end)   echo "${ev%:end}:after" ;;
    # Already canonical or invalid; the matcher decides.
    *) echo "$ev" ;;
  esac
}

# Echo a single jq expression that, given a hooks-config JSON object on stdin,
# emits one compact JSON object per entry shaped:
#   { id, event, mode, command, timeout, on_failure, layer }
_kbd_hooks_collect_one() {
  local file="$1" layer="$2"
  [[ -f "$file" ]] || return 0
  command -v jq >/dev/null 2>&1 || return 0

  jq -c --arg layer "$layer" --arg path "$file" '
    .hooks // [] | to_entries | .[] |
    .value as $h |
    {
      id:         ($h.id // ($layer + "/" + ($h.event // "unknown") + "/" + (.key | tostring))),
      event:      ($h.event // ""),
      mode:       ($h.mode  // "augment"),
      command:    ($h.action.command // $h.action.target // ""),
      timeout:    ($h.action.timeout // 15),
      on_failure: ($h.action.on_failure // "warn"),
      enabled:    ($h.enabled // true),
      layer:      $layer
    }
  ' "$file" 2>/dev/null
}

# Collect matching entries across all three layers, normalising events and
# filtering by (kind, edge). Emits one JSON entry per line.
_kbd_hooks_collect() {
  local kind="$1" edge="$2"
  local target="$kind:$edge"
  local builtin="$KBD_ORCHESTRATOR_ROOT/hooks/hooks.json"
  local user="$KBD_ORCHESTRATOR_ROOT/hooks/user.json"
  local project=".kbd-orchestrator/hooks-config.json"

  {
    _kbd_hooks_collect_one "$builtin" "builtin"
    _kbd_hooks_collect_one "$user"    "user"
    _kbd_hooks_collect_one "$project" "project"
  } | while IFS= read -r entry; do
    [[ -n "$entry" ]] || continue
    local raw norm
    raw="$(printf '%s' "$entry" | jq -r '.event')"
    [[ "${raw:-}" ]] || continue
    norm="$(_kbd_hooks_normalise_event "$raw")"
    _kbd_hooks_debug "normalised $raw → $norm (id=$(printf '%s' "$entry" | jq -r '.id'))"

    # Match: exact, wildcard kind, wildcard edge, or full wildcard.
    case "$norm" in
      "$target"|"$kind:*"|"*:$edge"|"*:*") : ;;
      *) continue ;;
    esac

    # Skip disabled entries.
    local enabled
    enabled="$(printf '%s' "$entry" | jq -r '.enabled')"
    [[ "$enabled" == "false" ]] && continue

    # Re-emit with normalised event embedded for downstream consumers.
    printf '%s\n' "$entry" | jq -c --arg ev "$norm" '. + {normEvent: $ev}'
  done
}

# Override resolution. Reads matching entries (JSONL) on stdin, emits the
# entries that should actually fire on stdout.
_kbd_hooks_resolve_overrides() {
  local kind="$1" edge="$2"
  local entries
  entries="$(cat)"
  [[ -n "$entries" ]] || return 0

  # Defensive: tolerate non-JSON lines on the input stream (e.g. stray xtrace,
  # an accidental echo from an augment, or a polluted shell). Without this a
  # single non-JSON line aborts the override jq with a parse error and breaks
  # all dispatch. `fromjson? // empty` drops anything that isn't valid JSON.
  if command -v jq >/dev/null 2>&1; then
    entries="$(printf '%s\n' "$entries" | jq -cR 'fromjson? // empty' 2>/dev/null)"
    [[ -n "$entries" ]] || return 0
  fi

  # Partition.
  local overrides augments
  overrides="$(printf '%s\n' "$entries" | jq -c 'select(.mode == "override")')"
  augments="$(printf '%s\n' "$entries"  | jq -c 'select(.mode != "override")')"

  # Pick override winner: project > user > builtin; within layer, last wins.
  local winner=""
  local losers=""
  if [[ -n "$overrides" ]]; then
    winner="$(printf '%s\n' "$overrides" | jq -s -c '
      [.[] | select(.layer == "project")] as $p |
      [.[] | select(.layer == "user")]    as $u |
      [.[] | select(.layer == "builtin")] as $b |
      if   ($p | length) > 0 then $p[-1]
      elif ($u | length) > 0 then $u[-1]
      elif ($b | length) > 0 then $b[-1]
      else empty end
    ' 2>/dev/null)"
    if [[ -n "$winner" ]]; then
      losers="$(printf '%s\n' "$overrides" | jq -c --argjson w "$winner" '
        select(. != $w)
      ' 2>/dev/null)"
    fi
  fi

  if [[ -n "$losers" && "$losers" != "null" ]]; then
    local winner_id loser_ids
    winner_id="$(printf '%s' "$winner" | jq -r '.layer + "/" + .id')"
    loser_ids="$(printf '%s\n' "$losers" | jq -r '.layer + "/" + .id' | paste -sd ',' -)"
    _kbd_hooks_warn "override conflict on $kind:$edge — using $winner_id, suppressing $loser_ids"
  fi

  # When an override resolves, suppress the built-in default reporter only.
  # All non-default augments still fire.
  if [[ -n "$winner" && "$winner" != "null" ]]; then
    augments="$(printf '%s\n' "$augments" | jq -c 'select(.id != "builtin/report-progress" and .id != "report-progress")')"
    printf '%s\n' "$winner"
  fi
  [[ -n "$augments" ]] && printf '%s\n' "$augments"
}

# Resolve the active phase directory (or empty string if none).
_kbd_hooks_active_phase_dir() {
  local wp=".kbd-orchestrator/current-waypoint.json"
  [[ -f "$wp" ]] || { printf ''; return 0; }
  command -v jq >/dev/null 2>&1 || { printf ''; return 0; }
  local phase
  phase="$(jq -r '.phase // ""' "$wp" 2>/dev/null)"
  [[ -n "$phase" ]] || { printf ''; return 0; }
  local dir=".kbd-orchestrator/phases/$phase"
  [[ -d "$dir" ]] && printf '%s' "$dir"
}

_kbd_hooks_log_path() {
  local phase_dir
  phase_dir="$(_kbd_hooks_active_phase_dir)"
  if [[ -n "$phase_dir" ]]; then
    printf '%s/hooks.log.jsonl' "$phase_dir"
  else
    mkdir -p .kbd-orchestrator 2>/dev/null || true
    printf '.kbd-orchestrator/hooks.log.jsonl'
  fi
}

# Append a single JSON object to the hook log, with locking. Best-effort.
_kbd_hooks_log_append() {
  local line="$1"
  local log
  log="$(_kbd_hooks_log_path)"
  local lockdir="$log.lockd"

  if command -v flock >/dev/null 2>&1; then
    (
      flock -w 5 9 || exit 0
      printf '%s\n' "$line" >> "$log"
    ) 9> "$log.lock"
    return 0
  fi

  # mkdir-based lock fallback (macOS).
  local i=0
  while ! mkdir "$lockdir" 2>/dev/null; do
    i=$((i+1))
    [[ $i -ge 50 ]] && { printf '%s\n' "$line" >> "$log"; return 0; }
    sleep 0.1
  done
  printf '%s\n' "$line" >> "$log"
  rmdir "$lockdir" 2>/dev/null || true
}

# Run a single resolved entry.
_kbd_hooks_run() {
  local entry="$1" kind="$2" edge="$3" name="$4" index="$5" total="$6"

  local id command timeout on_failure layer mode
  id="$(printf '%s' "$entry" | jq -r '.id')"
  command="$(printf '%s' "$entry" | jq -r '.command')"
  timeout="$(printf '%s' "$entry" | jq -r '.timeout')"
  on_failure="$(printf '%s' "$entry" | jq -r '.on_failure')"
  layer="$(printf '%s' "$entry" | jq -r '.layer')"
  mode="$(printf '%s' "$entry" | jq -r '.mode')"

  [[ -n "$command" ]] || return 0

  # Build context env. waypoint.sh is expected to be sourced; chain_separator
  # is optional (fallback to "› " inline).
  local sep="› "
  if command -v chain_separator >/dev/null 2>&1; then sep="$(chain_separator)"; fi

  local phase_path child_path source_tool started_at
  if command -v waypoint_load >/dev/null 2>&1 && [[ -f .kbd-orchestrator/current-waypoint.json ]]; then
    local kvs
    kvs="$(waypoint_load .kbd-orchestrator/current-waypoint.json 2>/dev/null || true)"
    local _phase _parent _ptr _src
    _phase="$(printf '%s\n' "$kvs"  | sed -n 's/^phase=//p')"
    _parent="$(printf '%s\n' "$kvs" | sed -n 's/^parentPhase=//p')"
    _ptr="$(printf '%s\n' "$kvs"    | sed -n 's/^childPointer=//p')"
    _src="$(printf '%s\n' "$kvs"    | sed -n 's/^sourceTool=//p')"

    phase_path="$(command -v waypoint_chain >/dev/null 2>&1 \
      && waypoint_chain "$_parent" "$_phase" "$_ptr" \
      || printf '%s' "$_phase")"
    child_path="${_ptr:-}"
    source_tool="${_src:-unknown}"
  else
    phase_path=""
    child_path=""
    source_tool="unknown"
  fi
  started_at="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

  # Run the command with timeout. macOS lacks `timeout(1)` by default; degrade.
  local rc stderr_snippet stderr_file
  stderr_file="$(mktemp -t kbdhook.XXXX)"
  # The hook command's stderr is the canonical output channel (the default
  # reporter writes there). We tee it: visible to the parent process AND
  # captured to a temp file so we can include a snippet in the JSONL log
  # entry on failure. Process substitution requires bash, which we already
  # depend on.
  if command -v timeout >/dev/null 2>&1; then
    KBD_HOOK_KIND="$kind" KBD_HOOK_EDGE="$edge" KBD_HOOK_NAME="$name" \
    KBD_HOOK_INDEX="$index" KBD_HOOK_TOTAL="$total" \
    KBD_HOOK_PHASE_PATH="$phase_path" KBD_HOOK_CHILD_PATH="$child_path" \
    KBD_HOOK_SOURCE_TOOL="$source_tool" KBD_HOOK_STARTED_AT="$started_at" \
    PHASE="$phase_path" STEP="$kind" EVENT="$edge" TIMESTAMP="$started_at" \
      timeout "$timeout" bash -c "$command" 2> >(tee "$stderr_file" >&2)
    rc=$?
  else
    KBD_HOOK_KIND="$kind" KBD_HOOK_EDGE="$edge" KBD_HOOK_NAME="$name" \
    KBD_HOOK_INDEX="$index" KBD_HOOK_TOTAL="$total" \
    KBD_HOOK_PHASE_PATH="$phase_path" KBD_HOOK_CHILD_PATH="$child_path" \
    KBD_HOOK_SOURCE_TOOL="$source_tool" KBD_HOOK_STARTED_AT="$started_at" \
    PHASE="$phase_path" STEP="$kind" EVENT="$edge" TIMESTAMP="$started_at" \
      bash -c "$command" 2> >(tee "$stderr_file" >&2)
    rc=$?
  fi
  wait 2>/dev/null || true   # let the tee subshell flush before we read
  stderr_snippet="$(head -c 200 "$stderr_file" 2>/dev/null | tr '\n' ' ')"
  rm -f "$stderr_file"

  # JSONL append.
  local entry_log
  entry_log="$(jq -c -n \
    --arg ts "$started_at" --arg kind "$kind" --arg edge "$edge" --arg name "$name" \
    --argjson index "${index:-1}" --argjson total "${total:-1}" \
    --arg phasePath "$phase_path" --arg sourceTool "$source_tool" \
    --arg hookId "$id" --arg layer "$layer" --arg mode "$mode" \
    --argjson status "$rc" --arg stderrSnippet "$stderr_snippet" '
      {ts:$ts, kind:$kind, edge:$edge, name:$name, index:$index, total:$total,
       phasePath:$phasePath, sourceTool:$sourceTool, hookId:$hookId,
       layer:$layer, mode:$mode, status:$status,
       stderrSnippet: (if $status == 0 then null else $stderrSnippet end)}
      | with_entries(select(.value != null))
    ')"
  _kbd_hooks_log_append "$entry_log"

  if [[ "$rc" -ne 0 ]]; then
    case "$on_failure" in
      error)  _kbd_hooks_warn "hook $id failed (exit $rc); on_failure=error"; return "$rc" ;;
      ignore) : ;;
      *)      _kbd_hooks_warn "hook $id failed (exit $rc): $stderr_snippet" ;;
    esac
  fi
}

# Public entry point.
kbd_hooks_fire() {
  local kind="$1" edge="$2" name="$3"
  local index="${4:-1}" total="${5:-1}"

  : "${KBD_ORCHESTRATOR_ROOT:?kbd_hooks_fire: KBD_ORCHESTRATOR_ROOT must be set}"

  if ! command -v jq >/dev/null 2>&1; then
    _kbd_hooks_warn "jq not available; hooks dispatch skipped"
    return 0
  fi

  local resolved
  resolved="$(_kbd_hooks_collect "$kind" "$edge" | _kbd_hooks_resolve_overrides "$kind" "$edge")"

  while IFS= read -r entry; do
    [[ -n "$entry" ]] || continue
    _kbd_hooks_run "$entry" "$kind" "$edge" "$name" "$index" "$total"
  done <<EOF
$resolved
EOF

  # on_change_complete alias — D5 runtime conditional.
  if [[ "$kind" == "task" && "$edge" == "after" && "$index" == "$total" ]]; then
    local oc_matches
    oc_matches="$(
      {
        _kbd_hooks_collect_one "$KBD_ORCHESTRATOR_ROOT/hooks/hooks.json" "builtin"
        _kbd_hooks_collect_one "$KBD_ORCHESTRATOR_ROOT/hooks/user.json"  "user"
        _kbd_hooks_collect_one ".kbd-orchestrator/hooks-config.json"     "project"
      } | jq -c 'select(.event == "on_change_complete" and (.enabled // true))'
    )"
    while IFS= read -r entry; do
      [[ -n "$entry" ]] || continue
      _kbd_hooks_run "$entry" "$kind" "$edge" "$name" "$index" "$total"
    done <<EOF
$oc_matches
EOF
  fi
}
