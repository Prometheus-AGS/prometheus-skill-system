#!/usr/bin/env bash
# Full local integration coverage for the KBD shell memory subsystem.

set -uo pipefail
cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
export KBD_ORCHESTRATOR_ROOT="$SKILL_ROOT"

pass_count=0
pass() {
  pass_count=$((pass_count + 1))
  printf 'pass: %s\n' "$*"
}
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
assert_contains() {
  grep -Fq -- "$2" "$1" || fail "$3 (missing: $2)"
}
assert_not_contains() {
  if grep -Fq -- "$2" "$1"; then fail "$3 (unexpected: $2)"; fi
}

command -v jq >/dev/null 2>&1 || fail 'jq is required'
command -v curl >/dev/null 2>&1 || fail 'curl is required'
command -v python3 >/dev/null 2>&1 || fail 'python3 is required'

fixture="$(mktemp -d)"
server_pid=""
cleanup() {
  if [[ -n "$server_pid" ]]; then
    kill "$server_pid" >/dev/null 2>&1 || true
    wait "$server_pid" >/dev/null 2>&1 || true
  fi
  rm -rf "$fixture"
}
trap cleanup EXIT INT TERM

printf 'empty\n' > "$fixture/mode"
: > "$fixture/requests.jsonl"
printf '[]\n' > "$fixture/response.json"

python3 - "$fixture" > "$fixture/server.log" 2>&1 <<'PY' &
import json
import pathlib
import sys
import urllib.parse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

