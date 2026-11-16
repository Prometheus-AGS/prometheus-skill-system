#!/bin/bash
# test-recall-paging.sh — learning_recall.py reads a whole scope listing, however
# the memory server pages it.
#
# surreal-memory-server pages GET /api/v1/memory since its issue #62: at most 200
# memories per response, ordered by record ID. Record IDs are random UUIDs, so one
# page is an arbitrary sample of a scope. A recall that reads one page and sorts
# it by time reports "the most recent" of that sample, not of the scope. Servers
# before that change answer with a bare array of the whole scope.
#
# Real processes, scratch everything: a scratch surreal-memory-server (embedded),
# a scratch git repo with a fixture team, a scratch HOME, and the real
# learning_recall.py CLI. The team digest scope (<team>/@team, the "recent" mode
# listing) is seeded with 180 older memories and then, after a pause, 50 newer
# ones: 230 in all, more than one page on a paging server.
#
#   1. the main-thread recall returns exactly the 50 newest team entries, and
#      reports no partial listing;
#   2. the listing was read through the path the server offers: more than one
#      page on a paging server, one bare array on an older one;
#   3. with the page cap lowered to one page, a paging server's listing is
#      reported in `partialListings` instead of passing as complete (skipped on a
#      server that does not page).
#
# Run it against both server generations by setting TLI_SM_BIN.
# Exit 0 pass, 1 fail, 2 BLOCKED (a prerequisite is missing). bash 3.2.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
LR="$ROOT/shared/scripts/lib/learning_recall.py"
LIB_DIR="$ROOT/shared/scripts/lib"
. "$HERE/lib/scratch-surreal.sh"   # before any HOME override (locates the real model cache)
PORT="$(scratch_surreal_pick_port)"; SCRATCH_SURREAL_PORT="$PORT"

OLDER=180
NEWER=50   # learning_recall.TEAM_DIGEST_LIMIT

blocked() { echo "BLOCKED: $*" >&2; exit 2; }
for tool in python3 git curl; do command -v "$tool" >/dev/null 2>&1 || blocked "$tool missing"; done

S="$(mktemp -d "${TMPDIR:-/tmp}/recall-paging.XXXXXX")"
S="$(cd "$S" && pwd -P)"
cleanup() { scratch_surreal_stop >/dev/null 2>&1; wait 2>/dev/null; if [ -n "${TLI_KEEP:-}" ]; then echo "kept $S" >&2; else rm -rf "$S"; fi; }
trap cleanup EXIT
export HOME="$S/home" CODEX_HOME="$S/codex" PROMETHEUS_PLUGIN_ROOT="$S/plugin-root"
export PROMETHEUS_LEARNING_QUEUE="$S/queue" PROMETHEUS_LEARNING_LOG_DIR="$S/log" PROMETHEUS_LEARNING_INDEX_DIR="$S/index"
export PROMETHEUS_TEAM_DIGEST_DIR="$S/team-digest" PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=1 PROMETHEUS_USER_ID=paging-fixture
unset PROMETHEUS_LEARNING_PK CLAUDE_PLUGIN_ROOT PLUGIN_ROOT KBD_PACK_ROOT PK_KB_DIR PROMETHEUS_HARNESS PROMETHEUS_MEMORY_URL
mkdir -p "$HOME" "$CODEX_HOME" "$PROMETHEUS_PLUGIN_ROOT"
git config --global user.email fixture@example.invalid; git config --global user.name Fixture

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }

# --- fixture repo with one team --------------------------------------------------
PROJECT="project:paging-fixture"
TEAM="tlm-paging"
REPO="$S/repo"
mkdir -p "$REPO/.agent-team/$TEAM" "$REPO/.prometheus" "$REPO/src"
git -C "$REPO" init -q
printf '{"projectId":"%s"}\n' "$PROJECT" > "$REPO/.prometheus/project.json"
cat > "$REPO/.agent-team/$TEAM/team.json" <<EOF
{"schemaVersion":1,"id":"$TEAM","roles":[
 {"id":"dev","description":"dev","prompt":"p","skills":[],"owns":["src/**"],"inputs":[],"outputs":[],"dependsOn":[]}]}
EOF
printf 'x\n' > "$REPO/src/a.ts"
git -C "$REPO" add -A >/dev/null; git -C "$REPO" commit -qm fixture

# --- scratch server, seeded in two batches separated in time ---------------------
scratch_surreal_start "$S/sm" paging
SM_VERSION="$(curl -fsS -m 5 "$SURREAL_MEMORY_URL/health" 2>/dev/null | python3 -c 'import json,sys; print(json.load(sys.stdin).get("version","?"))' 2>/dev/null || echo '?')"

seed() { # <prefix> <count>
  python3 - "$SURREAL_MEMORY_URL" "$PROJECT" "$TEAM/@team" "$1" "$2" <<'PY'
import json, random, sys, urllib.request
base, user, agent, prefix, count = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5])
opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
# The server merges a new memory into an existing one at 0.92 cosine similarity,
# so entries that differ only in a counter collapse into a handful. Each entry is
# eight words drawn from a varied vocabulary, which keeps them far apart.
VOCABULARY = """anchor ballot cactus dynamo ember falcon glacier harbor iodine jasmine kernel lantern
magnet nectar orchid pepper quartz raven saddle tundra umbrella velvet walnut xylophone yogurt zipper
abacus bakery canyon dentist eclipse furnace gazelle hammock incense jigsaw kettle lagoon mosaic
napkin oatmeal parsley quiver ribbon sawdust thimble utensil violin wardrobe yardstick zeppelin
avalanche biscuit compass drizzle engine flannel granite hornet island jungle kayak ladder mitten
noodle octopus pillow quilt rocket sponge teapot unicorn volcano whistle beacon cobalt dolphin
elbow fiddle garlic helmet igloo jacket koala lemon marble needle onion pencil rabbit salmon
tractor vinegar wagon yacht zebra almond bridge candle desert eagle forest guitar honey insect
jewel kitchen lizard meadow nickel ocean planet radio silver tunnel valley window blanket copper
diamond feather ginger hazel ivory juniper lilac maple nutmeg olive pebble rustle saffron thistle""".split()
for index in range(count):
    words = random.Random(f"{prefix}-{index}").sample(VOCABULARY, 8)
    body = {"content": f"{prefix}-{index:03d} " + " ".join(words), "user_id": user, "agent_id": agent}
    request = urllib.request.Request(base + "/api/v1/memory", data=json.dumps(body).encode(), method="POST",
                                     headers={"Content-Type": "application/json"})
    opener.open(request, timeout=60).read()
