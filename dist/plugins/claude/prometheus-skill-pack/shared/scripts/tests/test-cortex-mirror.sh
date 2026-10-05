#!/bin/bash
# test-cortex-mirror.sh — integration test for the optional Cortex mirror (D3).
#
# learning_write.py is run as a real process. "Cortex present" uses a stub MCP
# stdio server (a real child process speaking JSON-RPC, started the way a real
# Cortex server is) selected through PROMETHEUS_CORTEX_MCP; the lesson is then
# recalled through the stub by project and by role tag. "Cortex absent" runs the
# same write with nothing discoverable and requires exit 0 and empty stderr.
# Scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; nothing real is touched.
#
# Case 7 ("real") runs the same mirror against the REAL installed Cortex MCP server
# (default 2.0.3; override with CORTEX_EXPECTED_VERSION) on a scratch CORTEX_DATA_DIR,
# then recalls the lesson through the real server's cortex_recall by projectId. The
# installed plugin is located under the real HOME *before* HOME is switched and handed
# over through PROMETHEUS_CORTEX_MCP. It needs the embedding model that ships inside the
# plugin (node_modules/@xenova/transformers/.cache); without it the case is BLOCKED
# (exit 2) - the test never downloads anything into the plugin cache. It also asserts
# that ~/.cortex/memory.db and ~/.claude/plugins/cache/cortex are untouched.
# CORTEX_REAL=skip runs the stub cases only (the real case is slow: model load).
# Exit 0 pass, 1 fail, 2 BLOCKED. bash 3.2.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
LW="$ROOT/shared/scripts/lib/learning_write.py"
blocked() { echo "BLOCKED: $*" >&2; exit 2; }
command -v python3 >/dev/null 2>&1 || blocked "python3 missing"
python3 -c 'import jsonschema' 2>/dev/null || blocked "python jsonschema missing"
[ -f "$LW" ] || blocked "learning_write.py missing"

REAL_HOME="$HOME"
S="$(mktemp -d)"
cleanup() { rm -rf "$S"; }
trap cleanup EXIT
export HOME="$S/home" CODEX_HOME="$S/codex" PROMETHEUS_PLUGIN_ROOT="$S/plugin"
export PROMETHEUS_LEARNING_QUEUE="$S/queue" PROMETHEUS_LEARNING_LOG_DIR="$S/log" PROMETHEUS_LEARNING_INDEX_DIR="$S/index"
export PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=1 PROMETHEUS_TEAM_DIGEST=0
unset PROMETHEUS_CORTEX_MCP PROMETHEUS_LEARNING_CORTEX
mkdir -p "$HOME" "$CODEX_HOME" "$PROMETHEUS_PLUGIN_ROOT"

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }

REPO="$S/repo"; mkdir -p "$REPO/.agent-team/cx-fixture" "$REPO/.prometheus"
( git -C "$REPO" init -q ) || exit $?
printf '{"projectId":"project:d3-fixture"}\n' > "$REPO/.prometheus/project.json"
cat > "$REPO/.agent-team/cx-fixture/team.json" <<'JSON'
{"schemaVersion":1,"id":"cx-fixture","roles":[
 {"id":"api-dev","description":"api","prompt":"p","skills":[],"owns":["src/api/**"],"inputs":[],"outputs":[],"dependsOn":[]}]}
JSON
PAYLOAD='{"agent_type":"prometheus-skill-pack:api-dev","agent_id":"a-api","session_id":"s-d3","cwd":"'"$REPO"'"}'

