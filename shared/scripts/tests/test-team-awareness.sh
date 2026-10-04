#!/bin/bash
# test-team-awareness.sh — B7 beta gate: path-overlap routing, team digest, lead view.
#
#   test-team-awareness.sh [--harness claude|codex|both|none]   (default: both)
#   (`none` runs everything except the two model-driven harness runs)
#
# Real processes end to end, driving the GENERATED hook entries (design §7):
#   1. fixture team `tlm-fixture` (api-dev owns src/api/**, ui-dev owns src/ui/**)
#      in a scratch git repo. Cycle 1 — ONE real SubagentStop payload for api-dev
#      is run through the GENERATED subagentstop-learning entry (behind
#      team-role-guard); its paths come from the hook, not from a --paths call:
#        BETA-PRIV   `paths: src/api/handler.ts` suffix   only api-dev owns them: private
#        BETA-ROUTED no suffix; paths taken from the subagent transcript's Write
#                    tool_use (src/ui/client.ts)          ui-dev owns them: to ui-dev
#        BETA-LEAD   `(paths: docs/readme.md)` suffix     nobody owns them: to lead
#      The real prometheus-learning-worker delivers them to a scratch
#      surreal-memory-server 1.10 (embedded, local MLX embeddings);
#   2. routing rules on a second team (5 roles): > 3 owners -> lead only, 2
#      owners -> two role copies, no owner -> lead, no paths -> unrouted;
#   3. digest: each line <= 160 chars, no lesson text, mirrored under
#      <team>/@team, a repeated lesson adds nothing, rotation at 1,000 lines
#      with no temp file left behind;
#   4. every stored record validates against learning-envelope.schema.json and
#      carries the design-table agent_id;
#   5. cycle 2 — the generated `subagentstart-learning` entry, run directly and
#      then by Claude Code and Codex subagents: api-dev receives BETA-PRIV, ui-dev
#      receives BETA-ROUTED and only the digest line for BETA-PRIV (never its
#      text, never BETA-LEAD);
#   6. the main thread receives BETA-LEAD and the digest lines, and no
#      role-private text: through learning_recall --main-thread and through the
#      GENERATED sessionstart-learning entry (plain-text fenced context, silent
#      for a subagent payload and for a repo without a team), and live in both
#      harnesses;
#   7. scripts/report-learning-delivery.py prints bytes per agent per channel and
#      honours --require-reduction;
#   8. surreal-memory stopped: the digest file still reaches ui-dev and the main
#      thread, and role-private text still does not.
#
# Isolation: scratch HOME, CODEX_HOME (auth.json symlinked, never read or copied),
# PROMETHEUS_PLUGIN_ROOT, queue, log, index and team-digest directory. Claude Code
# authenticates only with the real HOME, so the Claude run keeps it, excludes user
# settings (--setting-sources project,local), and loads a scratch copy of the
# generated package whose hooks.json carries only the generated SubagentStart and
# SessionStart-learning groups.
# The scratch digest, index and queue directories are exported, so nothing is
# written under the real ~/.prometheus.
#
# Binaries: TLI_SM_BIN, TLI_WORKER_BIN, TLI_PK_BIN (else PATH), all >= 1.10.0.
# Exit 0 pass, 1 fail, 2 BLOCKED (a prerequisite is missing). bash 3.2.
set -u

HARNESS=both
while [ $# -gt 0 ]; do
  case "$1" in
    --harness) HARNESS="${2:-}"; shift 2 ;;
    *) echo "usage: $0 [--harness claude|codex|both|none]" >&2; exit 64 ;;
  esac
done
case "$HARNESS" in claude|codex|both|none) ;; *) echo "bad --harness: $HARNESS" >&2; exit 64 ;; esac
want() { [ "$HARNESS" = both ] || [ "$HARNESS" = "$1" ]; }

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
LW="$ROOT/shared/scripts/lib/learning_write.py"
LR="$ROOT/shared/scripts/lib/learning_recall.py"
REPORT="$ROOT/scripts/report-learning-delivery.py"
SCHEMA="$ROOT/shared/schemas/learning-envelope.schema.json"
CLAUDE_PKG="$ROOT/dist/plugins/claude/prometheus-skill-pack"
CODEX_PKG="$ROOT/dist/plugins/codex/prometheus-skill-pack"
SM_BIN="${TLI_SM_BIN:-$(command -v surreal-memory-server || true)}"
WORKER_BIN="${TLI_WORKER_BIN:-$(command -v prometheus-learning-worker || true)}"
PK_BIN="${TLI_PK_BIN:-$(command -v pk || true)}"
PORT="${TLI_SM_PORT:-23027}"
REAL_HOME="$HOME"
MODEL_TIMEOUT="${TLI_MODEL_TIMEOUT:-900}"

blocked() { echo "BLOCKED: $*" >&2; exit 2; }
version_ok() { "$1" --version 2>/dev/null | grep -Eq '1\.(1[0-9]|[2-9][0-9])\.'; }
[ -x "$SM_BIN" ] || blocked "surreal-memory-server not found (set TLI_SM_BIN)"
[ -x "$WORKER_BIN" ] || blocked "prometheus-learning-worker not found (set TLI_WORKER_BIN)"
[ -x "$PK_BIN" ] || blocked "pk not found (set TLI_PK_BIN or put pk >= 1.10 on PATH)"
version_ok "$SM_BIN" || blocked "surreal-memory-server < 1.10.0"
version_ok "$WORKER_BIN" || blocked "prometheus-learning-worker < 1.10.0"
version_ok "$PK_BIN" || blocked "pk < 1.10.0"
for tool in python3 node git curl; do command -v "$tool" >/dev/null 2>&1 || blocked "$tool missing"; done
python3 -c 'import jsonschema' 2>/dev/null || blocked "python3 jsonschema missing (envelope validation needs it)"
[ -x /usr/local/bin/surreal-memory-mlx-executor ] || blocked "MLX embedding executor missing"
want claude && { command -v claude >/dev/null 2>&1 || blocked "claude CLI not on PATH"; }
want codex && { command -v codex >/dev/null 2>&1 || blocked "codex CLI not on PATH"; }
want codex && { [ -f "$REAL_HOME/.codex/auth.json" ] || blocked "no Codex login (~/.codex/auth.json absent)"; }
for pkg in "$CLAUDE_PKG" "$CODEX_PKG"; do
  grep -q 'subagentstart-learning' "$pkg/hooks/hooks.json" 2>/dev/null \
    || blocked "generated package lacks subagentstart-learning: $pkg (run the generators)"
  cmp -s "$ROOT/shared/scripts/lib/learning_route.py" "$pkg/shared/scripts/lib/learning_route.py" 2>/dev/null \
    || blocked "generated package is stale (learning_route.py differs or is missing): $pkg (run the generators)"
