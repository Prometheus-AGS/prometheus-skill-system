#!/bin/bash
# test-recall-quality.sh — recall-quality gate for learning_recall.py (change tlh-08).
#
# Real processes, scratch everything: a scratch surreal-memory-server (change 04
# library), a scratch HOME holding the shared/global pk KBs, a scratch repo whose
# own .prometheus/knowledge is the project pk KB, and the real `pk` and
# learning_recall.py CLIs. The fixture is fixtures/recall-quality/fixture.json:
# project lessons, current-project pk entries, foreign-project untagged pk
# entries (shared/global scopes, some with only topical tags) and global feedback.
#
# Six fixed phase-goal queries: 3 tuning, 3 held-out. The similarity threshold
# (PK_UNTAGGED_MIN_SIMILARITY) is derived from the TUNING queries only: the
# smallest 0.01 step above every foreign entry's tuning similarity. The assertions
# run on the HELD-OUT queries: >= 3 of the top 5 recalled items are current-project
# or role scope, no foreign-project entry is recalled, and the relevant
# current-project untagged pk entry is recalled. The per-query table is printed.
#
# Exit 0 pass, 1 fail, 2 BLOCKED (a prerequisite is missing). bash 3.2.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
LR="$ROOT/shared/scripts/lib/learning_recall.py"
FIXTURE="$HERE/fixtures/recall-quality/fixture.json"
. "$HERE/lib/scratch-surreal.sh"   # before any HOME override (locates the real model cache)
PK_BIN="${TLI_PK_BIN:-$(command -v pk || true)}"
PORT="$(scratch_surreal_pick_port)"; SCRATCH_SURREAL_PORT="$PORT"

blocked() { echo "BLOCKED: $*" >&2; exit 2; }
[ -x "$PK_BIN" ] || blocked "pk not found (set TLI_PK_BIN or put pk >= 1.10 on PATH)"
"$PK_BIN" --version 2>/dev/null | grep -Eq '1\.(1[0-9]|[2-9][0-9])\.' || blocked "pk < 1.10.0"
for tool in python3 git curl; do command -v "$tool" >/dev/null 2>&1 || blocked "$tool missing"; done
[ -f "$FIXTURE" ] || blocked "fixture missing: $FIXTURE"

S="$(mktemp -d)"
cleanup() { scratch_surreal_stop >/dev/null 2>&1; wait 2>/dev/null; if [ -n "${TLI_KEEP:-}" ]; then echo "kept $S" >&2; else rm -rf "$S"; fi; }
trap cleanup EXIT
export HOME="$S/home" CODEX_HOME="$S/codex" PROMETHEUS_PLUGIN_ROOT="$S/plugin-root"
export PROMETHEUS_LEARNING_QUEUE="$S/queue" PROMETHEUS_LEARNING_LOG_DIR="$S/log" PROMETHEUS_LEARNING_INDEX_DIR="$S/index"
export PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=1
export SURREAL_MEMORY_URL="http://127.0.0.1:$PORT"
export PATH="$(dirname "$PK_BIN"):$PATH"
unset PROMETHEUS_LEARNING_PK CLAUDE_PLUGIN_ROOT PLUGIN_ROOT KBD_PACK_ROOT KBD_ORCHESTRATOR_ROOT UAR_MEMORY_MCP_URL PK_KB_DIR
mkdir -p "$HOME" "$CODEX_HOME" "$PROMETHEUS_PLUGIN_ROOT"
git config --global user.email fixture@example.invalid; git config --global user.name Fixture

fail() { echo "FAIL: $*" >&2; exit 1; }
REPO="$S/repo"

# --- scratch project, scratch pk KBs, scratch surreal-memory --------------------
mkdir -p "$REPO/.agent-team/tlm-fixture" "$REPO/.prometheus"
git -C "$REPO" init -q
cat > "$REPO/.agent-team/tlm-fixture/team.json" <<'JSON'
{"schemaVersion":1,"id":"tlm-fixture","roles":[
 {"id":"lead","lead":true,"description":"lead","prompt":"p","skills":[],"owns":[],"inputs":[],"outputs":[],"dependsOn":[]},
 {"id":"api-dev","description":"api","prompt":"p","skills":[],"owns":["src/api/**"],"inputs":[],"outputs":[],"dependsOn":[]}]}