# Stub Cortex: newline-delimited JSON-RPC on stdio, store in $STORE.
STORE="$S/cortex-store.jsonl"; : > "$STORE"
cat > "$S/stub-cortex.py" <<'PY'
import json, os, sys
store = os.environ["STUB_STORE"]
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    req = json.loads(line)
    method, rid = req.get("method"), req.get("id")
    if method == "initialize":
        out = {"protocolVersion": "2024-11-05", "capabilities": {"tools": {}}, "serverInfo": {"name": "stub-cortex", "version": "0"}}
    elif method == "tools/call":
        name, args = req["params"]["name"], req["params"].get("arguments", {})
        if name == "cortex_remember":
            with open(store, "a") as handle:
                handle.write(json.dumps(args, sort_keys=True) + "\n")
            out = {"content": [{"type": "text", "text": json.dumps({"success": True})}]}
        elif name == "cortex_recall":
            hits = []
            with open(store) as handle:
                for row in handle:
                    rec = json.loads(row)
                    if args.get("projectId") not in (None, rec.get("projectId")):
                        continue
                    if args["query"] in rec.get("context", "") + rec.get("content", ""):
                        hits.append(rec)
            out = {"content": [{"type": "text", "text": json.dumps({"results": hits})}]}
        else:
            out = {}
    else:
        out = {}
    sys.stdout.write(json.dumps({"jsonrpc": "2.0", "id": rid, "result": out}) + "\n")
    sys.stdout.flush()
PY
export STUB_STORE="$STORE"

write() { printf '%s' "$PAYLOAD" | python3 "$LW" --payload-stdin --cwd "$REPO" "$@"; }
wait_for_lines() { # <expected count> — the mirror is detached, so poll
  n=0
  while [ "$n" -lt 100 ]; do
    [ "$(wc -l < "$STORE" | tr -d ' ')" -ge "$1" ] && return 0
    sleep 0.1; n=$((n + 1))
  done
  return 1
}
recall() { # <query> [projectId] — real round trip through the stub's recall tool
  python3 - "$S/stub-cortex.py" "$1" "${2:-}" <<'PY'
import json, subprocess, sys
stub, query, project = sys.argv[1:4]
args = {"query": query}
if project:
    args["projectId"] = project
reqs = [{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "cortex_recall", "arguments": args}}]
out = subprocess.run([sys.executable, stub], input="\n".join(json.dumps(r) for r in reqs) + "\n",
                     capture_output=True, text=True, timeout=10).stdout.strip().splitlines()[-1]
print(json.loads(json.loads(out)["result"]["content"][0]["text"])["results"].__len__())
PY
}

# --- 1. Cortex absent: nothing discoverable -> exit 0, no output on stderr ----
( write --text "Absent lesson: run the gate once at the phase boundary" >"$S/absent.out" 2>"$S/absent.err" ) || fail "write exited non-zero with Cortex absent"
[ ! -s "$S/absent.err" ] || fail "stderr not empty with Cortex absent: $(cat "$S/absent.err")"
python3 - "$S/absent.out" <<'PY' || fail "absent: summary wrong"
import json, sys
r = json.load(open(sys.argv[1]))
assert r["written"] >= 1 and r["cortex"] is False, r
PY
[ "$(wc -l < "$STORE" | tr -d ' ')" = "0" ] || fail "absent: store was written"
ok "Cortex absent: write exits 0, stderr empty, no mirror"

# --- 2. Cortex absent even with a broken explicit command --------------------
( PROMETHEUS_CORTEX_MCP="$S/does-not-exist-cortex" write --text "Broken command lesson" >"$S/broken.out" 2>"$S/broken.err" ) || fail "broken command: non-zero exit"
[ ! -s "$S/broken.err" ] || fail "broken command: stderr not empty"
ok "unusable Cortex command is silent"

# --- 3. Cortex present: one attributed write mirrors, recalled by project and role tag
export PROMETHEUS_CORTEX_MCP="python3 $S/stub-cortex.py"
( write --text "Present lesson: Cortex mirror carries the role tag" --kind gotcha >"$S/present.out" 2>"$S/present.err" ) || fail "write exited non-zero with Cortex present"
[ ! -s "$S/present.err" ] || fail "present: stderr not empty"
wait_for_lines 1 || fail "present: Cortex never received cortex_remember"
python3 - "$STORE" <<'PY' || fail "present: stored record wrong"
import json, sys
rows = [json.loads(l) for l in open(sys.argv[1])]
assert len(rows) == 1, rows
r = rows[0]
assert r["projectId"] == "project:d3-fixture", r
assert "team/role:cx-fixture/api-dev" in r["context"], r
assert "Present lesson" in r["content"] and "prometheus-envelope" not in r["content"], r
PY
[ "$(recall 'team/role:cx-fixture/api-dev' project:d3-fixture)" = "1" ] || fail "recall by project and role tag found no lesson"
[ "$(recall 'team/role:cx-fixture/ui-dev' project:d3-fixture)" = "0" ] || fail "recall by another role tag matched"
[ "$(recall 'team/role:cx-fixture/api-dev' project:other)" = "0" ] || fail "recall in another project matched"
ok "Cortex present: lesson mirrored and recalled by project + role tag"