done
if curl -fsS -m 1 "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; then blocked "port $PORT is already in use (set TLI_SM_PORT)"; fi

S="$(mktemp -d "${TMPDIR:-/tmp}/tli-b7.XXXXXX")"
S="$(cd "$S" && pwd -P)"
srv=""
cleanup() { [ -n "$srv" ] && kill "$srv" 2>/dev/null; wait 2>/dev/null; if [ -n "${TLI_KEEP:-}" ]; then echo "kept $S" >&2; else rm -rf "$S"; fi; }
trap cleanup EXIT
export HOME="$S/home" CODEX_HOME="$S/codex" PROMETHEUS_PLUGIN_ROOT="$S/plugin-root"
export PROMETHEUS_LEARNING_QUEUE="$S/queue" PROMETHEUS_LEARNING_LOG_DIR="$S/log" PROMETHEUS_LEARNING_INDEX_DIR="$S/index"
export PROMETHEUS_TEAM_DIGEST_DIR="$S/team-digest"
export PROMETHEUS_LEARNING_DELIVERY_TRACE_DIR="$S/trace" PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=1 PROMETHEUS_USER_ID=b7-fixture
export SURREAL_MEMORY_URL="http://127.0.0.1:$PORT" PATH="$(dirname "$PK_BIN"):$PATH"
unset PROMETHEUS_LEARNING_PK CLAUDE_PLUGIN_ROOT PLUGIN_ROOT KBD_PACK_ROOT PK_KB_DIR PROMETHEUS_HARNESS PROMETHEUS_LEARNING_ROUTE PROMETHEUS_TEAM_DIGEST
mkdir -p "$HOME" "$CODEX_HOME" "$PROMETHEUS_PLUGIN_ROOT"
git config --global user.email fixture@example.invalid; git config --global user.name Fixture

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }
run_timeout() { # <seconds> <cmd...>   (bash 3.2: no coreutils timeout)
  local secs="$1"; shift
  "$@" & local pid=$!
  ( sleep "$secs"; kill -TERM "$pid" 2>/dev/null ) >/dev/null 2>&1 & local wd=$!
  wait "$pid"; local rc=$?
  kill "$wd" 2>/dev/null; wait "$wd" 2>/dev/null
  return $rc
}

# --- 1. fixture repos -----------------------------------------------------------
PROJECT="project:b7-fixture"
REPO="$S/repo"
mkdir -p "$REPO/.agent-team/tlm-fixture" "$REPO/.prometheus" "$REPO/src/api" "$REPO/src/ui" "$REPO/docs" "$REPO/.codex/agents"
git -C "$REPO" init -q
printf '{"projectId":"%s"}\n' "$PROJECT" > "$REPO/.prometheus/project.json"
cat > "$REPO/.agent-team/tlm-fixture/team.json" <<'EOF'
{"schemaVersion":1,"id":"tlm-fixture","roles":[
 {"id":"api-dev","description":"api","prompt":"p","skills":[],"owns":["src/api/**"],"inputs":[],"outputs":[],"dependsOn":[]},
 {"id":"ui-dev","description":"ui","prompt":"p","skills":[],"owns":["src/ui/**"],"inputs":[],"outputs":[],"dependsOn":[]}]}
EOF
PROBE_INSTRUCTIONS='You are a delivery probe. Look in your context for tokens of the form BETA-<WORD>. Reply with exactly those tokens separated by single spaces, or NONE if there are none. Never use tools.'
for role in api_dev ui_dev; do
  printf 'name = "%s"\ndescription = "Fixture role %s. Use only when explicitly asked to spawn %s."\ndeveloper_instructions = "%s"\n' \
    "$role" "$role" "$role" "$PROBE_INSTRUCTIONS" > "$REPO/.codex/agents/$role.toml"
done
printf 'x\n' > "$REPO/src/api/a.ts"; printf 'x\n' > "$REPO/src/ui/u.ts"; printf 'x\n' > "$REPO/docs/readme.md"
git -C "$REPO" add -A >/dev/null; git -C "$REPO" commit -qm fixture

# a second team for the routing rules (five roles, overlapping globs)
REPO2="$S/repo-route"
mkdir -p "$REPO2/.agent-team/tlm-route" "$REPO2/.prometheus" "$REPO2/pkg/shared" "$REPO2/pkg/pair" "$REPO2/pkg/own1"
git -C "$REPO2" init -q
printf '{"projectId":"project:b7-route"}\n' > "$REPO2/.prometheus/project.json"
python3 - "$REPO2/.agent-team/tlm-route/team.json" <<'PY'
import json, sys
owns = {"r1": ["pkg/shared/**", "pkg/pair/**", "pkg/own1/**"], "r2": ["pkg/shared/**", "pkg/pair/**"],
        "r3": ["pkg/shared/**", "pkg/pair/**"], "r4": ["pkg/shared/**"], "r5": ["pkg/shared/**"]}
json.dump({"schemaVersion": 1, "id": "tlm-route", "roles": [
    {"id": r, "description": r, "prompt": "p", "skills": [], "owns": g, "inputs": [], "outputs": [], "dependsOn": []}
    for r, g in owns.items()]}, open(sys.argv[1], "w"))
PY
git -C "$REPO2" add -A >/dev/null; git -C "$REPO2" commit -qm fixture

write_as() { # <repo> <agent_type> <args...>   -> learning_write JSON on stdout
  local repo="$1" agent="$2"; shift 2
  printf '%s' '{"agent_type":"'"$agent"'","agent_id":"seed-'"$agent"'","session_id":"s-b7","cwd":"'"$repo"'"}' \
    | python3 "$LW" --payload-stdin --cwd "$repo" "$@"
}
scopes_of() { # <learning_write json> -> sorted agent_ids of the lesson copies
  python3 -c 'import json,sys;print(" ".join(sorted(o["agent_id"] for o in json.loads(sys.argv[1])["operations"])))' "$1"
}

# --- 2. scratch surreal-memory -------------------------------------------------
mkdir -p "$S/sm"
( cd "$S/sm" && exec env SURREAL_MODE=embedded SURREAL_PATH="$S/sm/db" SURREAL_NAMESPACE=b7 SURREAL_DATABASE=gate \
    API_HOST=127.0.0.1 API_PORT="$PORT" MCP_STDIO=false EMBEDDING_PROVIDER=local LOCAL_EMBEDDING_BACKEND=mlx \
    LOCAL_EMBEDDING_EXECUTOR=/usr/local/bin/surreal-memory-mlx-executor LOCAL_EMBEDDING_MODEL=BAAI/bge-small-en-v1.5 \
    LOCAL_EMBEDDING_MODEL_REVISION=5c38ec7c405ec4b44b94cc5a9bb96e735b38267a LOCAL_EMBEDDING_DIMENSIONS=384 \
    HF_HUB_CACHE="$REAL_HOME/.cache/huggingface/hub" MODEL_CACHE_DIR="$REAL_HOME/.cache/huggingface" \
    SURREAL_EXECUTOR_STARTUP_MS=300000 RUST_LOG=warn "$SM_BIN" > "$S/sm/server.log" 2>&1 ) &