JSON
python3 - "$FIXTURE" "$REPO" "$HOME" <<'PY' || fail "could not seed the scratch pk KBs"
import json, os, sys
fixture, repo, home = json.load(open(sys.argv[1])), sys.argv[2], sys.argv[3]
open(os.path.join(repo, ".prometheus", "project.json"), "w").write(json.dumps({"projectId": fixture["projectId"]}) + "\n")
dirs = {"project": os.path.join(repo, ".prometheus", "knowledge", "wiki"),
        "shared": os.path.join(home, ".prometheus", "knowledge", "shared", "wiki"),
        "global": os.path.join(home, ".prometheus", "knowledge", "wiki")}
for d in dirs.values():
    os.makedirs(d, exist_ok=True)
def write(scope, entry):
    lines = ["---", "type: Reference", f"id: {entry['id']}", f"title: {entry['title']}"]
    if entry["tags"]:
        lines += ["tags:"] + [f"- {t}" for t in entry["tags"]]
    lines += ["links: []", "sources:", "- stdin", "timestamp: 2026-09-01T00:00:00+00:00",
              "created_at: 2026-09-01T00:00:00+00:00", "updated_at: 2026-09-01T00:00:00+00:00", "revision: 0", "---", "", entry["body"], ""]
    open(os.path.join(dirs[scope], entry["id"] + ".md"), "w").write("\n".join(lines))
for entry in fixture["projectPk"]:
    write("project", entry)
for entry in fixture["foreignPk"]:
    write(entry["scope"], entry)
PY
( cd "$REPO" && pk snapshot ) >"$S/snapshot.log" 2>&1 || { cat "$S/snapshot.log" >&2; fail "pk snapshot of the scratch KBs failed"; }

scratch_surreal_start "$S/sm" rq
python3 - "$FIXTURE" "$SURREAL_MEMORY_URL" <<'PY' || fail "could not seed the scratch surreal-memory"
import json, sys, urllib.request
fixture, base = json.load(open(sys.argv[1])), sys.argv[2]
opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
for lesson in fixture["surrealLessons"]:
    body = {"content": lesson["content"], "user_id": lesson.get("user_id", fixture["projectId"]), "agent_id": lesson["agent_id"]}
    request = urllib.request.Request(base + "/api/v1/memory", data=json.dumps(body).encode(), method="POST",
                                     headers={"Content-Type": "application/json"})
    opener.open(request, timeout=60).read()
PY

# --- derive the threshold on the tuning queries, assert on the held-out ones -----
python3 - "$FIXTURE" "$REPO" "$LR" <<'PY'
import json, math, subprocess, sys
from pathlib import Path
fixture, repo, lr = json.load(open(sys.argv[1])), sys.argv[2], sys.argv[3]
sys.path.insert(0, str(Path(lr).parent))
import learning_recall as L

foreign = {e["id"]: e for e in fixture["foreignPk"]}
markers = [m.lower() for m in fixture["foreignMarkers"]]
role = fixture["role"]
problems = []

def pk_returned(query):
    out = subprocess.run(["pk", "context", "--format", "json", "--limit", "8", query], cwd=repo,
                         capture_output=True, text=True, timeout=30, check=True).stdout
    return [r for r in json.loads(out)["results"]]

def similarity(entry):
    return L.lexical_similarity(QUERY, f"{entry['title']} {L.wiki_excerpt(entry['body'])}")

# 1. tuning: smallest 0.01 step strictly above every returned foreign entry's similarity
worst, seen_foreign = 0.0, 0
for q in fixture["queries"]["tuning"]:
    QUERY = q["text"]
    for r in pk_returned(q["text"]):
        if r["id"] in foreign:
            seen_foreign += 1
            worst = max(worst, similarity(foreign[r["id"]]))
