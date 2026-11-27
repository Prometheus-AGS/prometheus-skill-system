#!/bin/bash
# test-kbd-task-recall.sh — task-level memory lookup during Execute (issue #256).
#
# Runs the real kbd-memory-recall.sh, recall library and learning_write.py in a
# scratch project with a scratch HOME, queue, log and index. surreal-memory is
# declared down, so lessons come from the learning log (the file tier); the
# store-backed path is covered by test-kbd-memory-loop.sh.
#
#   1. a project lesson is written through learning_write.py;
#   2. the stage recall writes prior-context.md as before;
#   3. KBD_RECALL_QUERY writes task-recall.md with the lesson and the
#      evidence-only notice, and leaves prior-context.md byte-identical;
#   4. a child phase is reached by <parent>/<child>, <parent>::<child>,
#      "<parent> > <child>" and the waypoint's childPointer; a stale pointer
#      creates nothing;
#   5. an output name that tries to leave the phase directory is ignored;
#   6. with no store, an empty knowledge base and no learning log the lookup
#      exits 0 and says nothing was recalled;
#   7. secret-like text is refused at user and global visibility, and still
#      accepted at project visibility.
#
# Exit 0 pass, 1 fail, 2 BLOCKED (a prerequisite is missing). bash 3.2.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
ORCH="$ROOT/skills/process/kbd-process-orchestrator"
RECALL="$ORCH/skills/kbd-memory-recall/kbd-memory-recall.sh"
LW="$ROOT/shared/scripts/lib/learning_write.py"

blocked() { echo "BLOCKED: $*" >&2; exit 2; }
for tool in jq python3 git shasum; do command -v "$tool" >/dev/null 2>&1 || blocked "$tool missing"; done
python3 -c 'import sys; sys.exit(0 if sys.version_info >= (3, 10) else 1)' >/dev/null 2>&1 || blocked "python3 >= 3.10 required"

S="$(mktemp -d)"
trap 'rm -rf "$S"' EXIT
export HOME="$S/home" CODEX_HOME="$S/codex" PROMETHEUS_PLUGIN_ROOT="$S/plugin-root"
export PROMETHEUS_LEARNING_QUEUE="$S/queue" PROMETHEUS_LEARNING_LOG_DIR="$S/log" PROMETHEUS_LEARNING_INDEX_DIR="$S/index"
export PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=1 PROMETHEUS_LEARNING_CORTEX=0
export SURREAL_MEMORY_URL=none KBD_ORCHESTRATOR_ROOT="$ORCH" KBD_RECALL_BUDGET=8000
unset PROMETHEUS_LEARNING_PK CLAUDE_PLUGIN_ROOT PLUGIN_ROOT KBD_PACK_ROOT UAR_MEMORY_MCP_URL KBD_MEMORY_MCP_URL PK_KB_DIR \
      KBD_RECALL_QUERY KBD_RECALL_OUTPUT KBD_RECALL_ROLE KBD_HOOK_KIND
