#!/usr/bin/env bash
# shared/lib/memory-log.sh — wrapper invoked by the kbd-memory-log hook.
#
# Mirrors each hook fire into surreal-memory as a structured entity. No-ops
# silently when the memory endpoint is unreachable.

set -euo pipefail

KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}"
# shellcheck source=/dev/null
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/memory.sh"

kbd_memory_available || exit 0
url="$(kbd_memory_url)"
[[ -n "$url" ]] || exit 0
command -v jq >/dev/null 2>&1 || exit 0
command -v curl >/dev/null 2>&1 || exit 0

project="unknown"
if [[ -f .kbd-orchestrator/project.json ]]; then
  project="$(jq -r '.project // "unknown"' .kbd-orchestrator/project.json 2>/dev/null || echo unknown)"
fi

# Extract first segment of phase path for the relation target.
phase="${KBD_HOOK_PHASE_PATH%% *}"
[[ -n "$phase" ]] || phase="unknown"

entity_id="$project/$phase/${KBD_HOOK_KIND:-?}/${KBD_HOOK_EDGE:-?}/${KBD_HOOK_INDEX:-1}/${KBD_HOOK_STARTED_AT:-}"

payload="$(jq -c -n \
  --arg eid       "$entity_id" \
  --arg project   "$project" \
  --arg phase     "$phase" \
  --arg kind      "${KBD_HOOK_KIND:-}" \
  --arg edge      "${KBD_HOOK_EDGE:-}" \
  --arg name      "${KBD_HOOK_NAME:-}" \
  --argjson index "${KBD_HOOK_INDEX:-1}" \
  --argjson total "${KBD_HOOK_TOTAL:-1}" \
  --arg phasePath "${KBD_HOOK_PHASE_PATH:-}" \
  --arg srcTool   "${KBD_HOOK_SOURCE_TOOL:-unknown}" \
  --arg ts        "${KBD_HOOK_STARTED_AT:-}" '
  {
    entityType: "kbd_lifecycle_event",
    entityId:   $eid,
    observations: [{
      kind:       $kind,
      edge:       $edge,
      name:       $name,
      index:      $index,
      total:      $total,
      phasePath:  $phasePath,
      sourceTool: $srcTool,
      project:    $project,
      ts:         $ts
    }],
    relations: [
      { from: $eid, to: ("phase:"   + $phase),   label: "fires-in" },
      { from: $eid, to: ("project:" + $project), label: "belongs-to" }
    ]
  }
')"

curl -fsS -X POST --max-time 3 \
  -H 'content-type: application/json' \
  -d "$payload" \
  "$url/api/entities" >/dev/null 2>&1 || true

exit 0