if not seen_foreign:
    problems.append("no foreign entry reached pk for any tuning query: the fixture cannot tune a threshold")
tuned = math.floor(worst * 100 + 1e-9) / 100 + 0.01
tuned = round(tuned, 2)
print(f"tuning: max foreign similarity {worst:.3f} over {seen_foreign} pk hits -> PK_UNTAGGED_MIN_SIMILARITY {tuned:.2f}")
if abs(L.PK_UNTAGGED_MIN_SIMILARITY - tuned) > 1e-9:
    problems.append(f"PK_UNTAGGED_MIN_SIMILARITY is {L.PK_UNTAGGED_MIN_SIMILARITY}, the tuning queries give {tuned:.2f}: update the constant in learning_recall.py")

# 2. recall through the real CLI (surreal lessons + pk knowledge, as kbd-memory-recall does)
def recall(query, threshold=None):
    code = "import sys; sys.path.insert(0, %r); import learning_recall as L; " % str(Path(lr).parent)
    if threshold is not None:
        code += "L.PK_UNTAGGED_MIN_SIMILARITY = %r; " % threshold
    code += "sys.argv = ['learning_recall.py', '--cwd', %r, '--role', %r, '--query', %r, '--budget', '8000', '--pk-budget', '2400', '--no-log']; raise SystemExit(L.main())" % (repo, role, query)
    out = subprocess.run([sys.executable, "-c", code], cwd=repo, capture_output=True, text=True, timeout=60).stdout
    return json.loads(out)

def is_foreign(item):
    return item.get("pkId") in foreign or any(m in item["text"].lower() for m in markers)

def is_current(item):
    return item["scope"] in ("role", "lead", "team", "project", "pk:project")

rows = []
def evaluate(label, q, assert_it):
    result = recall(q["text"])
    if not result["storeReachable"]:
        problems.append(f"{q['id']}: the scratch surreal-memory was not reachable")
    items = result["lessons"] + result["knowledge"]
    top = items[:5]
    current = sum(1 for i in top if is_current(i))
    leaked = [i.get("pkId") or i["text"][:40] for i in items if is_foreign(i)]
    must = q.get("mustRecall")
    found = (not must) or any(i.get("pkId") == must for i in items)
    rows.append((label, q["id"], len(items), current, len(leaked), "-" if not must else ("yes" if found else "NO")))
    if assert_it:
        if current < 3:
            problems.append(f"{q['id']}: only {current} of the top {len(top)} recalled items are current-project or role scope")
        if leaked:
            problems.append(f"{q['id']}: foreign-project entries recalled: {leaked}")
        if not found:
            problems.append(f"{q['id']}: current-project entry {must} was not recalled")

for q in fixture["queries"]["tuning"]:
    evaluate("tuning", q, False)
for q in fixture["queries"]["heldout"]:
    evaluate("held-out", q, True)

print("\n%-9s %-4s %-8s %-18s %-14s %s" % ("set", "id", "recalled", "top5 current/role", "foreign leaked", "must-recall"))
for row in rows:
    print("%-9s %-4s %-8d %-18d %-14d %s" % row)

# 3. negative control: without the threshold the held-out queries do leak foreign entries
leaks = 0
for q in fixture["queries"]["heldout"]:
    leaks += sum(1 for i in recall(q["text"], threshold=0.0)["knowledge"] if is_foreign(i))
print(f"control: with the threshold at 0.0 the held-out queries recall {leaks} foreign entries")
if leaks == 0:
    problems.append("control: the held-out queries never reach a foreign entry even unfiltered, so the assertions would be vacuous")

if problems:
    for p in problems:
        print("FAIL: " + p, file=sys.stderr)
    raise SystemExit(1)
print("\nok - held-out queries: >= 3 of the top 5 are current-project/role scope, no foreign entries, relevant project pk entry recalled")
PY
rc=$?
[ $rc -eq 0 ] || exit 1
scratch_surreal_stop || fail "scratch surreal-memory stop left a process or listener"
echo "test-recall-quality: passed"