root = pathlib.Path(sys.argv[1])

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *_args):
        return

    def record(self, body=""):
        with (root / "requests.jsonl").open("a", encoding="utf-8") as handle:
            handle.write(json.dumps({
                "method": self.command,
                "path": self.path,
                "body": body,
            }, separators=(",", ":")) + "\n")

    def send_json(self, status, value):
        body = json.dumps(value, separators=(",", ":")).encode()
        self.send_response(status)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        self.record()
        mode = (root / "mode").read_text(encoding="utf-8").strip()
        parsed = urllib.parse.urlparse(self.path)
        if parsed.path == "/health":
            if mode == "health_fail":
                self.send_json(503, {"status": "starting"})
            else:
                self.send_json(200, {"status": "ok"})
            return
        if parsed.path == "/api/v1/entities/search":
            query = urllib.parse.parse_qs(parsed.query).get("q", [""])[0]
            if query != "kbd_lifecycle_event":
                self.send_json(400, {"error": "unexpected query"})
            elif mode == "http_fail":
                self.send_json(404, {"error": "route unavailable"})
            elif mode == "invalid":
                body = b"not-json"
                self.send_response(200)
                self.send_header("content-length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
            elif mode == "match":
                self.send_json(200, json.loads((root / "response.json").read_text()))
            else:
                self.send_json(200, [])
            return
        self.send_json(404, {"error": "not found"})

    def do_POST(self):
        length = int(self.headers.get("content-length", "0"))
        body = self.rfile.read(length).decode("utf-8")
        self.record(body)
        mode = (root / "mode").read_text(encoding="utf-8").strip()
        if self.path != "/api/v1/entities":
            self.send_json(404, {"error": "not found"})
        elif mode == "post_fail":
            self.send_json(500, {"error": "injected write failure"})
        else:
            self.send_json(201, json.loads(body))

server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
(root / "port").write_text(str(server.server_address[1]), encoding="utf-8")
server.serve_forever()
PY
server_pid=$!

ready=0
attempt=0
while [[ "$attempt" -lt 100 ]]; do
  if [[ -s "$fixture/port" ]]; then ready=1; break; fi
  attempt=$((attempt + 1))
  sleep 0.05
done
[[ "$ready" -eq 1 ]] || fail 'fake memory service did not start'
base="http://127.0.0.1:$(cat "$fixture/port")"

new_project() {
  local name="$1"
  local project_id="${2:-project-1}"
  local root="$fixture/$name"
  mkdir -p "$root/.kbd-orchestrator/phases"
  jq -n --arg phase "$name" --arg project "$project_id" \
    '{phase: $phase, projectId: $project}' \
    > "$root/.kbd-orchestrator/current-waypoint.json"
  printf '%s' "$root"
}

# Scenario 1: explicit MCP transport normalization plus the current lifecycle
# entity contract across a real local HTTP boundary.
project="$(new_project explicit-write project-1)"
: > "$fixture/requests.jsonl"
printf 'empty\n' > "$fixture/mode"
(
  cd "$project" || exit 1
  export UAR_MEMORY_MCP_URL="$base/mcp/sse"
  unset KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
  export KBD_HOOK_KIND=task KBD_HOOK_EDGE=after KBD_HOOK_NAME='contract repair'
  export KBD_HOOK_INDEX=3 KBD_HOOK_TOTAL=6 KBD_HOOK_PHASE_PATH=explicit-write
  export KBD_HOOK_SOURCE_TOOL=integration KBD_HOOK_STARTED_AT=2026-08-29T20:00:00Z
  /bin/bash "$SKILL_ROOT/shared/lib/memory-log.sh"
) 2> "$fixture/write.err" || fail 'explicit configured lifecycle write exited non-zero'
jq -e -s '
  map(select(.method == "POST" and .path == "/api/v1/entities"))
  | length == 1
  and (.[0].body | fromjson | .entity_type == "kbd_lifecycle_event")
  and (.[0].body | fromjson | .observations | length == 1)
  and (.[0].body | fromjson | .observations[0] | type == "string")
  and (.[0].body | fromjson | .observations[0] | fromjson
       | .project == "project-1" and .kind == "task" and .index == 3 and .total == 6)
' "$fixture/requests.jsonl" >/dev/null || fail 'captured lifecycle entity did not match the installed contract'
[[ ! -s "$fixture/write.err" ]] || fail 'successful lifecycle write emitted a diagnostic'
pass 'explicit MCP URL normalization and lifecycle entity write'

# Scenario 1b: MCP-only availability deliberately has no REST URL. Hook logging
# exits silently without issuing a malformed curl request.
project="$(new_project mcp-only project-1)"
: > "$fixture/requests.jsonl"
(
  cd "$project" || exit 1
  export UAR_MEMORY_MCP_URL='mcp://tool-only'
  unset KBD_MEMORY_MCP_URL
  export KBD_AVAILABLE_TOOLS='create_entity'
  export KBD_HOOK_KIND=task KBD_HOOK_EDGE=after KBD_HOOK_NAME=mcp-only
  export KBD_HOOK_INDEX=1 KBD_HOOK_TOTAL=1 KBD_HOOK_PHASE_PATH=mcp-only
  export KBD_HOOK_SOURCE_TOOL=integration KBD_HOOK_STARTED_AT=2026-08-29T20:01:00Z
  /bin/bash "$SKILL_ROOT/shared/lib/memory-log.sh"
) 2> "$fixture/mcp-only.err" || fail 'MCP-only memory logging exited non-zero'
[[ ! -s "$fixture/requests.jsonl" ]] || fail 'MCP-only memory logging issued an HTTP request'
[[ ! -s "$fixture/mcp-only.err" ]] || fail 'MCP-only memory logging emitted a diagnostic'
pass 'MCP-only availability skips REST mirror cleanly'

# Scenario 2: positive discovery remains cached after a real health exchange.
: > "$fixture/requests.jsonl"
printf 'empty\n' > "$fixture/mode"
(
  cd "$project" || exit 1
  export UAR_MEMORY_MCP_URL="$base/mcp/http"
  unset KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
  # shellcheck source=/dev/null
  . "$SKILL_ROOT/shared/lib/memory.sh"
  kbd_memory_available || exit 1
  printf 'health_fail\n' > "$fixture/mode"
  kbd_memory_available || exit 1
  [[ "$(kbd_memory_url)" == "$base" ]] || exit 1
) || fail 'positive availability cache did not survive the injected health failure'
health_count="$(jq -s '[.[] | select(.method == "GET" and .path == "/health")] | length' "$fixture/requests.jsonl")"
[[ "$health_count" == 1 ]] || fail "cached detection issued $health_count health probes instead of 1"
printf 'empty\n' > "$fixture/mode"
pass 'process-lifetime availability cache after real health probe'

# Scenario 3: current strings and a legacy object are flattened and ranked by
# same project, token overlap, and recency, with stable tie-breakers.
project="$(new_project ranking project-1)"
mkdir -p "$project/.kbd-orchestrator/phases/ranking"
printf '# Goals\nrepair endpoint contract entity search ranking\n' \
  > "$project/.kbd-orchestrator/phases/ranking/goals.md"
printf '# Assessment\nverify deterministic memory recall\n' \
  > "$project/.kbd-orchestrator/phases/ranking/assessment.md"
jq -n '
  def entity($name; $project; $phase; $label; $ts; $legacy):
    ({project: $project, phase: $phase, name: $label, kind: "task", ts: $ts}) as $event
    | {
        name: $name,
        entity_type: "kbd_lifecycle_event",
        observations: (if $legacy then [$event] else [($event | tojson)] end),
        created_at: $ts,
        updated_at: $ts
      };
  [
    entity("d-other"; "other-project"; "exact-other"; "repair endpoint contract entity search ranking"; "2026-08-29T23:00:00Z"; false),
    entity("c-unrelated"; "project-1"; "unrelated"; "banana"; "2026-08-29T22:00:00Z"; false),
    entity("b-exact-old"; "project-1"; "exact-old"; "repair endpoint contract entity search ranking"; "2026-08-28T20:00:00Z"; true),
    entity("a-exact-new"; "project-1"; "exact-new"; "repair endpoint contract entity search ranking"; "2026-08-29T20:00:00Z"; false)
  ]
' > "$fixture/response.json"
: > "$fixture/requests.jsonl"
printf 'match\n' > "$fixture/mode"
(
  cd "$project" || exit 1
  export UAR_MEMORY_MCP_URL="$base"
  unset KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
  /bin/bash "$SKILL_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh" ranking
) 2> "$fixture/ranking.err" || fail 'matching recall exited non-zero'
digest="$project/.kbd-orchestrator/phases/ranking/prior-context.md"
[[ -f "$digest" ]] || fail 'matching recall did not create a digest'
line_new="$(grep -nF '**project-1/exact-new**' "$digest" | cut -d: -f1)"
line_old="$(grep -nF '**project-1/exact-old**' "$digest" | cut -d: -f1)"
line_unrelated="$(grep -nF '**project-1/unrelated**' "$digest" | cut -d: -f1)"
line_other="$(grep -nF '**other-project/exact-other**' "$digest" | cut -d: -f1)"
[[ -n "$line_new" && -n "$line_old" && -n "$line_unrelated" && -n "$line_other" ]] \
  || fail 'matching recall omitted an expected candidate'
[[ "$line_new" -lt "$line_old" && "$line_old" -lt "$line_unrelated" && "$line_unrelated" -lt "$line_other" ]] \
  || fail 'matching recall order violated project/token/recency precedence'
jq -e -s 'any(.[]; .method == "GET" and (.path | startswith("/api/v1/entities/search?q=kbd_lifecycle_event")))' \
  "$fixture/requests.jsonl" >/dev/null || fail 'recall did not use canonical entity search'
pass 'reachable recall ranking across current and legacy observations'

# Scenario 4: restEndpoint discovery reaches search; a reachable empty array is
# never mislabeled as an unavailable service.
project="$(new_project reachable-empty project-1)"
jq -n --arg endpoint "$base/mcp/sse" '{restEndpoint: $endpoint}' \
  > "$project/.kbd-orchestrator/memory.config.json"
printf 'empty\n' > "$fixture/mode"
(
  cd "$project" || exit 1
  unset UAR_MEMORY_MCP_URL KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
  /bin/bash "$SKILL_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh" reachable-empty
) 2> "$fixture/empty.err" || fail 'reachable-empty recall exited non-zero'
digest="$project/.kbd-orchestrator/phases/reachable-empty/prior-context.md"
assert_contains "$digest" '*(no prior matches found)*' 'reachable-empty digest lost its explicit marker'
assert_not_contains "$digest" 'unreachable' 'reachable-empty digest was mislabeled unavailable'
pass 'project restEndpoint discovery and reachable-empty digest'

# Scenario 5: legacy mcpEndpoint discovery remains supported and normalized.
project="$(new_project legacy-config project-1)"
jq -n --arg endpoint "$base/mcp/http" '{mcpEndpoint: $endpoint}' \
  > "$project/.kbd-orchestrator/memory.config.json"
(
  cd "$project" || exit 1
  unset UAR_MEMORY_MCP_URL KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
  /bin/bash "$SKILL_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh" legacy-config
) 2> "$fixture/legacy.err" || fail 'legacy mcpEndpoint recall exited non-zero'
digest="$project/.kbd-orchestrator/phases/legacy-config/prior-context.md"
assert_contains "$digest" '*(no prior matches found)*' 'legacy mcpEndpoint did not reach entity search'
pass 'legacy mcpEndpoint normalization through production recall'

# Scenario 6: the installed canonical local service is an explicit live-system
# integration probe. The default suite remains hermetic and exercises the same
# production recall path against the local HTTP fixture in scenarios 1-5.
if [[ "${KBD_MEMORY_LIVE_PROBE:-0}" == "1" ]]; then
  project="$(new_project canonical-default project-1)"
  (
    cd "$project" || exit 1
    unset UAR_MEMORY_MCP_URL KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
    /bin/bash "$SKILL_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh" canonical-default
  ) 2> "$fixture/default.err" || fail 'canonical-default recall exited non-zero'
  digest="$project/.kbd-orchestrator/phases/canonical-default/prior-context.md"
  [[ -f "$digest" ]] || fail 'canonical-default recall did not create a digest'
  assert_not_contains "$digest" 'memory endpoint unreachable' 'canonical local service was not discovered'
  assert_not_contains "$digest" 'invalid; no prior context' 'canonical local service returned an invalid contract'
  pass 'canonical local service discovery through production recall'
else
  printf 'skip: canonical local service discovery (set KBD_MEMORY_LIVE_PROBE=1)\n'
fi

# Scenario 7: explicit unreachability wins over the default and creates an
# atomic stub without a non-zero lifecycle result.
project="$(new_project unreachable project-1)"
(
  cd "$project" || exit 1
  export UAR_MEMORY_MCP_URL='http://127.0.0.1:1/mcp/sse'
  unset KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
  /bin/bash "$SKILL_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh" unreachable
) 2> "$fixture/unreachable.err" || fail 'unreachable recall blocked orchestration'
digest="$project/.kbd-orchestrator/phases/unreachable/prior-context.md"
assert_contains "$digest" 'memory endpoint unreachable' 'unreachable recall did not write its stub'
[[ ! -e "$digest.tmp" ]] || fail 'unreachable recall left a temporary digest behind'
pass 'explicit unreachable service fail-open stub'

# Scenario 8: a healthy service whose write fails emits exactly one fixed
# diagnostic and never blocks the hook lifecycle.
project="$(new_project write-failure project-1)"
printf 'post_fail\n' > "$fixture/mode"
(
  cd "$project" || exit 1
  export UAR_MEMORY_MCP_URL="$base"
  unset KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
  export KBD_HOOK_KIND=phase KBD_HOOK_EDGE=before KBD_HOOK_NAME=write-failure
  export KBD_HOOK_INDEX=1 KBD_HOOK_TOTAL=1 KBD_HOOK_PHASE_PATH=write-failure
  export KBD_HOOK_SOURCE_TOOL=integration KBD_HOOK_STARTED_AT=2026-08-29T20:05:00Z
  /bin/bash "$SKILL_ROOT/shared/lib/memory-log.sh"
) 2> "$fixture/write-failure.err" || fail 'HTTP 500 memory write blocked lifecycle'
line_count="$(wc -l < "$fixture/write-failure.err" | tr -d ' ')"
[[ "$line_count" == 1 ]] || fail "write failure emitted $line_count diagnostics instead of 1"
assert_contains "$fixture/write-failure.err" 'kbd-memory-log: mirror write failed; lifecycle continues' \
  'write failure diagnostic changed'
pass 'bounded fail-open lifecycle write diagnostic'

# Scenario 9: an invalid response is distinct from transport failure and is
# committed atomically as a diagnostic stub.
project="$(new_project invalid-response project-1)"
printf 'invalid\n' > "$fixture/mode"
(
  cd "$project" || exit 1
  export UAR_MEMORY_MCP_URL="$base"
  unset KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
  /bin/bash "$SKILL_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh" invalid-response
) 2> "$fixture/invalid.err" || fail 'invalid response blocked orchestration'
digest="$project/.kbd-orchestrator/phases/invalid-response/prior-context.md"
assert_contains "$digest" 'invalid; no prior context retrieved' 'invalid response did not produce its distinct stub'
assert_not_contains "$digest" 'memory endpoint unreachable' 'invalid response was mislabeled unreachable'
[[ ! -e "$digest.tmp" ]] || fail 'invalid response left a temporary digest behind'
pass 'invalid entity-search contract fail-open stub'

# Scenario 10: a reachable service returning an HTTP route error is distinct
# from both transport unreachability and an invalid successful response.
project="$(new_project http-error project-1)"
printf 'http_fail\n' > "$fixture/mode"
(
  cd "$project" || exit 1
  export UAR_MEMORY_MCP_URL="$base"
  unset KBD_MEMORY_MCP_URL KBD_AVAILABLE_TOOLS
  /bin/bash "$SKILL_ROOT/skills/kbd-memory-recall/kbd-memory-recall.sh" http-error
) 2> "$fixture/http-error.err" || fail 'HTTP route error blocked orchestration'
digest="$project/.kbd-orchestrator/phases/http-error/prior-context.md"
assert_contains "$digest" 'memory entity-search HTTP error 404' 'HTTP route failure lacked its distinct stub'
assert_not_contains "$digest" 'memory endpoint unreachable' 'HTTP route failure was mislabeled unreachable'
if find "$project/.kbd-orchestrator/phases/http-error" \
  -name '.prior-context-search.*' -print | grep -q .; then
  fail 'HTTP route failure left a temporary search response behind'
fi
pass 'reachable entity-search HTTP failure diagnostic stub'

expected_count=10
[[ "${KBD_MEMORY_LIVE_PROBE:-0}" == "1" ]] && expected_count=11
printf '\nall enabled KBD memory full-integration scenarios passed (%s/%s)\n' \
  "$pass_count" "$expected_count"