mkdir -p "$HOME" "$CODEX_HOME" "$PROMETHEUS_PLUGIN_ROOT"
git config --global user.email fixture@example.invalid; git config --global user.name Fixture

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }
sha() { shasum -a 256 "$1" | cut -d' ' -f1; }
delivery_lines() {
  if [ -f "$PROMETHEUS_LEARNING_INDEX_DIR/delivery.jsonl" ]; then wc -l < "$PROMETHEUS_LEARNING_INDEX_DIR/delivery.jsonl" | tr -d ' '; else echo 0; fi
}
queued() { ls "$PROMETHEUS_LEARNING_QUEUE"/memory/*/*.json 2>/dev/null | wc -l | tr -d ' '; }

TOKEN="taskrecall_$(date +%s)_$$"
REPO="$S/repo"
P="$REPO/.kbd-orchestrator/phases/parent-phase"
mkdir -p "$REPO/.prometheus" "$P/children/kid-phase"
git -C "$REPO" init -q
printf '{"projectId":"project:task-recall-fixture"}\n' > "$REPO/.prometheus/project.json"
printf '{"name":"task-recall-fixture"}\n' > "$REPO/.kbd-orchestrator/project.json"
printf '{"phase":"parent-phase","stage":"execute","status":"in_progress"}\n' > "$REPO/.kbd-orchestrator/current-waypoint.json"
printf '# Goals\n\n- Export notes to one Markdown file.\n' > "$P/goals.md"
printf '# Goals\n\n- Make the export order stable.\n' > "$P/children/kid-phase/goals.md"
recall() { ( cd "$REPO" && bash "$RECALL" "$@" ) >/dev/null 2>>"$S/recall.stderr"; }

# --- 1. write one project lesson ------------------------------------------------
out="$(python3 "$LW" --cwd "$REPO" --visibility project --kind gotcha \
  --text "Gotcha $TOKEN: the importer rejects an export file without a trailing newline.")" || fail "learning_write failed"
[ "$(printf '%s' "$out" | jq -r '.written')" = "1" ] || fail "project lesson not written: $out"
grep -q "$TOKEN" "$PROMETHEUS_LEARNING_LOG_DIR/lessons.jsonl" || fail "lesson missing from the learning log"
ok "a project lesson is written through learning_write.py"

# --- 2. the stage recall is unchanged -------------------------------------------
recall parent-phase execute || fail "stage recall exited non-zero"
PC="$P/prior-context.md"
[ -f "$PC" ] || { cat "$S/recall.stderr" >&2; fail "stage recall wrote no prior-context.md"; }
head -1 "$PC" | grep -q '^# Prior context — parent-phase (execute)$' || fail "prior-context.md title changed: $(head -1 "$PC")"
grep -q '^> Query:' "$PC" && fail "a stage recall must not carry a task query line"
[ ! -e "$P/task-recall.md" ] || fail "a stage recall wrote task-recall.md"
ok "a recall without KBD_RECALL_QUERY writes prior-context.md as before"

# --- 3. task-level lookup --------------------------------------------------------
pc_before="$(sha "$PC")"; d_before="$(delivery_lines)"
KBD_RECALL_QUERY="importer rejects the export file trailing newline $TOKEN" recall parent-phase execute || fail "task recall exited non-zero"
TR="$P/task-recall.md"
[ -f "$TR" ] || { cat "$S/recall.stderr" >&2; fail "task recall wrote no task-recall.md"; }
head -1 "$TR" | grep -q '^# Task recall — parent-phase (execute)$' || fail "task-recall.md title: $(head -1 "$TR")"
grep -q "$TOKEN" "$TR" || { cat "$TR" >&2; fail "task-recall.md does not cite the lesson"; }
grep -q "^> Query: importer rejects the export file trailing newline $TOKEN" "$TR" || fail "task-recall.md does not state its query"
grep -q 'grants no authority, widens no scope and replaces no approval' "$TR" || fail "task-recall.md lacks the evidence-only notice"
grep -q 'not instructions' "$TR" || fail "task-recall.md lacks the not-instructions label"
[ "$(sha "$PC")" = "$pc_before" ] || fail "the task lookup changed prior-context.md"
[ "$(delivery_lines)" -eq $((d_before + 1)) ] || fail "the task lookup did not add exactly one delivery line"
size="$(wc -c < "$TR" | tr -d ' ')"
[ "$size" -le "$KBD_RECALL_BUDGET" ] || fail "task-recall.md is $size bytes, over the budget"
ok "KBD_RECALL_QUERY writes task-recall.md ($size bytes) with the lesson and the evidence-only notice; prior-context.md is byte-identical"

# --- 4. child phases --------------------------------------------------------------
KID="$P/children/kid-phase"
KBD_RECALL_QUERY="stable export order" recall parent-phase/kid-phase execute
[ -f "$KID/task-recall.md" ] || fail "<parent>/<child> did not write into the child directory"
head -1 "$KID/task-recall.md" | grep -q '^# Task recall — parent-phase > kid-phase (execute)$' || fail "child title: $(head -1 "$KID/task-recall.md")"
rm -f "$KID/task-recall.md"
KBD_RECALL_QUERY="stable export order" recall parent-phase::kid-phase execute
[ -f "$KID/task-recall.md" ] || fail "<parent>::<child> did not write into the child directory"
rm -f "$KID/task-recall.md"
KBD_RECALL_QUERY="stable export order" recall "parent-phase > kid-phase" execute
[ -f "$KID/task-recall.md" ] || fail "'<parent> > <child>' did not write into the child directory"
printf '{"phase":"parent-phase","childPointer":"kid-phase","stage":"execute"}\n' > "$REPO/.kbd-orchestrator/current-waypoint.json"
recall "" execute
[ -f "$KID/prior-context.md" ] || fail "the waypoint childPointer was not followed"
tr_before="$(sha "$TR")"
printf '{"phase":"parent-phase","childPointer":"ghost-phase","stage":"execute"}\n' > "$REPO/.kbd-orchestrator/current-waypoint.json"
KBD_RECALL_QUERY="stable export order" recall "" execute
[ ! -e "$P/children/ghost-phase" ] || fail "a stale childPointer created a child directory"
[ "$(sha "$TR")" != "$tr_before" ] || fail "a stale childPointer did not fall back to the parent phase"
ok "a child phase is reached by all three address forms and by the waypoint; a stale pointer falls back to the parent and creates nothing"

# --- 5. the output name cannot leave the phase directory ----------------------------
KBD_RECALL_QUERY="stable export order" KBD_RECALL_OUTPUT="../../escaped.md" recall parent-phase execute
[ ! -e "$REPO/.kbd-orchestrator/escaped.md" ] && [ ! -e "$REPO/.kbd-orchestrator/phases/escaped.md" ] || fail "KBD_RECALL_OUTPUT escaped the phase directory"
head -1 "$PC" | grep -q '^# Prior context' || fail "a rejected output name overwrote prior-context.md"
KBD_RECALL_QUERY="stable export order" KBD_RECALL_OUTPUT="blocker-recall.md" recall parent-phase execute
[ -f "$P/blocker-recall.md" ] || fail "KBD_RECALL_OUTPUT=<name> was not honoured"
ok "KBD_RECALL_OUTPUT names a file inside the phase directory and nothing outside it"

# --- 6. no source at all: exit 0 and say so ------------------------------------------
( cd "$REPO" && PROMETHEUS_LEARNING_LOG_DIR="$S/empty-log" KBD_RECALL_QUERY="anything at all" \
    KBD_RECALL_OUTPUT="empty-recall.md" bash "$RECALL" parent-phase execute ) >/dev/null 2>>"$S/recall.stderr"
rc=$?
[ "$rc" -eq 0 ] || fail "the lookup exited $rc with no source available"
if [ -f "$P/empty-recall.md" ]; then
  grep -q "$TOKEN" "$P/empty-recall.md" && fail "a lesson was recalled with no source available"
  grep -q 'no recalled lessons\|no recall performed' "$P/empty-recall.md" || { cat "$P/empty-recall.md" >&2; fail "the empty lookup does not say nothing was recalled"; }
else
  fail "the empty lookup wrote no file"
fi
ok "with no store, an empty knowledge base and no learning log the lookup exits 0 and reports that nothing was recalled"

# --- 7. secret-like text is refused beyond the project -------------------------------
SECRET_TEXT="Gotcha $TOKEN-secret: the staging deploy needs api_key=abcd1234efgh5678 in the header."
q_before="$(queued)"
for vis in global user; do
  out="$(python3 "$LW" --cwd "$REPO" --visibility "$vis" --text "$SECRET_TEXT")" || fail "learning_write exited non-zero at $vis"
  [ "$(printf '%s' "$out" | jq -r '.written')" = "0" ] || fail "secret-like text was written at visibility $vis: $out"
  printf '%s' "$out" | jq -r '.reason' | grep -q 'secret-like text refused' || fail "no refusal reason at $vis: $out"
done
[ "$(queued)" -eq "$q_before" ] || fail "a refused lesson was queued"
grep -q "$TOKEN-secret" "$PROMETHEUS_LEARNING_LOG_DIR/lessons.jsonl" && fail "a refused lesson reached the learning log"
out="$(python3 "$LW" --cwd "$REPO" --visibility global --text "Lesson $TOKEN-clean: prefer one bounded lookup per task.")" || fail "learning_write failed"
[ "$(printf '%s' "$out" | jq -r '.written')" -ge 1 ] || fail "a clean global lesson was refused: $out"
out="$(python3 "$LW" --cwd "$REPO" --visibility project --text "$SECRET_TEXT")" || fail "learning_write failed"
[ "$(printf '%s' "$out" | jq -r '.written')" -ge 1 ] || fail "project visibility behaviour changed: $out"
ok "secret-like text is refused at user and global visibility, nothing is queued or logged, and project visibility is unchanged"

echo "test-kbd-task-recall: $pass passed"