srv=$!
for _ in $(seq 1 180); do curl -fsS -m 1 "$SURREAL_MEMORY_URL/ready" 2>/dev/null | grep -q '"ledger":true' && break; sleep 1; done
curl -fsS -m 1 "$SURREAL_MEMORY_URL/ready" 2>/dev/null | grep -q '"ledger":true' || { tail -20 "$S/sm/server.log" >&2; blocked "scratch surreal-memory never became ready"; }

# --- 2b. the generated package, activated in the scratch plugin root -------------
BUNDLE="$(node -e 'process.stdout.write(JSON.parse(require("fs").readFileSync(process.argv[1])).bundleId)' "$CLAUDE_PKG/shared/harnesses/generated/release-manifest.json")"
grep -q -- "--bundle $BUNDLE --hook subagentstart-learning" "$CODEX_PKG/hooks/hooks.json" || fail "codex package bundle differs from claude package"
bash "$CLAUDE_PKG/shared/scripts/bootstrap-hook-runtime.sh" --source-root "$CLAUDE_PKG" --expected-bundle "$BUNDLE" >"$S/bootstrap.log" 2>&1 \
  || { cat "$S/bootstrap.log" >&2; fail "could not activate the generated bundle in the scratch PROMETHEUS_PLUGIN_ROOT"; }
hook_entry() { # <hook> <harness> <payload>   -> stdout of the generated entry
  printf '%s' "$3" | CLAUDE_PLUGIN_ROOT="$CLAUDE_PKG" node "$CLAUDE_PKG/scripts/hook-entry.mjs" \
    --bundle "$BUNDLE" --hook "$1" --harness "$2"
}
hook_direct() { hook_entry subagentstart-learning "$1" "$2"; }   # <harness> <payload>

# --- 3. cycle 1: api-dev's lessons, routing and digest ---------------------------
T_PRIV="BETA-PRIV is the api-dev handler marker: keep the retry budget for the handler private to api-dev."
T_ROUTED="BETA-ROUTED is the client contract marker: the ui client must send the idempotency header the handler expects."
T_LEAD="BETA-LEAD is the lead marker: the readme rewrite waits for the release owner."
SUBAGENT_TRANSCRIPT="$S/api-dev-transcript.jsonl"
python3 - "$SUBAGENT_TRANSCRIPT" "$REPO" <<'PY' || fail "build the subagent transcript fixture"
import json, sys
path, repo = sys.argv[1:]
def tool(name, **inp): return {"type": "assistant", "message": {"role": "assistant", "content": [{"type": "tool_use", "id": "t", "name": name, "input": inp}]}}
rows = [tool("Read", file_path=f"{repo}/src/api/a.ts"),            # reads are not writes
        tool("Write", file_path=f"{repo}/src/ui/client.ts"),       # absolute, inside the repo
        tool("Write", file_path="/etc/outside-the-repo.conf"),     # outside the repo: dropped
        tool("Write", file_path=f"{repo}/src/ui/client.ts"),       # duplicate: once
        {"type": "user", "message": {"role": "user", "content": "noise"}}]
open(path, "w").write("\n".join(json.dumps(r) for r in rows) + "\n")
PY
T_PRIV_LINE="$T_PRIV paths: src/api/handler.ts"
T_LEAD_LINE="$T_LEAD (paths: docs/readme.md)"
STOP_PAYLOAD="$(python3 - "$REPO" "$SUBAGENT_TRANSCRIPT" "$T_PRIV_LINE" "$T_ROUTED" "$T_LEAD_LINE" <<'PY'
import json, sys
repo, transcript, priv, routed, lead = sys.argv[1:]
message = "Done.\nLESSON: " + priv + "\n- LESSON: " + routed + "\nGOTCHA: " + lead + "\nnot a lesson line"
print(json.dumps({"hook_event_name": "SubagentStop", "agent_type": "prometheus-skill-pack:api-dev", "agent_id": "seed-api-dev",
                  "session_id": "s-b7", "cwd": repo, "agent_transcript_path": transcript, "last_assistant_message": message}))
