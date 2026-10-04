#!/bin/bash
# test-promotion-routing.sh — C1b gate: [GLOBAL]/[USER] routing, recall quotas, candidate review.
#
#   test-promotion-routing.sh
#
# Real processes end to end, no mocks of the behaviour under test:
#   1. memory-writeback.sh, driven as the PostToolUse(Write|Edit) hook, persists a
#      reflection with 5 [GLOBAL], 5 [USER] and 1 unmarked lesson in project A:
#      [GLOBAL] -> @global/@global, [USER] -> @user:<hash>/@user, unmarked ->
#      project:<A>/@project, each recorded with the real learning_write queue;
#   2. the detached `pk ingest` calls (a recording `pk` stands in only for the
#      model-backed ingest) carry `--scope shared` for [GLOBAL]/[USER] and not for
#      project lessons;
#   3. a subagent in a third scratch project C (different project id, same user)
#      recalls through the real learning_recall.py: it receives the global and user
#      lessons within the top-3 quota per scope and never project A's lesson. The
#      store is a scratch HTTP server that serves the queued operations by
#      (user_id, agent_id), the equality filter surreal-memory applies;
#   4. kbd-open.sh with the real pk >= 1.11.0 lists a pending promotion candidate
#      (`pk candidates list --kind promotion`), is silent when none is pending, and
#      with a pk that predates `candidates` prints no section and exits 0.
#
# Isolation: scratch HOME, CODEX_HOME, PROMETHEUS_PLUGIN_ROOT, queue, log and index;
# the scratch store is killed on exit. Binary: TLI_PK_BIN (else PATH) must be
# pk >= 1.11.0 for step 4; without it everything else still runs and the script
# exits 2 (BLOCKED), never 0. Exit 0 pass, 1 fail, 2 BLOCKED. bash 3.2.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
WRITEBACK="$ROOT/shared/scripts/memory-writeback.sh"
KBD_OPEN="$ROOT/shared/scripts/kbd-open.sh"
RECALL="$ROOT/shared/scripts/lib/learning_recall.py"
PK_BIN="${TLI_PK_BIN:-$(command -v pk || true)}"

for tool in python3 jq git; do command -v "$tool" >/dev/null 2>&1 || { echo "BLOCKED: $tool missing" >&2; exit 2; }; done

S="$(mktemp -d "${TMPDIR:-/tmp}/tli-c1b.XXXXXX")"
S="$(cd "$S" && pwd -P)"
srv=""
cleanup() { [ -n "$srv" ] && kill "$srv" 2>/dev/null; wait 2>/dev/null; rm -rf "$S"; }
trap cleanup EXIT
export HOME="$S/home" CODEX_HOME="$S/codex" PROMETHEUS_PLUGIN_ROOT="$S/plugin-root"
export PROMETHEUS_LEARNING_QUEUE="$S/queue" PROMETHEUS_LEARNING_LOG_DIR="$S/log" PROMETHEUS_LEARNING_INDEX_DIR="$S/index"
export PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=1
unset PROMETHEUS_LEARNING_PK PROMETHEUS_USER_ID PROMETHEUS_PROJECT_ID PROMETHEUS_HARNESS PK_KB_DIR CLAUDE_PLUGIN_ROOT PLUGIN_ROOT
mkdir -p "$HOME" "$CODEX_HOME" "$PROMETHEUS_PLUGIN_ROOT" "$S/bin"
git config --global user.email c1b-fixture@example.invalid
git config --global user.name Fixture

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }
mkrepo() { # <dir> <project id>
  mkdir -p "$1/.prometheus"; git -C "$1" init -q
  printf '{"projectId":"%s"}\n' "$2" > "$1/.prometheus/project.json"
  printf 'x\n' > "$1/f.txt"; git -C "$1" add -A >/dev/null; git -C "$1" commit -qm fixture
}

# --- recording pk: stands in for the model-backed `pk ingest` only -----------
INGEST_LOG="$S/ingest.log"
cat > "$S/bin/pk" <<SH
#!/bin/sh
if [ "\$1" = ingest ]; then
  body="\$(cat)"
  printf '%s\t%s\n' "\$*" "\$(printf '%s' "\$body" | tr '\n' ' ')" >> "$INGEST_LOG"
  exit 0
