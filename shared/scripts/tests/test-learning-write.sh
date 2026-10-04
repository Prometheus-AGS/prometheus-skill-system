#!/bin/bash
# test-learning-write.sh — integration test for the attributed write path (B3).
#
# Real processes end to end: learning_write.py and the SubagentStop hook write
# envelope-bearing operations to a scratch learning-queue outbox; a real
# prometheus-learning-worker delivers them to a real surreal-memory-server
# (embedded mode, scratch database, local MLX embeddings) and the records are
# read back over REST. Nothing touches the user's real HOME, queue or memory
# service.
#
# Binaries: TLI_SM_BIN / TLI_WORKER_BIN override the surreal-memory-server and
# prometheus-learning-worker on PATH (both must be >= 1.10.0).
# Exit 0 pass, 1 fail, 2 BLOCKED (a prerequisite is missing). bash 3.2.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
LW="$ROOT/shared/scripts/lib/learning_write.py"
GUARD="$ROOT/shared/scripts/team-role-guard.sh"
SM_BIN="${TLI_SM_BIN:-$(command -v surreal-memory-server || true)}"
WORKER_BIN="${TLI_WORKER_BIN:-$(command -v prometheus-learning-worker || true)}"
SCHEMA="$ROOT/shared/schemas/learning-envelope.schema.json"
PORT="${TLI_SM_PORT:-23021}"

blocked() { echo "BLOCKED: $*" >&2; exit 2; }
[ -x "$SM_BIN" ] || blocked "surreal-memory-server not found (set TLI_SM_BIN)"
[ -x "$WORKER_BIN" ] || blocked "prometheus-learning-worker not found (set TLI_WORKER_BIN)"
"$SM_BIN" --version 2>/dev/null | grep -Eq '1\.(1[0-9]|[2-9][0-9])\.' || blocked "surreal-memory-server < 1.10.0"
"$WORKER_BIN" --version 2>/dev/null | grep -Eq '1\.(1[0-9]|[2-9][0-9])\.' || blocked "prometheus-learning-worker < 1.10.0"
python3 -c 'import jsonschema' 2>/dev/null || blocked "python jsonschema missing"
[ -x /usr/local/bin/surreal-memory-mlx-executor ] || blocked "MLX embedding executor missing"

S="$(mktemp -d)"
srv=""
cleanup() { [ -n "$srv" ] && kill "$srv" 2>/dev/null; rm -rf "$S"; }
trap cleanup EXIT
export HOME="$S/home" PROMETHEUS_LEARNING_QUEUE="$S/queue" PROMETHEUS_LEARNING_LOG_DIR="$S/log" PROMETHEUS_LEARNING_INDEX_DIR="$S/index"
export PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=1
mkdir -p "$HOME"
git config --global user.email fixture@example.invalid; git config --global user.name Fixture

REPO="$S/repo"; mkdir -p "$REPO/.agent-team/tlm-fixture" "$REPO/.prometheus"
git -C "$REPO" init -q
printf '{"projectId":"project:b3-fixture"}\n' > "$REPO/.prometheus/project.json"
cat > "$REPO/.agent-team/tlm-fixture/team.json" <<'EOF'
{"schemaVersion":1,"id":"tlm-fixture","roles":[
 {"id":"api-dev","description":"api","prompt":"p","skills":[],"owns":["src/api/**"],"inputs":[],"outputs":[],"dependsOn":[]},
 {"id":"ui-dev","description":"ui","prompt":"p","skills":[],"owns":["src/ui/**"],"inputs":[],"outputs":[],"dependsOn":[]}]}
