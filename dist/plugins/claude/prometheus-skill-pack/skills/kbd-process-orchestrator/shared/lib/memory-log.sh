#!/usr/bin/env bash
# shared/lib/memory-log.sh — wrapper invoked by the kbd-memory-log hook.
#
# Mirrors each hook fire into surreal-memory as a structured entity. Memory
# failures emit one bounded diagnostic and never fail the lifecycle hook.

set -euo pipefail

KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}"
# shellcheck source=/dev/null
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/memory.sh"

kbd_memory_available || exit 0
url="$(kbd_memory_url)"
# MCP-only availability has no shell REST origin and requires no mirror attempt.
[[ -n "$url" ]] || exit 0
command -v jq >/dev/null 2>&1 || exit 0
command -v curl >/dev/null 2>&1 || exit 0

project="unknown"
if [[ -f .kbd-orchestrator/project.json ]]; then
  project="$(jq -r '.project // .projectId // "unknown"' \
    .kbd-orchestrator/project.json 2>/dev/null || echo unknown)"
elif [[ -f .kbd-orchestrator/current-waypoint.json ]]; then
  project="$(jq -r '.project // .projectId // "unknown"' \
    .kbd-orchestrator/current-waypoint.json 2>/dev/null || echo unknown)"
fi

# Extract first segment of phase path for the relation target.
phase="${KBD_HOOK_PHASE_PATH%% *}"
[[ -n "$phase" ]] || phase="unknown"

# Coerce index/total to integers — they are fed to jq via --argjson, which
# aborts with a parse error on empty or non-numeric input.
idx="${KBD_HOOK_INDEX:-1}"; [[ "$idx" =~ ^[0-9]+$ ]] || idx=1
tot="${KBD_HOOK_TOTAL:-1}"; [[ "$tot" =~ ^[0-9]+$ ]] || tot=1

entity_id="$project/$phase/${KBD_HOOK_KIND:-?}/${KBD_HOOK_EDGE:-?}/$idx/${KBD_HOOK_STARTED_AT:-}"

payload="$(jq -c -n \
  --arg eid       "$entity_id" \
  --arg project   "$project" \
  --arg phase     "$phase" \
  --arg kind      "${KBD_HOOK_KIND:-}" \
  --arg edge      "${KBD_HOOK_EDGE:-}" \
  --arg name      "${KBD_HOOK_NAME:-}" \
  --argjson index "$idx" \
  --argjson total "$tot" \
  --arg phasePath "${KBD_HOOK_PHASE_PATH:-}" \
  --arg srcTool   "${KBD_HOOK_SOURCE_TOOL:-unknown}" \
  --arg ts        "${KBD_HOOK_STARTED_AT:-}" '
  {
    name:        $eid,
    entity_type: "kbd_lifecycle_event",
    observations: [({
      kind:        $kind,
      edge:        $edge,
      name:        $name,
      index:       $index,
      total:       $total,
      phase:       $phase,
      phasePath:   $phasePath,
      sourceTool:  $srcTool,
      project:     $project,
      ts:          $ts
    } | tojson)]
  }
' 2>/dev/null || true)"

if [[ -z "$payload" ]]; then
  printf 'kbd-memory-log: could not encode mirror payload; lifecycle continues\n' >&2
  exit 0
fi

if ! curl --noproxy '127.0.0.1,localhost,::1' -fs \
  --connect-timeout 1 --max-time 3 -X POST \
  -H 'content-type: application/json' \
  -d "$payload" \
  "$url/api/v1/entities" >/dev/null 2>&1; then
  printf 'kbd-memory-log: mirror write failed; lifecycle continues\n' >&2
fi

exit 0