fi
exit 2
SH
chmod +x "$S/bin/pk"

# --- 1. project A: reflection with marked and unmarked lessons ---------------
A="$S/proj-a"; mkrepo "$A" project:c1b-a
PHASE="$A/.kbd-orchestrator/phases/p1"; mkdir -p "$PHASE"
echo '{ "phase": "p1" }' > "$A/.kbd-orchestrator/current-waypoint.json"
echo '{ "phase": "p1" }' > "$PHASE/progress.json"
{
  printf '# Reflection - p1\n\n## Delta\n1. Something was missed.\n\n## Root Cause\n1. The path was assumed.\n\n## Lessons Learned\n'
  i=1; while [ $i -le 5 ]; do
    printf -- '- [GLOBAL] GLOBALTOKEN-%s keep exact command lines in every runbook entry number %s.\n' "$i" "$i"
    printf -- '- [USER] USERTOKEN-%s prefer the smaller reproducible check on this machine, note %s.\n' "$i" "$i"
    i=$((i + 1))
  done
  printf -- '- PROJECTTOKEN-1 only project A uses the staging gateway alias.\n'
} > "$PHASE/reflection.md"
( cd "$A" && printf '{"tool_input":{"file_path":"%s"}}' "$PHASE/reflection.md" | PATH="$S/bin:$PATH" bash "$WRITEBACK" >/dev/null 2>&1 ) || fail "memory-writeback exited non-zero"

PENDING="$PROMETHEUS_LEARNING_QUEUE/memory/pending"
USER_SCOPE="$(cd "$A" && python3 "$ROOT/shared/scripts/lib/project_id.py" --json | jq -r .userScope)"
case "$USER_SCOPE" in user:*) ;; *) fail "unexpected user scope $USER_SCOPE" ;; esac
count_ops() { # <user_id> <agent_id> <content substring>
  find "$PENDING" -type f -name '*.json' -exec cat {} + | jq -s --arg u "$1" --arg a "$2" --arg c "$3" \
    '[.[] | select(.arguments.user_id == $u and .arguments.agent_id == $a and (.arguments.content | contains($c)))] | length'
}
[ "$(count_ops @global @global GLOBALTOKEN-)" = 5 ] || fail "5 [GLOBAL] lessons must be stored under @global/@global"
[ "$(count_ops "@$USER_SCOPE" @user USERTOKEN-)" = 5 ] || fail "5 [USER] lessons must be stored under @$USER_SCOPE/@user"
[ "$(count_ops project:c1b-a @project PROJECTTOKEN-1)" = 1 ] || fail "the unmarked lesson must stay in project:c1b-a/@project"
[ "$(count_ops @global @global PROJECTTOKEN)" = 0 ] && [ "$(count_ops "@$USER_SCOPE" @user PROJECTTOKEN)" = 0 ] \
  || fail "an unmarked lesson leaked into a shared scope"
[ "$(count_ops project:c1b-a @project GLOBALTOKEN-)" = 0 ] || fail "a [GLOBAL] lesson must not also be stored at project scope"
ok "[GLOBAL] -> @global, [USER] -> @$USER_SCOPE, unmarked -> project:c1b-a/@project"

# --- 2. pk ingest scope flags -------------------------------------------------
n=0; while [ $n -lt 50 ] && [ "$(wc -l < "$INGEST_LOG" 2>/dev/null | tr -d ' ')" != 12 ]; do sleep 0.2; n=$((n + 1)); done
[ "$(wc -l < "$INGEST_LOG" 2>/dev/null | tr -d ' ')" = 12 ] || fail "expected 12 detached pk ingest calls (reflection record, 5+5 marked, 1 unmarked), got $(wc -l < "$INGEST_LOG" 2>/dev/null)"
shared_calls="$(grep -c -- '--scope shared' "$INGEST_LOG")"
[ "$shared_calls" = 10 ] || fail "10 ingests must use --scope shared, got $shared_calls"
grep -- 'PROJECTTOKEN-1' "$INGEST_LOG" | grep -q -- '--scope shared' && fail "the project lesson was ingested into the shared KB"
grep -- 'GLOBALTOKEN-1' "$INGEST_LOG" | grep -q -- 'vis:global' || fail "global ingest lacks vis:global"
grep -- 'USERTOKEN-1' "$INGEST_LOG" | grep -q -- 'vis:user' || fail "user ingest lacks vis:user"
ok "[GLOBAL]/[USER] lessons ingest into pk shared; the project lesson does not"

