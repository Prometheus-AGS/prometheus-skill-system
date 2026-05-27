#!/usr/bin/env bash
# skills/kbd-memory-recall/kbd-memory-recall.sh — recall prior KBD events.

set -u
KBD_ORCHESTRATOR_ROOT="${KBD_ORCHESTRATOR_ROOT:-$HOME/.claude/skills/kbd-process-orchestrator}"
# shellcheck source=/dev/null
. "$KBD_ORCHESTRATOR_ROOT/shared/lib/memory.sh"

phase="${1:-}"
if [[ -z "$phase" && -f .kbd-orchestrator/current-waypoint.json ]] && command -v jq >/dev/null 2>&1; then
  phase="$(jq -r '.phase // ""' .kbd-orchestrator/current-waypoint.json 2>/dev/null || true)"
fi
if [[ -z "$phase" ]]; then
  printf 'kbd-memory-recall: no phase resolved (arg empty + no waypoint)\n' >&2
  exit 0
fi

phase_dir=".kbd-orchestrator/phases/$phase"
mkdir -p "$phase_dir"
digest="$phase_dir/prior-context.md"

# Memory unreachable → stub and bail.
if ! kbd_memory_available; then
  printf '%s\n' "<!-- memory endpoint unreachable; no prior context retrieved -->" > "$digest.tmp"
  mv -f "$digest.tmp" "$digest"
  printf 'Completed kbd-memory-recall — %s (stub; memory unreachable)\n' "$phase" >&2
  exit 0
fi

url="$(kbd_memory_url)"
if [[ -z "$url" ]]; then
  # MCP-only mode (tool available but no HTTP URL) — write a stub with note.
  printf '<!-- memory recall requires HTTP endpoint; MCP-only mode not yet wired here -->\n' > "$digest.tmp"
  mv -f "$digest.tmp" "$digest"
  exit 0
fi
command -v curl >/dev/null 2>&1 || { printf '<!-- curl missing; no recall performed -->\n' > "$digest"; exit 0; }
command -v jq   >/dev/null 2>&1 || { printf '<!-- jq missing; no recall performed -->\n'   > "$digest"; exit 0; }

# Build query text from goals + assessment.
query=""
[[ -f "$phase_dir/goals.md"      ]] && query+="$(cat "$phase_dir/goals.md")"
[[ -f "$phase_dir/assessment.md" ]] && query+=$'\n\n'"$(cat "$phase_dir/assessment.md")"
[[ -n "$query" ]] || query="$phase"

req="$(jq -c -n \
  --arg q "$query" --arg etype "kbd_lifecycle_event" '
  {query: $q, entityType: $etype, topN: 5}
')"

resp="$(curl -fsS -X POST --max-time 10 \
  -H 'content-type: application/json' \
  -d "$req" "$url/api/find_relevant" 2>/dev/null || true)"

{
  printf '# Prior context — %s\n\n' "$phase"
  printf '> Auto-populated by /kbd-memory-recall. Replace or extend if needed.\n\n'
  printf '## Most relevant prior phases (top 5)\n\n'
  if [[ -n "$resp" ]] && echo "$resp" | jq -e 'type == "array" and length > 0' >/dev/null 2>&1; then
    echo "$resp" | jq -r '
      to_entries[] |
      "\(.key + 1). **\(.value.observations[0].project // "?")/\(.value.observations[0].phase // .value.observations[0].name // "?")** — \(.value.observations[0].kind // "?") @ \(.value.observations[0].ts // "?")"
    '
  else
    printf '*(no prior matches found)*\n'
  fi
  printf '\n## Patterns observed\n\n'
  if [[ -n "$resp" ]] && echo "$resp" | jq -e 'type == "array" and length > 0' >/dev/null 2>&1; then
    echo "$resp" | jq -r '
      [.[] | .observations[0].kind // "?"] | group_by(.) |
      map({kind: .[0], count: length}) | sort_by(-.count) |
      .[] | "- \(.count)× \(.kind) events recalled"
    '
  else
    printf '*(none — first phase of its kind, or memory empty)*\n'
  fi
} > "$digest.tmp"
mv -f "$digest.tmp" "$digest"

printf 'Completed kbd-memory-recall — %s wrote prior-context.md\n' "$phase" >&2
exit 0