PY
)"
stop_out="$(hook_entry subagentstop-learning claude-code "$STOP_PAYLOAD" 2>&1)"; stop_rc=$?
[ "$stop_rc" -eq 0 ] || fail "subagentstop-learning exited $stop_rc"
[ -z "$stop_out" ] || fail "subagentstop-learning must stay silent, printed: $stop_out"
[ -n "$(ls "$PROMETHEUS_LEARNING_QUEUE/pending" 2>/dev/null | grep json)" ] || fail "the SubagentStop hook did not queue its learning job"
rm -f "$PROMETHEUS_LEARNING_QUEUE"/pending/*.json   # the model-driven extraction job is not under test here
scopes_of_log() { # <marker> -> sorted agent_ids the logged lesson was written to
  python3 - "$PROMETHEUS_LEARNING_LOG_DIR/lessons.jsonl" "$1" <<'PY'
import json, sys
rows = [json.loads(l) for l in open(sys.argv[1]) if l.strip()]
hit = [r for r in rows if sys.argv[2] in r["text"]]
assert len(hit) == 1, (sys.argv[2], len(hit))
print(" ".join(sorted(a for _, a in hit[0]["scopes"])))
PY
}
paths_of_log() { # <marker> -> the envelope paths the hook derived
  python3 - "$PROMETHEUS_LEARNING_LOG_DIR/lessons.jsonl" "$1" <<'PY'
import json, sys
rows = [json.loads(l) for l in open(sys.argv[1]) if l.strip()]
print(" ".join(next(r for r in rows if sys.argv[2] in r["text"])["envelope"].get("paths", [])))
PY
}
[ "$(paths_of_log BETA-PRIV)" = "src/api/handler.ts" ] || fail "BETA-PRIV paths from the LESSON suffix, got: $(paths_of_log BETA-PRIV)"
[ "$(paths_of_log BETA-ROUTED)" = "src/ui/client.ts" ] || fail "BETA-ROUTED paths from the transcript Write, got: $(paths_of_log BETA-ROUTED)"
[ "$(paths_of_log BETA-LEAD)" = "docs/readme.md" ] || fail "BETA-LEAD paths from the (paths: ...) suffix, got: $(paths_of_log BETA-LEAD)"
[ "$(scopes_of_log BETA-PRIV)" = "tlm-fixture/api-dev" ] || fail "BETA-PRIV must stay private to api-dev, got: $(scopes_of_log BETA-PRIV)"
[ "$(scopes_of_log BETA-ROUTED)" = "tlm-fixture/api-dev tlm-fixture/ui-dev" ] || fail "BETA-ROUTED must reach ui-dev, got: $(scopes_of_log BETA-ROUTED)"
[ "$(scopes_of_log BETA-LEAD)" = "tlm-fixture/@lead tlm-fixture/api-dev" ] || fail "BETA-LEAD (no owner) must reach lead, got: $(scopes_of_log BETA-LEAD)"
python3 - "$ROOT/shared/scripts/lib" "$S" <<'PY' || fail "path derivation limits"
import json, sys
from pathlib import Path
sys.path.insert(0, sys.argv[1])
import learning_write as lw
repo = Path(sys.argv[2]) / "repo"
rows = [{"type": "assistant", "message": {"content": [{"type": "tool_use", "name": "MultiEdit" if i % 2 else "Edit", "input": {"file_path": f"{repo}/src/ui/f{i}.ts"}}]}} for i in range(25)]
transcript = Path(sys.argv[2]) / "many.jsonl"
transcript.write_text("\n".join(json.dumps(r) for r in rows) + "\n")
got = lw.transcript_written_paths(str(transcript), repo)
assert len(got) == 20 and got[0] == "src/ui/f0.ts" and got[-1] == "src/ui/f19.ts", got        # capped at 20, repo-relative
assert lw.transcript_written_paths(str(Path(sys.argv[2]) / "missing.jsonl"), repo) == []
assert lw.split_lesson_paths("keep the budget paths: a/b.ts, c/d.ts", repo) == ("keep the budget", ["a/b.ts", "c/d.ts"])
assert lw.split_lesson_paths("no suffix here", repo) == ("no suffix here", [])
assert lw.split_lesson_paths("escape (paths: ../../etc/passwd /abs/elsewhere ok/x.ts)", repo) == ("escape", ["ok/x.ts"])
PY
ok "cycle 1 through the generated subagentstop entry (silent, rc 0): BETA-PRIV (suffix paths, only api-dev owns them) stays private, BETA-ROUTED (paths from the transcript Write) is addressed to ui-dev, BETA-LEAD (suffix paths, no owner) to lead; paths are repo-relative and capped at 20"

r1="$(write_as "$REPO2" r1 --text "ROUTE-FIVE five roles own the shared path." --paths pkg/shared/x.ts)" || fail "route five"
r2="$(write_as "$REPO2" r1 --text "ROUTE-PAIR two other roles own the pair path." --paths pkg/pair/x.ts)" || fail "route pair"
r3="$(write_as "$REPO2" r1 --text "ROUTE-NONE nobody owns the docs path." --paths docs/z.md)" || fail "route none"
r4="$(write_as "$REPO2" r1 --text "ROUTE-FREE this lesson carries no paths.")" || fail "route no paths"
r5="$(write_as "$REPO2" r1 --text "ROUTE-OWN only the author owns the own path." --paths pkg/own1/x.ts)" || fail "route own"
r6="$(write_as "$REPO2" r1 --text "ROUTE-EXPLICIT explicit audience plus routing." --paths pkg/pair/y.ts --audience role:r5)" || fail "route explicit"
[ "$(scopes_of "$r1")" = "tlm-route/@lead tlm-route/r1" ] || fail "4 other owners must collapse to lead, got: $(scopes_of "$r1")"
[ "$(scopes_of "$r2")" = "tlm-route/r1 tlm-route/r2 tlm-route/r3" ] || fail "pair routing, got: $(scopes_of "$r2")"
[ "$(scopes_of "$r3")" = "tlm-route/@lead tlm-route/r1" ] || fail "no owner must go to lead, got: $(scopes_of "$r3")"
[ "$(scopes_of "$r4")" = "tlm-route/r1" ] || fail "no paths must not be routed, got: $(scopes_of "$r4")"
[ "$(scopes_of "$r5")" = "tlm-route/r1" ] || fail "author-only paths must stay private, got: $(scopes_of "$r5")"
[ "$(scopes_of "$r6")" = "tlm-route/r1 tlm-route/r2 tlm-route/r3 tlm-route/r5" ] || fail "explicit audience is kept alongside routing, got: $(scopes_of "$r6")"
ok "routing rules: >3 owners -> lead only, 2 owners -> 2 copies, no owner -> lead, no paths and author-only -> none, explicit audience kept"

# --- 4. digest ---------------------------------------------------------------------
DIGEST="$PROMETHEUS_TEAM_DIGEST_DIR/$PROJECT/tlm-fixture.jsonl"
python3 - "$DIGEST" "$T_PRIV" "$T_ROUTED" "$T_LEAD" <<'PY' || fail "digest file"
import hashlib, json, sys
path, *texts = sys.argv[1:]
lines = open(path).read().splitlines()
assert len(lines) == 3, lines
records = [json.loads(l) for l in lines]
for line, record, text in zip(lines, records, texts):
    assert len(line) <= 160, (len(line), line)
    assert record["by"] == "api-dev" and record["h"] == hashlib.sha256(" ".join(text.split()).encode()).hexdigest(), record
    assert "BETA-" not in line, line  # the digest never carries lesson text
assert [r["p"] for r in records] == ["src/api/handler.ts", "src/ui/client.ts", "docs/readme.md"], records
PY
dup="$(write_as "$REPO" prometheus-skill-pack:api-dev --text "$T_PRIV" --paths src/api/handler.ts --stage execute)" || fail "repeat write"
[ "$(wc -l < "$DIGEST" | tr -d ' ')" = 3 ] || fail "a repeated lesson appended to the digest"
python3 -c 'import json,sys;assert json.loads(sys.argv[1])["written"]==0' "$dup" || fail "a repeated lesson was written twice"
python3 - "$ROOT/shared/scripts/lib" "$S/rotate" <<'PY' || fail "digest rotation"
import os, sys
sys.path.insert(0, sys.argv[1])
import learning_route as lr
from pathlib import Path
path = Path(sys.argv[2]) / "p" / "t.jsonl"
for i in range(1005):
    assert lr.append_digest(path, lr.digest_line("api-dev", [f"src/api/file-{i}.ts"] * 3, "a" * 64, now=1000 + i))
lines = path.read_text().splitlines()
assert len(lines) == 1000, len(lines)
assert '"t":2004' in lines[-1] and '"t":1005' in lines[0], (lines[0], lines[-1])
assert all(len(l) <= 160 for l in lines)
assert sorted(os.listdir(path.parent)) == [".t.jsonl.lock", "t.jsonl"], os.listdir(path.parent)
PY
ok "digest: <= 160 chars, author/paths/contentHash and no lesson text, a repeat adds nothing, rotation keeps 1,000 lines atomically"

# --- 5. worker delivery and store validation ----------------------------------------
for _ in $(seq 1 30); do
  "$WORKER_BIN" --memory-url "$SURREAL_MEMORY_URL" run-once >/dev/null 2>&1
  [ -z "$(ls "$PROMETHEUS_LEARNING_QUEUE"/memory/pending "$PROMETHEUS_LEARNING_QUEUE"/memory/submitting "$PROMETHEUS_LEARNING_QUEUE"/memory/accepted 2>/dev/null | grep json)" ] && break
  sleep 2
done
python3 - "$SURREAL_MEMORY_URL" "$SCHEMA" <<'PY' || fail "stored records"
import json, sys, urllib.parse, urllib.request
import jsonschema
base, schema_path = sys.argv[1:]
validator = jsonschema.Draft202012Validator(json.load(open(schema_path)))
TRAILER = "\n\n<!-- prometheus-envelope "
def get(uid, aid):
    q = urllib.parse.urlencode({"user_id": uid, "agent_id": aid})
    data = json.loads(urllib.request.urlopen(f"{base}/api/v1/memory?{q}", timeout=10).read())
    return data if isinstance(data, list) else data.get("memories", data.get("results", []))
def expected_agent(envelope, categories):
    team = envelope.get("teamId")
    aud = next((c[4:] for c in categories if c.startswith("aud:")), None)
    if aud:
        return f"{team}/@lead" if aud == "lead" else f"{team}/{aud.split(':', 1)[1]}"
    vis = envelope["visibility"]
    return {"agent": f"{team}/{envelope.get('roleId')}", "team": f"{team}/@team", "lead": f"{team}/@lead"}.get(vis, "@project")
total, seen = 0, {}
for project, team, roles in (("project:b7-fixture", "tlm-fixture", ["api-dev", "ui-dev"]), ("project:b7-route", "tlm-route", ["r1", "r2", "r3", "r4", "r5"])):
    for agent in [f"{team}/{r}" for r in roles] + [f"{team}/@lead", f"{team}/@team"]:
        for record in get(project, agent):
            total += 1
            assert record.get("agent_id") == agent and record.get("user_id") == project, (agent, record.get("agent_id"), record.get("user_id"))
            content = record["content"]
            assert TRAILER in content, content[:120]
            envelope = json.loads(content.rsplit(TRAILER, 1)[1].rsplit(" -->", 1)[0])
            errors = [e.message for e in validator.iter_errors(envelope)]
            assert not errors, (agent, errors)
            want = expected_agent(envelope, record.get("categories") or [])
            assert want == agent, f"design-table agent_id {want} != stored {agent} for {content[:60]!r}"
            seen.setdefault((project, agent), []).append(content.split(TRAILER)[0])
assert total >= 20, total
fx = lambda a: seen.get(("project:b7-fixture", a), [])
assert any("BETA-PRIV" in c for c in fx("tlm-fixture/api-dev")) and not any("BETA-PRIV" in c for k, v in seen.items() if k != ("project:b7-fixture", "tlm-fixture/api-dev") for c in v)
assert any("BETA-ROUTED" in c for c in fx("tlm-fixture/ui-dev")) and not any("BETA-PRIV" in c for c in fx("tlm-fixture/ui-dev"))
assert any("BETA-LEAD" in c for c in fx("tlm-fixture/@lead"))
digest = fx("tlm-fixture/@team")
assert len(digest) == 3 and not any("BETA-" in c for c in digest), digest
assert any("src/api/handler.ts" in c for c in digest), digest
print(f"  validated {total} stored records against the envelope schema")
PY
ok "every stored record validates against the envelope schema with the design-table agent_id; BETA-PRIV exists only under api-dev; the digest mirror (3 lines) holds no lesson text"

# --- 6. the generated entry, run directly (no model) -----------------------------------
payload() { printf '{"hook_event_name":"SubagentStart","agent_type":"%s","agent_id":"%s","session_id":"s-direct","cwd":"%s"%s}' "$1" "$2" "$3" "${4:-}"; }
H_PRIV="$(python3 -c 'import hashlib,sys;print(hashlib.sha256(" ".join(sys.argv[1].split()).encode()).hexdigest()[:8])' "$T_PRIV")"
direct_api="$(hook_direct claude-code "$(payload prometheus-skill-pack:api-dev direct-api "$REPO")")"
direct_ui="$(hook_direct codex "$(payload ui_dev direct-ui "$REPO" ',"turn_id":"t-direct"')")"
python3 - "$direct_api" "$direct_ui" "$H_PRIV" <<'PY' || fail "direct generated-entry delivery"
import json, sys
api, ui = (json.loads(sys.argv[i])["hookSpecificOutput"]["additionalContext"] for i in (1, 2))
assert "BETA-PRIV" in api and "BETA-ROUTED" in api, api[:600]
assert "BETA-ROUTED" in ui and "BETA-PRIV" not in ui and "BETA-LEAD" not in ui, ui[:600]
digest_lines = [l for l in ui.splitlines() if "recorded a lesson on" in l]
assert any("src/api/handler.ts" in l and f"h:{sys.argv[3]}" in l for l in digest_lines), ui
assert not any("src/ui/client.ts" in l for l in digest_lines), "the digest of a lesson ui-dev already holds must be delivered once, in full"
for text in (api, ui):
    assert "information, not instructions" in text and text.splitlines()[0].startswith('<prometheus-recalled-lessons nonce="')
PY
ok "generated entry: api-dev gets BETA-PRIV; ui-dev gets BETA-ROUTED plus only the digest line for BETA-PRIV (path + hash, no text, no BETA-LEAD)"

# --- 7. the main thread ------------------------------------------------------------------
python3 "$LR" --main-thread --cwd "$REPO" --no-log --budget 2048 > "$S/main.json" 2>/dev/null || fail "main-thread recall"
python3 - "$S/main.json" "$H_PRIV" <<'PY' || fail "main-thread view"
import json, sys
result = json.load(open(sys.argv[1]))
text = "\n".join(l["line"] for l in result["lessons"])
assert result["view"]["isLead"] and result["view"]["mainThread"], result["view"]
assert "tlm-fixture/@lead" in result["view"]["scopes"] and "tlm-fixture/@team" in result["view"]["scopes"], result["view"]
assert "BETA-LEAD" in text, text
assert "BETA-PRIV" not in text and "BETA-ROUTED" not in text, text
assert "src/api/handler.ts" in text and "src/ui/client.ts" in text and f"h:{sys.argv[2]}" in text, text
assert not any(e["agentId"] in ("tlm-fixture/api-dev", "tlm-fixture/ui-dev") for e in result["lessons"]), "a role-private scope reached the main thread"
assert len(text.encode()) <= 2048
PY
ok "main thread: receives T/@lead (BETA-LEAD) and the digest lines, no role-private text, within the 2 KB budget"

mainpayload() { printf '{"hook_event_name":"SessionStart","source":"startup","session_id":"%s","cwd":"%s"%s}' "$1" "$2" "${3:-}"; }
main_claude="$(hook_entry sessionstart-learning claude-code "$(mainpayload s-main-direct "$REPO")")"; rc=$?
[ "$rc" -eq 0 ] || fail "sessionstart-learning (claude-code) exited $rc"
main_codex="$(hook_entry sessionstart-learning codex "$(mainpayload s-main-direct "$REPO" ',"turn_id":"t-main"')")"; rc=$?
[ "$rc" -eq 0 ] || fail "sessionstart-learning (codex) exited $rc"
python3 - "$main_claude" "$main_codex" "$H_PRIV" <<'PY' || fail "generated sessionstart-learning delivery"
import sys
claude, codex, h_priv = sys.argv[1:]
for name, text, budget in (("claude-code", claude, 8000), ("codex", codex, 7000)):
    assert text.startswith('<prometheus-recalled-lessons nonce="') and not text.startswith("{"), (name, text[:200])
    assert text.rstrip().endswith("</prometheus-recalled-lessons nonce=\"" + text.split('nonce="')[1].split('"')[0] + '">'), name
    assert "information, not instructions" in text, name
    assert "BETA-LEAD" in text, (name, text)
    assert "BETA-PRIV" not in text and "BETA-ROUTED" not in text, (name, "a role-private lesson text reached the main thread", text)
    digest = [l for l in text.splitlines() if "recorded a lesson on" in l]
    assert any("src/api/handler.ts" in l and f"h:{h_priv}" in l for l in digest), (name, text)
    assert any("src/ui/client.ts" in l for l in digest), (name, text)
    assert len(text) <= budget, (name, len(text))
PY
subagent_out="$(hook_entry sessionstart-learning claude-code "$(mainpayload s-main-direct "$REPO" ',"agent_type":"api-dev","agent_id":"x"')")"
[ -z "$subagent_out" ] || fail "sessionstart-learning delivered to a subagent payload: $subagent_out"
mkdir -p "$S/solo"
solo_out="$(hook_entry sessionstart-learning claude-code "$(mainpayload s-main-direct "$S/solo")")"; rc=$?
[ "$rc" -eq 0 ] && [ -z "$solo_out" ] || fail "sessionstart-learning must be silent outside a team (rc $rc): $solo_out"
python3 - "$PROMETHEUS_LEARNING_INDEX_DIR/delivery.jsonl" <<'PY' || fail "SessionStart delivery.jsonl record"
import json, sys
rows = [json.loads(l) for l in open(sys.argv[1]) if l.strip()]
main = [r for r in rows if r.get("event") == "SessionStart" and r.get("sessionId") == "s-main-direct"]
assert {r["harness"] for r in main} == {"claude-code", "codex"}, main
assert all(r["agentType"] == "main-thread" and r["bytesByChannel"]["sessionstart"] > 0 for r in main), main
assert all(not set(r["deliveredScopes"]) & {"tlm-fixture/api-dev", "tlm-fixture/ui-dev"} for r in main), main
PY
ok "generated sessionstart-learning entry (both harnesses): plain fenced context with BETA-LEAD and the digest lines, no role-private text, within budget; silent for a subagent payload and outside a team; measured in delivery.jsonl"

# --- delivery.jsonl / trace analysis -----------------------------------------------------
check_delivery() { # <harness>
  python3 - "$PROMETHEUS_LEARNING_INDEX_DIR/delivery.jsonl" "$1" "$PROMETHEUS_LEARNING_DELIVERY_TRACE_DIR" "$H_PRIV" <<'PY'
import glob, json, os, sys
path, harness, trace_dir, h_priv = sys.argv[1:]
records = [json.loads(l) for l in open(path) if l.strip()]
timeouts = sum(1 for r in records if r.get("timedOut") and r.get("harness") == harness)
records = [r for r in records if r.get("event") == "SubagentStart" and r.get("harness") == harness
           and not r.get("timedOut") and not str(r.get("agentId") or "").startswith("direct-")]
if timeouts:
    print(f"  {harness}: {timeouts} SubagentStart recall(s) hit the hook watchdog")
allowed = lambda role: {f"tlm-fixture/{role}", "tlm-fixture/@team", "@project", "@user", "@global"}
assert {"api-dev", "ui-dev"} <= {r["roleId"] for r in records}, f"{harness}: no SubagentStart records for both roles"
leaks = sum(len(set(r["deliveredScopes"]) - allowed(r["roleId"])) for r in records)
for trace in glob.glob(os.path.join(trace_dir, f"{harness}-*.txt")):
    name, text = os.path.basename(trace), open(trace).read()
    if "-ui-dev-" in name:
        if "BETA-PRIV" in text or "BETA-LEAD" in text: leaks += 1
        assert "src/api/handler.ts" in text and f"h:{h_priv}" in text, f"{name}: the digest line for BETA-PRIV is missing"
assert leaks == 0, f"{harness}: {leaks} leaks"
for r in records:
    print(f"  {harness} {r['roleId']}: {r['chars']} chars, {sum(r['bytesByChannel'].values())} B, scopes {r['deliveredScopes']}")
PY
}

check_main_thread() { # <harness>: the live main thread was handed the team view, nothing private
  python3 - "$PROMETHEUS_LEARNING_INDEX_DIR/delivery.jsonl" "$1" "$PROMETHEUS_LEARNING_DELIVERY_TRACE_DIR" <<'PY'
import glob, json, os, sys
path, harness, trace_dir = sys.argv[1:]
live = [json.loads(l) for l in open(path) if l.strip()]
live = [r for r in live if r.get("event") == "SessionStart" and r.get("harness") == harness
        and not r.get("timedOut") and not str(r.get("sessionId") or "").startswith("s-main-")]
assert live, f"{harness}: no live SessionStart delivery record (the hook did not fire or delivered nothing)"
traces = [t for t in glob.glob(os.path.join(trace_dir, f"{harness}-main-thread-*.txt")) if "s-main-" not in os.path.basename(t)]
assert traces, f"{harness}: no live main-thread trace"
for trace in traces:
    text = open(trace).read()
    assert "BETA-LEAD" in text and "BETA-PRIV" not in text and "BETA-ROUTED" not in text, (trace, text[:400])
print(f"  {harness} main thread: {live[-1]['chars']} chars, scopes {live[-1]['deliveredScopes']}")
PY
}

# --- 8. Claude Code ------------------------------------------------------------------------
if want claude; then
  CPKG="$S/claude-pkg"
  cp -R "$CLAUDE_PKG" "$CPKG"
  python3 - "$CLAUDE_PKG/hooks/hooks.json" "$CPKG/hooks/hooks.json" <<'PY'
import json, sys
hooks = json.load(open(sys.argv[1]))["hooks"]
learning = [g for g in hooks["SessionStart"] if '"sessionstart-learning"' in json.dumps(g)]
assert learning, "generated claude hooks lack the sessionstart-learning group"
json.dump({"hooks": {"SubagentStart": hooks["SubagentStart"], "SessionStart": learning}}, open(sys.argv[2], "w"), indent=2)
PY
  AGENTS="$(python3 - "$PROBE_INSTRUCTIONS" <<'PY'
import json, sys
print(json.dumps({r: {"description": f"Fixture role {r}. Use only when explicitly asked.", "prompt": sys.argv[1], "model": "haiku"}
                  for r in ("api-dev", "ui-dev")}))
PY
)"
  PROMPT='Use the Task tool twice, one after the other: first with subagent_type "api-dev", then with subagent_type "ui-dev", each with the prompt "Report your BETA tokens." Do not answer for them. After both return, print exactly two lines and nothing else: api-dev=<its reply verbatim> and ui-dev=<its reply verbatim>.'
  ( cd "$REPO" && HOME="$REAL_HOME" run_timeout "$MODEL_TIMEOUT" claude -p "$PROMPT" --setting-sources project,local \
      --strict-mcp-config --no-session-persistence --plugin-dir "$CPKG" --agents "$AGENTS" --model sonnet ) \
      > "$S/claude.out" 2> "$S/claude.err" < /dev/null
  echo "  claude output: $(tr '\n' ' ' < "$S/claude.out")"
  python3 - "$S/claude.out" <<'PY' || { cat "$S/claude.err" >&2; fail "Claude Code subagents did not report the right tokens"; }
import re, sys
text = open(sys.argv[1]).read()
api = re.search(r"api-dev\s*[=:]\s*(.*)", text); ui = re.search(r"ui-dev\s*[=:]\s*(.*)", text)
assert api and ui, text
assert "BETA-PRIV" in api.group(1), api.group(1)
assert "BETA-ROUTED" in ui.group(1) and "BETA-PRIV" not in ui.group(1) and "BETA-LEAD" not in ui.group(1), ui.group(1)
PY
  check_delivery claude-code || fail "Claude Code delivery.jsonl budgets/leaks/digest"
  check_main_thread claude-code || fail "Claude Code main-thread SessionStart delivery"
  ok "Claude Code: api-dev reported BETA-PRIV; ui-dev reported BETA-ROUTED and not BETA-PRIV, and was handed the digest line for BETA-PRIV; 0 leaks"
fi

# --- 9. Codex ------------------------------------------------------------------------------
if want codex; then
  MKT="$S/mkt"; mkdir -p "$MKT/.claude-plugin" "$MKT/plugins"
  cp -R "$CODEX_PKG" "$MKT/plugins/prometheus-skill-pack"
  printf '{"mcpServers":{}}\n' > "$MKT/plugins/prometheus-skill-pack/.mcp.json"   # hooks under test, no MCP servers
  printf '{"name":"b7-fixture","owner":{"name":"fixture"},"plugins":[{"name":"prometheus-skill-pack","source":"./plugins/prometheus-skill-pack","description":"generated Codex package under test"}]}\n' \
    > "$MKT/.claude-plugin/marketplace.json"
  ln -s "$REAL_HOME/.codex/auth.json" "$CODEX_HOME/auth.json"
  cat > "$CODEX_HOME/config.toml" <<EOF
[features]
hooks = true
multi_agent = true
[memories]
generate_memories = false
[projects."$REPO"]
trust_level = "trusted"
EOF
  codex plugin marketplace add "$MKT" > "$S/codex-mkt.log" 2>&1 || { cat "$S/codex-mkt.log" >&2; fail "codex plugin marketplace add"; }
  codex plugin add prometheus-skill-pack@b7-fixture > "$S/codex-add.log" 2>&1 || { cat "$S/codex-add.log" >&2; fail "codex plugin add"; }
  PROMPT='Spawn the custom agent api_dev with the task "Report your BETA tokens." and wait for its reply. Then spawn the custom agent ui_dev with the same task and wait for its reply. Then print exactly two lines and nothing else: api_dev=<its reply verbatim> and ui_dev=<its reply verbatim>.'
  ( cd "$REPO" && run_timeout "$MODEL_TIMEOUT" codex exec --dangerously-bypass-hook-trust --skip-git-repo-check "$PROMPT" ) \
      > "$S/codex.out" 2> "$S/codex.err" < /dev/null
  echo "  codex output: $(tr '\n' ' ' < "$S/codex.out")"
  python3 - "$S/codex.out" "$CODEX_HOME/sessions" "$H_PRIV" <<'PY' || { tail -40 "$S/codex.err" >&2; grep timedOut "$PROMETHEUS_LEARNING_INDEX_DIR/delivery.jsonl" >&2; fail "Codex subagents / child rollouts"; }
import glob, json, re, sys
text, sessions, h_priv = open(sys.argv[1]).read(), sys.argv[2], sys.argv[3]
api = re.search(r"api_dev\s*[=:]\s*(.*)", text); ui = re.search(r"ui_dev\s*[=:]\s*(.*)", text)
assert api and ui, text
assert "BETA-PRIV" in api.group(1), api.group(1)
assert "BETA-ROUTED" in ui.group(1) and "BETA-PRIV" not in ui.group(1), ui.group(1)
# Codex forks the parent thread's history into a spawned child, so a child may legitimately
# see the main-thread view the parent was handed at SessionStart (its BETA-LEAD line). That
# is allowed only when it is the parent's own block, inherited verbatim; the child's OWN
# SubagentStart block (checked below) must still never carry BETA-LEAD or BETA-PRIV.
roles, inherited, parent_bodies = {}, {}, []
for path in glob.glob(f"{sessions}/**/rollout-*.jsonl", recursive=True):
    lines = [json.loads(l) for l in open(path) if l.strip()]
    meta = lines[0]["payload"] if lines and lines[0].get("type") == "session_meta" else {}
    bodies = []
    for l in lines:
        p = l.get("payload") or {}
        if l.get("type") == "response_item" and p.get("type") == "message" and p.get("role") == "developer":
            body = "".join(c.get("text", "") for c in p.get("content") or [] if isinstance(c, dict))
            if "prometheus-recalled-lessons" in body:
                bodies.append(body)
    if meta.get("agent_role"):
        roles[meta["agent_role"]] = "\n".join(b for b in bodies if "Recalled team view" not in b)
        inherited[meta["agent_role"]] = [b for b in bodies if "Recalled team view" in b]
    else:  # the parent thread: the main-thread team view (SessionStart), never a role's or a subagent's text
        parent_bodies.extend(bodies)
        assert not any("Recalled lessons for" in b or "BETA-PRIV" in b or "BETA-ROUTED" in b for b in bodies), "a parent thread received a role-scoped injection"