# --- 3. recall in a third scratch project C ----------------------------------
C="$S/proj-c"; mkrepo "$C" project:c1b-c
cat > "$S/store.py" <<'PY'
import glob, json, sys, urllib.parse
from http.server import BaseHTTPRequestHandler, HTTPServer
queue, portfile = sys.argv[1], sys.argv[2]
def records(uid, aid):
    out = []
    for path in sorted(glob.glob(queue + "/memory/*/*.json")):
        try: args = json.load(open(path))["arguments"]
        except Exception: continue
        if args.get("user_id") == uid and args.get("agent_id") == aid:
            out.append({"id": path, "content": args["content"], "user_id": uid, "agent_id": aid,
                        "categories": args.get("categories", []), "created_at": "2026-10-04T00:00:00Z"})
    return out
class H(BaseHTTPRequestHandler):
    def log_message(self, *a): pass
    def _send(self, payload):
        body = json.dumps(payload).encode(); self.send_response(200)
        self.send_header("Content-Type", "application/json"); self.send_header("Content-Length", str(len(body)))
        self.end_headers(); self.wfile.write(body)
    def do_GET(self):
        url = urllib.parse.urlparse(self.path)
        if url.path == "/health": return self._send({"status": "ok"})
        q = urllib.parse.parse_qs(url.query)
        self._send({"memories": records(q.get("user_id", [""])[0], q.get("agent_id", [""])[0])})
    def do_POST(self):
        length = int(self.headers.get("Content-Length") or 0)
        body = json.loads(self.rfile.read(length) or b"{}")
        self._send({"memories": records(body.get("user_id"), body.get("agent_id"))[: int(body.get("limit") or 10)]})
srv = HTTPServer(("127.0.0.1", 0), H)
open(portfile, "w").write(str(srv.server_address[1]))
srv.serve_forever()
PY
python3 "$S/store.py" "$PROMETHEUS_LEARNING_QUEUE" "$S/port" & srv=$!
n=0; while [ $n -lt 50 ] && [ ! -s "$S/port" ]; do sleep 0.1; n=$((n + 1)); done
[ -s "$S/port" ] || fail "scratch store did not start"
URL="http://127.0.0.1:$(cat "$S/port")"
RECALLED="$(cd "$C" && python3 "$RECALL" --cwd "$C" --main-thread --no-pk --no-log --memory-url "$URL" --format json)" \
  || fail "learning_recall failed in project C"
printf '%s' "$RECALLED" | jq -e '.storeReachable == true and .lessonSource == "surreal-memory"' >/dev/null || fail "recall did not use the store"
g="$(printf '%s' "$RECALLED" | jq '[.lessons[] | select(.scope == "global")] | length')"
u="$(printf '%s' "$RECALLED" | jq '[.lessons[] | select(.scope == "user")] | length')"
[ "$g" = 3 ] || fail "project C must receive exactly 3 of the 5 global lessons (the global quota), got $g"
[ "$u" = 3 ] || fail "project C must receive exactly 3 of the 5 user lessons (the user quota), got $u"
printf '%s' "$RECALLED" | jq -e '[.lessons[].line] | any(contains("GLOBALTOKEN-"))' >/dev/null || fail "no global lesson text recalled"
printf '%s' "$RECALLED" | jq -e '[.lessons[].line] | any(contains("PROJECTTOKEN"))' >/dev/null && fail "project A's lesson reached project C"
ok "project C recalls 3 global + 3 user lessons (quota), and no project A lesson"