EOF
PAYLOAD='{"agent_type":"prometheus-skill-pack:api-dev","agent_id":"a-api","session_id":"s-b3","cwd":"'"$REPO"'"}'

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }
write() { printf '%s' "$PAYLOAD" | python3 "$LW" --payload-stdin --cwd "$REPO" "$@"; }
ops() { cat "$PROMETHEUS_LEARNING_QUEUE"/memory/*/*.json 2>/dev/null; }

# --- one write per visibility level -> design-table scope keys --------------
check_scope() { # <visibility> <expected user_id> <expected agent_id>
  out="$(write --text "scope lesson for $1" --visibility "$1")"
  python3 - "$out" "$2" "$3" "$PROMETHEUS_LEARNING_QUEUE" "$SCHEMA" <<'PY' || fail "scope $1"
import json, sys, glob, jsonschema
out, uid, aid, queue, schema = sys.argv[1:]
result = json.loads(out)
assert result["written"] == 1, result
op_id = result["operations"][0]["operation"]
op = json.load(open(glob.glob(f"{queue}/memory/*/{op_id}.json")[0]))
args = op["arguments"]
assert (args["user_id"], args["agent_id"]) == (uid, aid), (args["user_id"], args["agent_id"], uid, aid)
trailer = args["content"].rsplit("<!-- learning-envelope: ", 1)[1].rsplit(" -->", 1)[0]
envelope = json.loads(trailer)
jsonschema.Draft202012Validator(json.load(open(schema))).validate(envelope)
assert "env:1" in args["categories"] and f"h:{envelope['contentHash'][:16]}" in args["categories"], args["categories"]
PY
}
USER_SCOPE="$(cd "$REPO" && python3 "$ROOT/shared/scripts/lib/project_id.py" --json | python3 -c 'import json,sys;print(json.load(sys.stdin)["userScope"])')"
check_scope agent        project:b3-fixture tlm-fixture/api-dev
check_scope role:ui-dev  project:b3-fixture tlm-fixture/ui-dev
check_scope lead         project:b3-fixture tlm-fixture/@lead
check_scope team         project:b3-fixture tlm-fixture/@team
check_scope project      project:b3-fixture @project
check_scope user         "@$USER_SCOPE"     @user
check_scope global       @global            @global
ok "every visibility level is stored under its design-table agent_id, and every envelope validates"

# --- dedupe and audience copies ---------------------------------------------
before="$(ls "$PROMETHEUS_LEARNING_QUEUE"/memory/pending | wc -l)"
write --text "scope lesson for agent" --visibility agent >/dev/null
after="$(ls "$PROMETHEUS_LEARNING_QUEUE"/memory/pending | wc -l)"
[ "$before" -eq "$after" ] || fail "repeated contentHash queued a second operation"
ok "a repeated lesson (same contentHash, same scope) is queued once"

out="$(write --text "routed lesson about src/ui" --visibility agent --audience role:ui-dev lead)"
python3 -c 'import json,sys;r=json.loads(sys.argv[1]);s=sorted(o["agent_id"] for o in r["operations"]);assert r["written"]==3 and s==["tlm-fixture/@lead","tlm-fixture/api-dev","tlm-fixture/ui-dev"],r' "$out" || fail "audience copies"
ok "audience entries produce one addressed copy each"

# --- SubagentStop hook through the team-role guard ---------------------------
HOOKPAYLOAD='{"hook_event_name":"SubagentStop","agent_type":"api-dev","agent_id":"a-api","session_id":"s-b3","cwd":"'"$REPO"'","last_assistant_message":"Done.\nLESSON: anchor SubagentStop matchers to the plugin prefix\n[GLOBAL] Codex agent names allow only [a-z0-9_]"}'
stdout="$(cd "$REPO" && printf '%s' "$HOOKPAYLOAD" | bash "$GUARD" --only-team-roles shared/scripts/subagentstop-learning.sh)"
[ -z "$stdout" ] || fail "hook printed to stdout: $stdout"
ops | python3 -c '
import json,sys
docs=[json.loads(l) for l in sys.stdin if l.strip()]
keys={(d["arguments"]["agent_id"], d["arguments"]["content"].split("\n")[0]) for d in docs}
assert ("tlm-fixture/api-dev","anchor SubagentStop matchers to the plugin prefix") in keys, keys
assert ("@global","Codex agent names allow only [a-z0-9_]") in keys, keys
' || fail "hook lessons not attributed"
python3 - "$PROMETHEUS_LEARNING_QUEUE" <<'PY' || fail "learning job identity"
import json, glob, sys
jobs = [json.load(open(p)) for p in glob.glob(f"{sys.argv[1]}/pending/*.json")]
assert any(j.get("teamId") == "tlm-fixture" and j.get("roleId") == "api-dev" and j.get("projectId") == "project:b3-fixture" for j in jobs), jobs
PY
ok "SubagentStop for a team role writes attributed + promoted lessons and queues an identity-bearing learning job, silently"