# --- 4. duplicate write is not mirrored twice ---------------------------------
( write --text "Present lesson: Cortex mirror carries the role tag" --kind gotcha >"$S/dup.out" 2>/dev/null ) || fail "duplicate write failed"
sleep 1
[ "$(wc -l < "$STORE" | tr -d ' ')" = "1" ] || fail "duplicate lesson was mirrored again"
ok "duplicate lesson not re-mirrored"

# --- 5. global visibility: no projectId, global flag --------------------------
( write --text "Global lesson: never edit a plugin cache" --visibility global >"$S/global.out" 2>/dev/null ) || fail "global write failed"
wait_for_lines 2 || fail "global: Cortex never received the lesson"
python3 - "$STORE" <<'PY' || fail "global: record wrong"
import json, sys
r = [json.loads(l) for l in open(sys.argv[1])][-1]
assert r.get("global") is True and "projectId" not in r, r
PY
ok "global lesson saved without projectId, global flag set"

# --- 6. PROMETHEUS_LEARNING_CORTEX=0 disables the mirror ----------------------
( PROMETHEUS_LEARNING_CORTEX=0 write --text "Disabled lesson: mirror switched off" >"$S/off.out" 2>"$S/off.err" ) || fail "disabled write failed"
sleep 1
[ "$(wc -l < "$STORE" | tr -d ' ')" = "2" ] || fail "mirror ran although disabled"
[ ! -s "$S/off.err" ] || fail "disabled: stderr not empty"
ok "PROMETHEUS_LEARNING_CORTEX=0 disables the mirror"

# --- 7. real Cortex: scratch CORTEX_DATA_DIR, real stdio JSON-RPC ------------------
EXPECTED_VERSION="${CORTEX_EXPECTED_VERSION:-2.0.3}"
CORTEX_CACHE_ROOT="$REAL_HOME/.claude/plugins/cache/cortex"