# --- 4. kbd-open promotion candidates ----------------------------------------
version_ok() { "$1" --version 2>/dev/null | grep -Eq '1\.(1[1-9]|[2-9][0-9])\.'; }
snapshot() { ( cd "$C" && PATH="$1" bash "$KBD_OPEN" 2>/dev/null ); }
BASEPATH="/usr/bin:/bin:/usr/sbin:/sbin"
for tool in python3; do d="$(dirname "$(command -v $tool)")"; case ":$BASEPATH:" in *":$d:"*) ;; *) BASEPATH="$BASEPATH:$d" ;; esac; done

# a pk that predates `candidates`: no section, exit 0
out="$(snapshot "$S/bin:$BASEPATH")"; rc=$?
[ "$rc" = 0 ] || fail "kbd-open exited $rc with a pk that has no candidates subcommand"
printf '%s' "$out" | grep -q 'Promotion candidates' && fail "older pk printed a promotion section"
out="$(snapshot "$BASEPATH")"; rc=$?
[ "$rc" = 0 ] && ! printf '%s' "$out" | grep -q 'Promotion candidates' || fail "no pk at all must also be silent and exit 0"
ok "older or absent pk: no promotion section, exit 0"

if [ -x "$PK_BIN" ] && version_ok "$PK_BIN"; then
  PKDIR="$(dirname "$PK_BIN")"
  out="$(snapshot "$PKDIR:$BASEPATH")"; rc=$?
  [ "$rc" = 0 ] || fail "kbd-open exited $rc with pk $("$PK_BIN" --version)"
  printf '%s' "$out" | grep -q 'Promotion candidates' && fail "section printed with no pending candidate"
  ok "pk >= 1.11.0 with nothing pending: silent"
  CAND="$HOME/.prometheus/promotion-candidates/pending"; mkdir -p "$CAND"
  cat > "$CAND/cand-c1b-0001.json" <<'JSON'
{"schemaVersion":1,"id":"cand-c1b-0001","kind":"promotion","state":"pending","scope":"global","reasons":["recurrence"],
 "title":"Anchor hook matchers to the plugin prefix","content":"Anchor hook matchers to the plugin prefix.","tags":["hooks"],"fingerprint":"f",
 "evidence":[{"lessonId":"l1","projectId":"project:alpha","projectRoot":"/alpha","source":"s1#0","recordedAt":"2026-10-04T00:00:00Z","similarity":1.0},
             {"lessonId":"l2","projectId":"project:beta","projectRoot":"/beta","source":"s2#0","recordedAt":"2026-10-04T00:00:00Z","similarity":1.0}],
 "createdAt":"2026-10-04T00:00:00Z","updatedAt":"2026-10-04T00:00:00Z"}
JSON
  out="$(snapshot "$PKDIR:$BASEPATH")"; rc=$?
  [ "$rc" = 0 ] || fail "kbd-open exited $rc with a pending candidate"
  printf '%s' "$out" | grep -q '## Promotion candidates awaiting review (1)' || fail "pending candidate not listed: $out"
  printf '%s' "$out" | grep -q 'cand-c1b-0001' && printf '%s' "$out" | grep -q 'Anchor hook matchers' || fail "candidate line lacks id/title"
  printf '%s' "$out" | grep -q '2 evidence, global' || fail "candidate line lacks evidence count and scope"
  ok "pk >= 1.11.0: the pending promotion candidate is listed with evidence and scope"
  "$PK_BIN" candidates reject cand-c1b-0001 --reason test >/dev/null 2>&1 || fail "pk candidates reject failed"
  out="$(snapshot "$PKDIR:$BASEPATH")"
  printf '%s' "$out" | grep -q 'Promotion candidates' && fail "rejected candidate is still listed"
  ok "after a human rejects it the candidate leaves the section"
  echo "PASS: $pass checks"; exit 0
fi
echo "BLOCKED: pk >= 1.11.0 not found (set TLI_PK_BIN); candidate-listing checks not run ($pass checks passed)" >&2
exit 2