assert any("Recalled team view" in b and "BETA-LEAD" in b for b in parent_bodies), "the Codex parent thread did not receive the main-thread team view"
assert "BETA-PRIV" in roles.get("api_dev", ""), roles.keys()
ui_text = roles.get("ui_dev", "")
assert "BETA-ROUTED" in ui_text and "BETA-PRIV" not in ui_text and "BETA-LEAD" not in ui_text, ui_text[:500]
assert "src/api/handler.ts" in ui_text and f"h:{h_priv}" in ui_text, "ui_dev child rollout lacks the BETA-PRIV digest line"
for role, blocks in inherited.items():  # only the parent's own main-thread block, verbatim, and never private text
    for block in blocks:
        assert block in parent_bodies, f"{role}: an inherited team view that is not the parent's"
        assert "BETA-PRIV" not in block and "BETA-ROUTED" not in block, f"{role}: private text inside the inherited team view"
if "BETA-LEAD" in ui.group(1):
    assert inherited.get("ui_dev"), "ui_dev reported BETA-LEAD but its own context has no inherited team view"
print("  codex: ui_dev inherited the parent's main-thread view:", bool(inherited.get("ui_dev")))
PY
  check_delivery codex > "$S/codex-delivery.txt" || { cat "$S/codex-delivery.txt"; fail "Codex delivery.jsonl budgets/leaks/digest"; }
  cat "$S/codex-delivery.txt"
  check_main_thread codex || fail "Codex main-thread SessionStart delivery"
  ok "Codex: api_dev reported BETA-PRIV; ui_dev reported BETA-ROUTED and not BETA-PRIV (BETA-LEAD only via the parent's forked main-thread view; its own SubagentStart block has neither), its child rollout carries the BETA-PRIV digest line; the parent got the team view; 0 leaks"
