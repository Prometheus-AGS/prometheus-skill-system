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

write_recall_stub() {
  printf '%s\n' "$1" > "$digest.tmp"
  mv -f "$digest.tmp" "$digest"
}

# Memory unreachable → stub and bail.
if ! kbd_memory_available; then
  write_recall_stub '<!-- memory endpoint unreachable; no prior context retrieved -->'
  printf 'Completed kbd-memory-recall — %s (stub; memory unreachable)\n' "$phase" >&2
  exit 0
fi

url="$(kbd_memory_url)"
if [[ -z "$url" ]]; then
  # MCP-only mode (tool available but no HTTP URL) — write a stub with note.
  write_recall_stub '<!-- memory recall requires HTTP endpoint; MCP-only mode not yet wired here -->'
  exit 0
fi
command -v curl >/dev/null 2>&1 || { write_recall_stub '<!-- curl missing; no recall performed -->'; exit 0; }
command -v jq   >/dev/null 2>&1 || { write_recall_stub '<!-- jq missing; no recall performed -->'; exit 0; }

project="unknown"
if [[ -f .kbd-orchestrator/project.json ]]; then
  project="$(jq -r '.project // .projectId // "unknown"' \
    .kbd-orchestrator/project.json 2>/dev/null || echo unknown)"
elif [[ -f .kbd-orchestrator/current-waypoint.json ]]; then
  project="$(jq -r '.project // .projectId // "unknown"' \
    .kbd-orchestrator/current-waypoint.json 2>/dev/null || echo unknown)"
fi

# Build query text from goals + assessment.
query=""
[[ -f "$phase_dir/goals.md"      ]] && query+="$(cat "$phase_dir/goals.md")"
[[ -f "$phase_dir/assessment.md" ]] && query+=$'\n\n'"$(cat "$phase_dir/assessment.md")"
[[ -n "$query" ]] || query="$phase"

# The canonical endpoint offers entity search rather than the removed semantic
# route. Retrieve the lifecycle-event class once, then rank locally so project
# affinity, token overlap, and recency have an explicit, stable order.
search_tmp="$(mktemp "$phase_dir/.prior-context-search.XXXXXX" 2>/dev/null || true)"
if [[ -z "$search_tmp" ]]; then
  write_recall_stub '<!-- memory recall could not allocate search workspace; no prior context retrieved -->'
  exit 0
fi
cleanup_search() { rm -f "$search_tmp"; }
trap cleanup_search EXIT INT TERM

if ! http_status="$(curl --noproxy '127.0.0.1,localhost,::1' -sS \
  --connect-timeout 1 --max-time 3 --get \
  --output "$search_tmp" --write-out '%{http_code}' \
  --data-urlencode 'q=kbd_lifecycle_event' \
  "$url/api/v1/entities/search" 2>/dev/null)"; then
  write_recall_stub '<!-- memory endpoint unreachable; no prior context retrieved -->'
  printf 'Completed kbd-memory-recall — %s (stub; entity search transport failed)\n' "$phase" >&2
  exit 0
fi

case "$http_status" in
  2??) ;;
  *)
    write_recall_stub "<!-- memory entity-search HTTP error $http_status; no prior context retrieved -->"
    printf 'Completed kbd-memory-recall — %s (stub; entity search HTTP %s)\n' \
      "$phase" "$http_status" >&2
    exit 0
    ;;
esac
resp="$(cat "$search_tmp")"
cleanup_search
trap - EXIT INT TERM

if ! ranked="$(printf '%s' "$resp" | jq -c \
  --arg project "$project" --arg query "$phase $query" '
  def tokens:
    tostring
    | ascii_downcase
    | [scan("[a-z0-9_][a-z0-9_-]*")]
    | map(select(length > 1))
    | unique;
  def recency_key:
    tostring | gsub("[^0-9]"; "") | .[0:14] | (tonumber? // 0);
  if type != "array" then error("entity search response is not an array") else . end
  | ($query | tokens) as $query_tokens
  | [
      .[] as $entity
      | select(($entity.entity_type // "") == "kbd_lifecycle_event")
      | ($entity.observations // [] | to_entries[]) as $observation
      | $observation.value as $raw
      | (if ($raw | type) == "string" then
           (try ($raw | fromjson) catch {text: $raw})
         elif ($raw | type) == "object" then
           $raw
         else
           {text: ($raw | tostring)}
         end) as $event
      | ([
           ($entity.name // ""),
           ($entity.entity_type // ""),
           ($event | tojson)
         ] | join(" ") | tokens) as $candidate_tokens
      | {
          entityName: ($entity.name // "?"),
          observationIndex: $observation.key,
          project: ($event.project // "?"),
          phase: ($event.phase // $event.name // "?"),
          kind: ($event.kind // "?"),
          ts: ($event.ts // $entity.updated_at // $entity.created_at // "?"),
          sameProject: (if ($event.project // "") == $project then 1 else 0 end),
          tokenOverlap: ([$candidate_tokens[] | select(. as $token | $query_tokens | index($token))] | length),
          recency: (($event.ts // $entity.updated_at // $entity.created_at // "") | recency_key)
        }
    ]
  | sort_by(-.sameProject, -.tokenOverlap, -.recency, .entityName, .observationIndex)
  | .[0:5]
')"; then
  write_recall_stub '<!-- memory entity-search response was invalid; no prior context retrieved -->'
  printf 'Completed kbd-memory-recall — %s (stub; invalid entity search response)\n' "$phase" >&2
  exit 0
fi

{
  printf '# Prior context — %s\n\n' "$phase"
  printf '> Auto-populated by /kbd-memory-recall. Replace or extend if needed.\n\n'
  printf '## Most relevant prior phases (top 5)\n\n'
  if printf '%s' "$ranked" | jq -e 'length > 0' >/dev/null 2>&1; then
    printf '%s' "$ranked" | jq -r '
      to_entries[] |
      "\(.key + 1). **\(.value.project)/\(.value.phase)** — \(.value.kind) @ \(.value.ts)"
    '
  else
    printf '*(no prior matches found)*\n'
  fi
  printf '\n## Patterns observed\n\n'
  if printf '%s' "$ranked" | jq -e 'length > 0' >/dev/null 2>&1; then
    printf '%s' "$ranked" | jq -r '
      [.[] | .kind] | group_by(.) |
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