# real_case <plugin dir> — exits 2 (BLOCKED) when the model cache is absent, 1 on failure.
real_case() {
  dir="$1"
  command -v node >/dev/null 2>&1 || blocked "node missing"
  [ -f "$dir/dist/mcp-server.js" ] || blocked "no dist/mcp-server.js in $dir"
  [ -d "$dir/node_modules/@xenova/transformers/.cache" ] || blocked "Cortex embedding model cache missing ($dir/node_modules/@xenova/transformers/.cache); refusing to download into the plugin cache"
  version="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["version"])' "$dir/package.json" 2>/dev/null)"
  [ "$version" = "$EXPECTED_VERSION" ] || fail "Cortex version is '$version', expected '$EXPECTED_VERSION' (set CORTEX_EXPECTED_VERSION to override)"
  echo "# real Cortex version: $version ($dir)"

  db_stat() { python3 -c 'import os,sys
try:
    st = os.stat(sys.argv[1]); print(int(st.st_mtime_ns), st.st_size)
except OSError:
    print("absent")' "$REAL_HOME/.cortex/memory.db"; }
  before_db="$(db_stat)"
  marker="$S/cache-marker"; touch "$marker"; sleep 1

  export CORTEX_DATA_DIR="$S/cortex-data"
  mkdir -p "$CORTEX_DATA_DIR"
  nonce="real-$$-$(date +%s)"
  lesson="Real Cortex lesson $nonce: the mirror must keep the server alive until it answers"
  server="node $dir/dist/mcp-server.js"
  ( PROMETHEUS_CORTEX_MCP="$server" write --text "$lesson" --kind gotcha >"$S/real.out" 2>"$S/real.err" ) || fail "real: write exited non-zero"
  [ ! -s "$S/real.err" ] || fail "real: stderr not empty: $(cat "$S/real.err")"

  # The mirror is detached and the real server loads a model first, so poll recall.
  n=0; found=""
  while [ "$n" -lt 48 ]; do
    found="$(real_recall "$server" "$nonce" project:d3-fixture 2>"$S/real-recall.err")" && [ -n "$found" ] && break
    found=""; sleep 5; n=$((n + 1))
  done
  [ -n "$found" ] || fail "real: cortex_recall never returned the lesson for project:d3-fixture ($(cat "$S/real-recall.err" 2>/dev/null))"
  printf '%s' "$found" | grep -q "$lesson" || fail "real: recalled content lacks the lesson text: $found"
  printf '%s' "$found" | grep -q 'team/role:cx-fixture/api-dev' || fail "real: recalled content lacks the role tag: $found"
  other="$(real_recall "$server" "$nonce" project:other 2>/dev/null)" || true
  [ -z "$other" ] || fail "real: lesson leaked into another projectId: $other"
  ok "real Cortex $version: lesson recalled by projectId with role tag"

  [ "$(db_stat)" = "$before_db" ] || fail "real: ~/.cortex/memory.db changed (before: $before_db, after: $(db_stat)) - another Cortex session may be running; rerun on a quiet machine"
  stray="$(find "$CORTEX_CACHE_ROOT" -newer "$marker" 2>/dev/null | head -3)"
  [ -z "$stray" ] || fail "real: files written under the plugin cache: $stray"
  ok "real Cortex: ~/.cortex/memory.db and the plugin cache are untouched"
}

# real_recall <server argv> <query> <projectId> — prints the recalled content (all hits)
# when any hit contains the query, nothing otherwise. Real stdio JSON-RPC, stdin held open.
real_recall() {
  python3 - "$1" "$2" "$3" <<'PY'
import json, shlex, subprocess, sys, time
argv, query, project = shlex.split(sys.argv[1]), sys.argv[2], sys.argv[3]
proc = subprocess.Popen(argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
reqs = [{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/call",
         "params": {"name": "cortex_recall", "arguments": {"query": query, "projectId": project, "limit": 10}}}]
try:
    proc.stdin.write("\n".join(json.dumps(r) for r in reqs) + "\n"); proc.stdin.flush()
    reply = None
    deadline = time.time() + 120
    while time.time() < deadline:
        line = proc.stdout.readline()
        if not line:
            break
        try:
            msg = json.loads(line)
        except ValueError:
            continue
        if msg.get("id") == 2:
            reply = msg
            break
finally:
    try:
        proc.stdin.close(); proc.wait(timeout=10)
    except Exception:
        proc.kill()
if reply is None or "result" not in reply:
    sys.exit(1)
hits = json.loads(reply["result"]["content"][0]["text"]).get("results", [])
text = "\n".join(h.get("content", "") for h in hits if query in h.get("content", ""))
print(text)
PY
}

# Absent-model simulation: discovery pointed at a copy without node_modules must exit 2.
NOMODEL="$S/nomodel/cortex/$EXPECTED_VERSION"
mkdir -p "$NOMODEL/dist"
: > "$NOMODEL/dist/mcp-server.js"
printf '{"version":"%s"}\n' "$EXPECTED_VERSION" > "$NOMODEL/package.json"
( real_case "$NOMODEL" >/dev/null 2>&1 ); rc=$?
[ "$rc" = "2" ] || fail "model cache absent: real case exited $rc, expected 2 (BLOCKED)"
ok "model cache absent: real case exits 2 (BLOCKED), not 0"

if [ "${CORTEX_REAL:-}" = "skip" ]; then
  echo "SKIP: real Cortex case (CORTEX_REAL=skip)"
else
  REAL_DIR="$(ls -d "$CORTEX_CACHE_ROOT"/*/*/dist/mcp-server.js 2>/dev/null | sort | tail -1)"
  [ -n "$REAL_DIR" ] || blocked "Cortex not installed under $CORTEX_CACHE_ROOT"
  REAL_DIR="$(dirname "$(dirname "$REAL_DIR")")"
  ( real_case "$REAL_DIR" ); rc=$?
  [ "$rc" = "0" ] || exit "$rc"
fi

echo "PASS: $pass checks"