fi

# --- 10. the delivery report ---------------------------------------------------------------
python3 "$REPORT" --index-dir "$PROMETHEUS_LEARNING_INDEX_DIR" --claude-memory "$S/absent-MEMORY.md" --codex-memory "$S/absent-summary.md" \
  --baseline-claude 14336 --baseline-codex 11059 --require-reduction > "$S/report.txt" || { cat "$S/report.txt"; fail "report-learning-delivery --require-reduction on a small delivery"; }
grep -q 'subagentstart' "$S/report.txt" && grep -q 'file-tier' "$S/report.txt" && grep -q 'total' "$S/report.txt" || { cat "$S/report.txt"; fail "report lacks per-channel bytes"; }
python3 "$REPORT" --index-dir "$PROMETHEUS_LEARNING_INDEX_DIR" --claude-memory "$S/absent-MEMORY.md" --codex-memory "$S/absent-summary.md" \
  --baseline-claude 10 --baseline-codex 10 --require-reduction >/dev/null 2>&1 && fail "report accepted a total above the baseline"
head -c 20000 /dev/zero | tr '\0' 'x' > "$S/big-MEMORY.md"
python3 "$REPORT" --index-dir "$PROMETHEUS_LEARNING_INDEX_DIR" --claude-memory "$S/big-MEMORY.md" --codex-memory "$S/absent-summary.md" \
  --require-reduction >/dev/null 2>&1 && fail "report accepted a 20 KB MEMORY.md"