count_before="$(ops | wc -l)"
NONTEAM='{"hook_event_name":"SubagentStop","agent_type":"general-purpose","cwd":"'"$REPO"'","last_assistant_message":"LESSON: should not be written"}'
(cd "$REPO" && printf '%s' "$NONTEAM" | bash "$GUARD" --only-team-roles shared/scripts/subagentstop-learning.sh)
[ "$(ops | wc -l)" -eq "$count_before" ] || fail "non-team agent wrote lessons"
ok "a non-team agent writes nothing"

# --- delivery: real worker -> real 1.10 server -> read back -----------------
mkdir -p "$S/sm"
( cd "$S/sm" && env SURREAL_MODE=embedded SURREAL_PATH="$S/sm/db" SURREAL_NAMESPACE=b3 SURREAL_DATABASE=gate \
    API_HOST=127.0.0.1 API_PORT="$PORT" MCP_STDIO=false EMBEDDING_PROVIDER=local LOCAL_EMBEDDING_BACKEND=mlx \
    LOCAL_EMBEDDING_EXECUTOR=/usr/local/bin/surreal-memory-mlx-executor LOCAL_EMBEDDING_MODEL=BAAI/bge-small-en-v1.5 \
    LOCAL_EMBEDDING_MODEL_REVISION=5c38ec7c405ec4b44b94cc5a9bb96e735b38267a LOCAL_EMBEDDING_DIMENSIONS=384 \
    HF_HUB_CACHE="/Users/$(id -un)/.cache/huggingface/hub" MODEL_CACHE_DIR="/Users/$(id -un)/.cache/huggingface" \
    SURREAL_EXECUTOR_STARTUP_MS=300000 RUST_LOG=warn "$SM_BIN" > "$S/sm/server.log" 2>&1 ) &
srv=$!
for _ in $(seq 1 180); do curl -fsS -m 1 "http://127.0.0.1:$PORT/ready" 2>/dev/null | grep -q '"ledger":true' && break; sleep 1; done
curl -fsS -m 1 "http://127.0.0.1:$PORT/ready" 2>/dev/null | grep -q '"ledger":true' || { tail -20 "$S/sm/server.log" >&2; blocked "scratch surreal-memory never became ready"; }
for _ in 1 2 3 4 5 6; do
  "$WORKER_BIN" --memory-url "http://127.0.0.1:$PORT" run-once >/dev/null 2>&1
  [ -z "$(ls "$PROMETHEUS_LEARNING_QUEUE"/memory/pending "$PROMETHEUS_LEARNING_QUEUE"/memory/submitting "$PROMETHEUS_LEARNING_QUEUE"/memory/accepted 2>/dev/null | grep json)" ] && break
  sleep 3
done
got="$(curl -fsS -m 10 "http://127.0.0.1:$PORT/api/v1/memory/?user_id=project:b3-fixture&agent_id=tlm-fixture/api-dev")"
python3 - "$got" "$SCHEMA" <<'PY' || fail "delivered record not found or invalid"
import json, sys, jsonschema
records = json.loads(sys.argv[1])
records = records if isinstance(records, list) else records.get("memories", records.get("results", []))
hits = [r for r in records if "anchor SubagentStop matchers" in r.get("content", "")]
assert hits, [r.get("content", "")[:60] for r in records]
envelope = json.loads(hits[0]["content"].rsplit("<!-- learning-envelope: ", 1)[1].rsplit(" -->", 1)[0])
jsonschema.Draft202012Validator(json.load(open(sys.argv[2]))).validate(envelope)
assert envelope["teamId"] == "tlm-fixture" and envelope["roleId"] == "api-dev", envelope
PY
ok "the worker delivers attributed lessons to surreal-memory and they read back under <team>/<role> with a valid envelope"

echo "test-learning-write: $pass passed"