PY
}
seed OLDER "$OLDER" || fail "seeding the older batch"
sleep 3   # created_at must separate the batches whatever its resolution
seed NEWER "$NEWER" || fail "seeding the newer batch"
ok "seeded $((OLDER + NEWER)) team-digest memories on surreal-memory-server $SM_VERSION"

# --- how does this server answer a listing? -------------------------------------
SHAPE="$(python3 - "$SURREAL_MEMORY_URL" "$PROJECT" "$TEAM/@team" <<'PY'
import json, sys, urllib.parse, urllib.request
base, user, agent = sys.argv[1:4]
opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
url = base + "/api/v1/memory?" + urllib.parse.urlencode({"user_id": user, "agent_id": agent})
payload = json.loads(opener.open(url, timeout=30).read())
if isinstance(payload, list):
    print(f"array {len(payload)}")
else:
    print(f"paged {len(payload.get('memories', []))} {bool(payload.get('has_more'))}")
PY
)" || fail "probing the listing shape"
case "$SHAPE" in
  "array $((OLDER + NEWER))") PAGED=0; ok "server answers one bare array of the whole scope ($SHAPE)" ;;
  paged\ *\ True) PAGED=1; ok "server pages the listing; a default request returns only part of the scope ($SHAPE)" ;;
  *) fail "unexpected listing shape for a scope of $((OLDER + NEWER)): $SHAPE" ;;
esac

# --- 1. the real CLI returns the newest entries of the whole scope --------------
python3 "$LR" --main-thread --cwd "$REPO" --no-log --no-pk --budget 100000 > "$S/recall.json" 2>"$S/recall.err" \
  || fail "learning_recall.py exited non-zero: $(cat "$S/recall.err")"
python3 - "$S/recall.json" "$NEWER" <<'PY' || fail "main-thread recall did not return the newest team entries"
import json, sys
result, wanted = json.load(open(sys.argv[1])), int(sys.argv[2])
assert result.get("storeReachable") is True and result.get("lessonSource") == "surreal-memory", (result.get("storeReachable"), result.get("lessonSource"), result.get("error"))
team = [e for e in result["lessons"] if e["scope"] == "team"]
older = [e["text"][:9] for e in team if not e["text"].startswith("NEWER-")]
assert len(team) == wanted, f"{len(team)} team entries, expected {wanted}"
assert not older, f"{len(older)} of {len(team)} team entries are not from the newest batch, e.g. {older[:3]}"
assert len({e["text"] for e in team}) == wanted, "duplicate team entries"
assert result.get("partialListings") == [], result.get("partialListings")
PY
ok "recall returns exactly the $NEWER newest of $((OLDER + NEWER)) team entries, with no partial listing"

# --- 2. the listing really followed the server's pages ---------------------------
python3 -B - "$LIB_DIR" "$SURREAL_MEMORY_URL" "$PROJECT" "$TEAM/@team" "$PAGED" "$((OLDER + NEWER))" <<'PY' || fail "scope listing"
import json, sys, time
sys.path.insert(0, sys.argv[1])
import learning_recall as L
base, user, agent, paged, total = sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5] == "1", int(sys.argv[6])
requests = []
real = L._http_json
L._http_json = lambda method, url, body, timeout: (requests.append(url), real(method, url, body, timeout))[1]  # count only; the call is real
records, complete = L.list_scope(base, user, agent, time.monotonic() + 30)
assert complete, "listing not complete"
ids = {json.dumps(r["id"], sort_keys=True) for r in records}  # an id may be a string or an object
assert len(records) == total and len(ids) == total, (len(records), len(ids), total)
if paged:
    assert len(requests) > 1 and "cursor=" in requests[1], requests
else:
    assert len(requests) == 1, requests
PY
ok "the listing read all $((OLDER + NEWER)) memories exactly once, through the server's own paging"

# --- 3. a listing that stops early is reported, never passed off as complete -----
if [ "$PAGED" = 1 ]; then
  python3 -B - "$LIB_DIR" "$REPO" <<'PY' || fail "a capped listing was not reported as partial"
import sys
from pathlib import Path
sys.path.insert(0, sys.argv[1])
import learning_recall as L
L.MAX_LIST_PAGES = 1
result = L.recall(cwd=Path(sys.argv[2]), main_thread=True, use_pk=False, log=False, budget=100000)
assert result["storeReachable"] is True, result
assert result["partialListings"] == ["team"], result["partialListings"]
PY
  ok "with the page cap at one page, the team listing is reported in partialListings"
else
  echo "skip - partial-listing report (this server does not page; nothing can be cut short)"
fi

scratch_surreal_stop || fail "scratch surreal-memory did not stop cleanly"
echo "PASS: $pass checks (surreal-memory-server $SM_VERSION, $([ "$PAGED" = 1 ] && echo paged || echo bare-array) listing)"