sed 's/^/  /' "$S/report.txt" | head -24
ok "report-learning-delivery: bytes per agent per channel; --require-reduction passes under the baseline, fails over it and with an oversized MEMORY.md"

# --- 11. surreal-memory stopped --------------------------------------------------------------
kill "$srv" 2>/dev/null; wait "$srv" 2>/dev/null; srv=""
curl -fsS -m 1 "$SURREAL_MEMORY_URL/health" >/dev/null 2>&1 && fail "scratch surreal-memory still answering after stop"
out="$(hook_direct claude-code "$(payload ui-dev stopped-ui "$REPO")")"; rc=$?
[ "$rc" -eq 0 ] || fail "stopped store: hook exited $rc"
python3 - "$out" "$H_PRIV" <<'PY' || fail "stopped-store ui-dev delivery"
import json, sys
text = json.loads(sys.argv[1])["hookSpecificOutput"]["additionalContext"]
assert "BETA-ROUTED" in text and "BETA-PRIV" not in text and "BETA-LEAD" not in text, text[:500]
assert "src/api/handler.ts" in text and f"h:{sys.argv[2]}" in text, text
PY
python3 "$LR" --main-thread --cwd "$REPO" --no-log --memory-url none --budget 2048 > "$S/main-down.json" 2>/dev/null || fail "main-thread recall (store down)"
python3 - "$S/main-down.json" <<'PY' || fail "stopped-store main-thread view"
import json, sys
text = "\n".join(l["line"] for l in json.load(open(sys.argv[1]))["lessons"])
assert "BETA-LEAD" in text and "src/api/handler.ts" in text and "src/ui/client.ts" in text, text
assert "BETA-PRIV" not in text and "BETA-ROUTED" not in text, text
PY
main_down_hook="$(hook_entry sessionstart-learning claude-code "$(mainpayload s-main-down "$REPO")")"; rc=$?
[ "$rc" -eq 0 ] || fail "stopped store: sessionstart-learning exited $rc"
python3 - "$main_down_hook" <<'PY' || fail "stopped-store sessionstart-learning delivery"
import sys
text = sys.argv[1]
assert "BETA-LEAD" in text and "src/api/handler.ts" in text and "src/ui/client.ts" in text, text
assert "BETA-PRIV" not in text and "BETA-ROUTED" not in text, text
PY
ok "surreal-memory stopped: the digest file and the file tier still reach ui-dev and the main thread (also through the sessionstart hook); role-private text does not"

echo "test-team-awareness: $pass passed (harness: $HARNESS)"
