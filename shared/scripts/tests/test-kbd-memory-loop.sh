#!/bin/bash
# test-kbd-memory-loop.sh — integration test for the KBD memory loop (B4).
#
# Real processes end to end, nothing touches the user's HOME, queue, memory
# service or ~/.claude:
#   1. a reflection carrying a unique token is written for phase-one;
#   2. the orchestrator `reflect:after` hook runs memory-writeback.sh — resolved
#      from the source tree, from the flat installed layout (dist/plugins/claude)
#      and through the plugin root — and queues attributed lessons;
#   3. a real prometheus-learning-worker `run-once` delivers them to a real
#      scratch surreal-memory-server (embedded, local MLX embeddings);
#      memory-writeback's detached `pk ingest` compiles them into the project KB;
#   4. the real kbd-next-phase seeds phase-two from the reflection;
#   5. the real `assess:before` hook runs kbd-memory-recall, and prior-context.md
#      must cite the token from surreal-memory AND from pk, stay under budget,
#      and add one delivery.jsonl line with per-channel bytes;
#   6. recall for role R never returns another role's private memory (seeded);
#   7. the `assess:after` stage write-back reaches the lead view of the next stage.
#
# Binaries: TLI_SM_BIN, TLI_WORKER_BIN, TLI_PK_BIN override surreal-memory-server,
# prometheus-learning-worker and pk on PATH (all must be >= 1.10.0).
# Exit 0 pass, 1 fail, 2 BLOCKED (a prerequisite is missing). bash 3.2.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
ORCH="$ROOT/skills/process/kbd-process-orchestrator"
FLAT_ORCH="$ROOT/dist/plugins/claude/prometheus-skill-pack/skills/kbd-process-orchestrator"
LW="$ROOT/shared/scripts/lib/learning_write.py"
LR="$ROOT/shared/scripts/lib/learning_recall.py"
. "$HERE/lib/scratch-surreal.sh"   # before any HOME override (locates the real model cache)
WORKER_BIN="${TLI_WORKER_BIN:-$(command -v prometheus-learning-worker || true)}"
PK_BIN="${TLI_PK_BIN:-$(command -v pk || true)}"
PORT="$(scratch_surreal_pick_port)"; SCRATCH_SURREAL_PORT="$PORT"
BUDGET=8000

blocked() { echo "BLOCKED: $*" >&2; exit 2; }
version_ok() { "$1" --version 2>/dev/null | grep -Eq '1\.(1[0-9]|[2-9][0-9])\.'; }
[ -x "$WORKER_BIN" ] || blocked "prometheus-learning-worker not found (set TLI_WORKER_BIN)"
[ -x "$PK_BIN" ] || blocked "pk not found (set TLI_PK_BIN or put pk >= 1.10 on PATH)"
version_ok "$WORKER_BIN" || blocked "prometheus-learning-worker < 1.10.0"
version_ok "$PK_BIN" || blocked "pk < 1.10.0"
for tool in jq python3 node git curl; do command -v "$tool" >/dev/null 2>&1 || blocked "$tool missing"; done
[ -f "$FLAT_ORCH/hooks/hooks.json" ] || blocked "flat installed layout missing (run the distribution generator)"

S="$(mktemp -d)"
cleanup() { scratch_surreal_stop >/dev/null 2>&1; wait 2>/dev/null; if [ -n "${TLI_KEEP:-}" ]; then echo "kept $S" >&2; else rm -rf "$S"; fi; }
trap cleanup EXIT
REAL_USER="$(id -un)"
export HOME="$S/home" CODEX_HOME="$S/codex" PROMETHEUS_PLUGIN_ROOT="$S/plugin-root"
export PROMETHEUS_LEARNING_QUEUE="$S/queue" PROMETHEUS_LEARNING_LOG_DIR="$S/log" PROMETHEUS_LEARNING_INDEX_DIR="$S/index"
export PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=1
export SURREAL_MEMORY_URL="http://127.0.0.1:$PORT" KBD_MEMORY_MCP_URL="http://127.0.0.1:$PORT"
export KBD_RECALL_BUDGET="$BUDGET"
export PATH="$(dirname "$PK_BIN"):$PATH"
unset PROMETHEUS_LEARNING_PK CLAUDE_PLUGIN_ROOT PLUGIN_ROOT KBD_PACK_ROOT KBD_ORCHESTRATOR_ROOT UAR_MEMORY_MCP_URL PK_KB_DIR
mkdir -p "$HOME" "$CODEX_HOME" "$PROMETHEUS_PLUGIN_ROOT"
git config --global user.email fixture@example.invalid; git config --global user.name Fixture

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }

TOKEN="kbdloop_$(date +%s)_$$"
PRIVATE_UI="uiprivate_$$_$(date +%s)"
OWN_API="apiown_$$_$(date +%s)"
CODIFY="codifyonly_$$"

# --- fixture project ----------------------------------------------------------
make_project() { # <dir>
  local repo="$1"
  mkdir -p "$repo/.agent-team/tlm-fixture" "$repo/.prometheus" "$repo/.kbd-orchestrator/phases/phase-one"
  git -C "$repo" init -q
  printf '{"projectId":"project:b4-fixture"}\n' > "$repo/.prometheus/project.json"
  cat > "$repo/.agent-team/tlm-fixture/team.json" <<'EOF'
{"schemaVersion":1,"id":"tlm-fixture","roles":[
 {"id":"lead","lead":true,"description":"lead","prompt":"p","skills":[],"owns":[],"inputs":[],"outputs":[],"dependsOn":[]},
 {"id":"api-dev","description":"api","prompt":"p","skills":[],"owns":["src/api/**"],"inputs":[],"outputs":[],"dependsOn":[]},
 {"id":"ui-dev","description":"ui","prompt":"p","skills":[],"owns":["src/ui/**"],"inputs":[],"outputs":[],"dependsOn":[]}]}
EOF
  printf '{"name":"b4-fixture"}\n' > "$repo/.kbd-orchestrator/project.json"
  printf '{"phase":"phase-one","stage":"reflect_complete","status":"reflect_complete"}\n' > "$repo/.kbd-orchestrator/current-waypoint.json"
  printf '{"phase":"phase-one"}\n' > "$repo/.kbd-orchestrator/phases/phase-one/progress.json"
  cat > "$repo/.kbd-orchestrator/phases/phase-one/reflection.md" <<EOF
# Phase Reflection: phase-one

**Project:** b4-fixture

## Delta

1. The reflect:after write-back never ran in the installed layout.

## Root Cause

1. The hook resolved memory-writeback.sh three directories above the orchestrator, outside a flat install.

## Corrective Actions

1. Resolve pack scripts through the plugin root, then the flat layout, then the source tree.

## Goals

| Goal | Status | Notes |
| ---- | ------ | ----- |
| Lessons return at the next stage | PARTIAL | recall read lifecycle metadata only |

## Technical Debt

- kbd-memory-recall ranked lifecycle events instead of lessons

## Lessons Learned

- Lesson \`$TOKEN\`: always resolve KBD hook scripts through the plugin root before any repo-relative path.
- [GLOBAL] Equality-filtered agent_id scopes keep role-private lessons out of other roles' recall.

## Next Phase Seed

\`phase-two-loop\` — recall lessons at assess, stage write-back, budgeted prior context.

## Codify as Skill?

- $CODIFY pattern: never written back.
EOF
}
REPO="$S/repo"; make_project "$REPO"
ops() { cat "$1"/memory/*/*.json 2>/dev/null; }

# --- 1. reflect:after resolves memory-writeback.sh in every layout -------------
fire_reflect() { # <orchestrator-root> <repo> <queue> [plugin-root]
  ( cd "$2" && export KBD_ORCHESTRATOR_ROOT="$1" PROMETHEUS_LEARNING_QUEUE="$3" \
      PROMETHEUS_LEARNING_INDEX_DIR="$3.index" PROMETHEUS_LEARNING_LOG_DIR="$3.log"
    [ -n "${4:-}" ] && export CLAUDE_PLUGIN_ROOT="$4"
    . "$1/shared/lib/waypoint.sh"; . "$1/shared/lib/hooks.sh"
    kbd_hooks_fire reflect after phase-one 1 1 ) </dev/null >/dev/null 2>"$3.stderr"
}
has_token_op() { ops "$1" | grep -q "$TOKEN"; }

REPO_FLAT="$S/repo-flat"; make_project "$REPO_FLAT"
PROMETHEUS_LEARNING_PK=0 fire_reflect "$FLAT_ORCH" "$REPO_FLAT" "$S/queue-flat"
has_token_op "$S/queue-flat" || { cat "$S/queue-flat.stderr" >&2; fail "flat layout: reflect:after did not run memory-writeback.sh"; }
ok "reflect:after resolves memory-writeback.sh in the flat installed layout (dist/plugins/claude, no plugin-root env)"

REPO_PLUGIN="$S/repo-plugin"; make_project "$REPO_PLUGIN"
mkdir -p "$S/bare"; cp -R "$ORCH" "$S/bare/kbd-process-orchestrator"
PROMETHEUS_LEARNING_PK=0 fire_reflect "$S/bare/kbd-process-orchestrator" "$REPO_PLUGIN" "$S/queue-plugin" "$ROOT"
has_token_op "$S/queue-plugin" || { cat "$S/queue-plugin.stderr" >&2; fail "plugin root: reflect:after did not run memory-writeback.sh"; }
ok "reflect:after resolves memory-writeback.sh through CLAUDE_PLUGIN_ROOT when the orchestrator stands alone"

fire_reflect "$ORCH" "$REPO" "$PROMETHEUS_LEARNING_QUEUE"
has_token_op "$PROMETHEUS_LEARNING_QUEUE" || { cat "$PROMETHEUS_LEARNING_QUEUE.stderr" >&2; fail "source tree: reflect:after did not run memory-writeback.sh"; }
ok "reflect:after resolves memory-writeback.sh in the source tree"
# the source run used the default queue/index dirs set above
export PROMETHEUS_LEARNING_INDEX_DIR="$PROMETHEUS_LEARNING_QUEUE.index" PROMETHEUS_LEARNING_LOG_DIR="$PROMETHEUS_LEARNING_QUEUE.log"

ops "$PROMETHEUS_LEARNING_QUEUE" | python3 -c '
import json, sys
token, codify = sys.argv[1], sys.argv[2]
docs = [json.loads(l) for l in sys.stdin if l.strip()]
args = [d["arguments"] for d in docs if d.get("method") == "add_memory"]
keys = {(a["user_id"], a["agent_id"]) for a in args if token in a["content"]}
assert ("project:b4-fixture", "@project") in keys, keys
assert any(a["agent_id"] == "@global" and a["content"].startswith("Equality-filtered") for a in args), [a["agent_id"] for a in args]
assert any("Delta:" in a["content"] and "Corrective Actions:" in a["content"] for a in args)
assert any(a["content"].startswith("Phase phase-one next-phase seed") for a in args)
assert not any(codify in a["content"] for a in args), "Codify as Skill? was written back"
assert all("<!-- prometheus-envelope " in a["content"] and "env:1" in a["categories"] for a in args)
' "$TOKEN" "$CODIFY" || fail "write-back records"
before="$(ops "$PROMETHEUS_LEARNING_QUEUE" | grep -c '"add_memory"')"
fire_reflect "$ORCH" "$REPO" "$PROMETHEUS_LEARNING_QUEUE"
[ "$(ops "$PROMETHEUS_LEARNING_QUEUE" | grep -c '"add_memory"')" -eq "$before" ] || fail "a second reflect:after re-queued the same lessons"
ok "write-back: project/global/seed records with envelopes, Codify as Skill? excluded, per-phase dedupe holds"
TOKEN_H="$(ops "$PROMETHEUS_LEARNING_QUEUE" | python3 -c '
import json, sys
for line in sys.stdin:
    a = json.loads(line).get("arguments", {})
    if sys.argv[1] in a.get("content", "") and a.get("agent_id") == "@project":
        print(next(c[2:] for c in a["categories"] if c.startswith("h:"))); break
' "$TOKEN")"
[ -n "$TOKEN_H" ] || fail "token lesson has no h: category"

# --- 2. seed role-private lessons (one per role) ------------------------------
printf '%s' '{"agent_type":"prometheus-skill-pack:ui-dev","agent_id":"a-ui","session_id":"s-b4","cwd":"'"$REPO"'"}' \
  | python3 "$LW" --payload-stdin --cwd "$REPO" --visibility agent --text "Private ui note $PRIVATE_UI: keep the token budget dial private to ui-dev." >/dev/null
printf '%s' '{"agent_type":"prometheus-skill-pack:api-dev","agent_id":"a-api","session_id":"s-b4","cwd":"'"$REPO"'"}' \
  | python3 "$LW" --payload-stdin --cwd "$REPO" --visibility agent --text "Own api note $OWN_API: api-dev retries idempotent writes." >/dev/null

# --- 3. real scratch surreal-memory + real worker run-once ---------------------
scratch_surreal_start "$S/sm" b4
deliver() {
  for _ in 1 2 3 4 5 6 7 8; do
    "$WORKER_BIN" --memory-url "$SURREAL_MEMORY_URL" run-once >/dev/null 2>&1
    [ -z "$(ls "$PROMETHEUS_LEARNING_QUEUE"/memory/pending "$PROMETHEUS_LEARNING_QUEUE"/memory/submitting "$PROMETHEUS_LEARNING_QUEUE"/memory/accepted 2>/dev/null | grep json)" ] && return 0
    sleep 3
  done
  return 1
}
deliver || fail "worker left memory operations undelivered"
got="$(curl -fsS -m 10 "$SURREAL_MEMORY_URL/api/v1/memory?user_id=project:b4-fixture&agent_id=@project")"
printf '%s' "$got" | grep -q "$TOKEN" || fail "the token lesson is not in surreal-memory under (project:b4-fixture, @project)"
ok "the real worker delivered the reflection lessons to the scratch surreal-memory"

# --- 4. pk: memory-writeback's detached ingests reach the project KB ----------
# One entry per project lesson. pk compiles each with its model, which may
# reword or drop identifiers, so the entry is matched by the source it carries:
# learning:<content hash>, the same h: hash as the surreal-memory record.
for _ in $(seq 1 90); do grep -lq "learning:$TOKEN_H" "$REPO/.prometheus/knowledge/wiki/"*.md 2>/dev/null && break; sleep 2; done
if ! grep -lq "learning:$TOKEN_H" "$REPO/.prometheus/knowledge/wiki/"*.md 2>/dev/null; then
  ls "$REPO/.prometheus/knowledge/wiki/"*.md >/dev/null 2>&1 && fail "pk compiled entries, but none carries the token lesson's source learning:$TOKEN_H"
  blocked "pk ingest produced no project KB entry (pk needs a working model route)"
fi
ok "the token lesson is compiled into the project pk KB (source learning:$TOKEN_H)"

# --- 5. kbd-next-phase, then assess:before recall ------------------------------
( cd "$REPO" && KBD_ORCHESTRATOR_ROOT="$ORCH" bash "$ORCH/skills/kbd-next-phase/scripts/kbd-next-phase.sh" phase-two-loop ) \
  </dev/null >"$S/next-phase.log" 2>&1 || { cat "$S/next-phase.log" >&2; fail "kbd-next-phase failed"; }
[ "$(jq -r '.phase' "$REPO/.kbd-orchestrator/current-waypoint.json")" = "phase-two-loop" ] || fail "kbd-next-phase did not activate phase-two-loop"
grep -q 'Seeded from: `phase-one/reflection.md`' "$REPO/.kbd-orchestrator/phases/phase-two-loop/goals.md" || fail "goals.md not seeded"
ok "the real kbd-next-phase seeded phase-two-loop from the reflection"

delivery_lines() {
  if [ -f "$PROMETHEUS_LEARNING_INDEX_DIR/delivery.jsonl" ]; then wc -l < "$PROMETHEUS_LEARNING_INDEX_DIR/delivery.jsonl" | tr -d ' '; else echo 0; fi
}
d_before="$(delivery_lines)"
fire_stage() { # <stage> <edge>
  ( cd "$REPO" && export KBD_ORCHESTRATOR_ROOT="$ORCH"
    . "$ORCH/shared/lib/waypoint.sh"; . "$ORCH/shared/lib/hooks.sh"
    kbd_hooks_fire "$1" "$2" phase-two-loop 1 1 ) </dev/null >/dev/null 2>>"$S/hooks.stderr"
}
fire_stage assess before
PC="$REPO/.kbd-orchestrator/phases/phase-two-loop/prior-context.md"
[ -f "$PC" ] || { cat "$S/hooks.stderr" >&2; fail "assess:before wrote no prior-context.md"; }
python3 - "$PC" "$TOKEN" "$PRIVATE_UI" "$CODIFY" <<'PY' || { cat "$PC" >&2; fail "prior-context.md content"; }
import re, sys
text, token, private, codify = open(sys.argv[1]).read(), sys.argv[2], sys.argv[3], sys.argv[4]
def section(name):
    m = re.search(r"(?ms)^## " + re.escape(name) + r"\s*$(.*?)(?=^## |\Z)", text)
    return m.group(1) if m else ""
for name in ("Lessons", "pk knowledge", "Previous reflection", "Knowledge gaps"):
    assert f"\n## {name}\n" in text, f"missing section {name}"
lessons, knowledge = section("Lessons"), section("pk knowledge")
assert any(token in line and "via surreal-memory" in line for line in lessons.splitlines()), "token not cited from surreal-memory"
assert "Corrective Actions" in section("Previous reflection") and "phase-two-loop" in section("Previous reflection")
assert "PARTIAL" in section("Knowledge gaps") and "lifecycle events" in section("Knowledge gaps")
assert private not in text, "another role's private lesson leaked into the lead view"
assert codify not in text
PY
ok "assess prior-context.md cites the token from surreal-memory, with all four sections (pk copies of stored lessons de-duplicated), previous reflection and gaps"

size="$(wc -c < "$PC" | tr -d ' ')"
[ "$size" -le "$BUDGET" ] || fail "prior-context.md is $size bytes, over the $BUDGET budget"
d_after="$(delivery_lines)"
[ "$d_after" -eq $((d_before + 1)) ] || fail "delivery.jsonl gained $((d_after - d_before)) lines, expected 1"
tail -1 "$PROMETHEUS_LEARNING_INDEX_DIR/delivery.jsonl" | python3 -c '
import json, sys
r = json.loads(sys.stdin.read())
assert r["agentType"] == "kbd-assess", r
b = r["bytesByChannel"]
assert b.get("surreal-memory", 0) > 0 and b.get("reflection", 0) > 0 and b.get("gaps", 0) > 0, b
assert r["entriesByScope"].get("project", 0) >= 1 and "ts" in r, r
' || fail "delivery.jsonl line lacks per-channel bytes"
ok "prior-context.md is $size bytes (budget $BUDGET) and delivery.jsonl gained one line with per-channel bytes"

# --- 6. isolation: recall for role R never returns another role's private memory
recall_json() { (cd "$REPO" && python3 "$LR" --cwd "$REPO" --no-pk --no-log "$@"); }
python3 - "$(recall_json --role api-dev --query "private ui note token budget dial api retries")" \
          "$(recall_json --role ui-dev --query "private ui note token budget dial")" \
          "$(recall_json --main-thread --query "private ui note token budget dial")" "$PRIVATE_UI" "$OWN_API" <<'PY' || fail "role isolation"
import json, sys
api, ui, main = (json.loads(sys.argv[i]) for i in (1, 2, 3))
private, own = sys.argv[4], sys.argv[5]
def texts(r): return [l["text"] for l in r["lessons"]]
assert api["storeReachable"] and api["lessonSource"] == "surreal-memory", api
assert any(own in t for t in texts(api)), "api-dev does not see its own private lesson"
assert not any(private in t for t in texts(api)), "api-dev recalled ui-dev's private lesson"
assert all(l["agentId"] in ("tlm-fixture/api-dev", "tlm-fixture/@team", "@project", "@user", "@global") for l in api["lessons"]), [l["agentId"] for l in api["lessons"]]
assert any(private in t for t in texts(ui)), "seed not delivered: ui-dev cannot see its own lesson (negative check would be vacuous)"
assert not any(private in t or own in t for t in texts(main)), "the lead view recalled a role-private lesson"
PY
ok "recall for api-dev returns its own lesson and never ui-dev's private one (ui-dev sees it; the lead view sees neither)"

# --- 7. assess:after stage write-back reaches the next stage's lead view --------
( cd "$REPO" && KBD_ORCHESTRATOR_ROOT="$ORCH" bash -c '. "$KBD_ORCHESTRATOR_ROOT/shared/lib/stage-gate.sh"; kbd_stage_handoff_write assess "Stage summary marker '"$TOKEN"'-handoff: two gaps remain." ".kbd-orchestrator/phases/phase-two-loop"' ) \
  >/dev/null 2>&1 || fail "could not write the assess handoff"
fire_stage assess after
deliver || fail "worker left the stage summary undelivered"
fire_stage analyze before
grep -q "${TOKEN}-handoff" "$PC" || { cat "$PC" >&2; fail "analyze prior-context.md lacks the assess stage summary"; }
grep "${TOKEN}-handoff" "$PC" | grep -q '^- \[lead\]' || fail "the stage summary was not recalled from the lead scope"
ok "assess:after writes the stage handoff summary at visibility lead and analyze:before recalls it"

# --- 8. pk path: with the store down, the token lesson comes back from pk ------
scratch_surreal_stop || fail "scratch surreal-memory stop left a process or listener"
curl -fsS -m 1 "$SURREAL_MEMORY_URL/health" >/dev/null 2>&1 && fail "scratch surreal-memory still answering after stop"
d_before="$(delivery_lines)"
fire_stage assess before
python3 - "$PC" "$TOKEN_H" "$PRIVATE_UI" <<'PY' || { cat "$PC" >&2; fail "pk-path prior-context.md"; }
import re, sys
text, digest, private = open(sys.argv[1]).read(), sys.argv[2], sys.argv[3]
assert "lessons from pk" in text, "lessons did not fall back to pk"
lessons = re.search(r"(?ms)^## Lessons\s*$(.*?)(?=^## )", text).group(1)
assert any(f"via pk learning:{digest}" in line for line in lessons.splitlines()), "token lesson not recalled from pk"
assert private not in text
PY
[ "$(delivery_lines)" -eq $((d_before + 1)) ] || fail "pk-path recall did not add exactly one delivery line"
tail -1 "$PROMETHEUS_LEARNING_INDEX_DIR/delivery.jsonl" | python3 -c 'import json,sys; b=json.loads(sys.stdin.read())["bytesByChannel"]; assert b.get("pk",0)>0 and not b.get("surreal-memory"), b' \
  || fail "pk-path delivery line lacks pk bytes"
ok "with surreal-memory down, assess recall cites the token lesson from pk (learning:$TOKEN_H)"

# --- 9. file tier: no store, no pk -> the learning log, still role-isolated -----
python3 - "$(recall_json --memory-url none --main-thread --query "resolve hook scripts plugin root")" \
          "$(recall_json --memory-url none --role api-dev --query "private ui note api retries")" "$TOKEN" "$PRIVATE_UI" "$OWN_API" <<'PY' || fail "file tier"
import json, sys
main, api = json.loads(sys.argv[1]), json.loads(sys.argv[2])
token, private, own = sys.argv[3:6]
assert main["lessonSource"] == "file" and any(token in l["text"] for l in main["lessons"]), main["lessonSource"]
assert not any(private in l["text"] or own in l["text"] for l in main["lessons"])
assert any(own in l["text"] for l in api["lessons"]) and not any(private in l["text"] for l in api["lessons"])
PY
ok "with no store and no pk, recall reads the learning log, keeps the token, and stays role-isolated"

echo "test-kbd-memory-loop: $pass passed"
