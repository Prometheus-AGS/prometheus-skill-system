#!/bin/bash
# test-skill-candidates.sh — C3b gate: skill candidates in kbd-open and reflect.
#
#   test-skill-candidates.sh
#
# Real processes, no mocks of the behaviour under test:
#   1. kbd-open.sh with the real pk >= 1.11.0 prints nothing for skill candidates
#      when none are pending, lists a seeded new-skill and a seeded skill-update
#      candidate (full id, type, title, evidence count) when both are pending, and
#      keeps them apart from promotion candidates;
#   2. `pk candidates accept --kind skill` prints a /pmpo-skill-creator invocation
#      (with --update <skill> for the update candidate), moves the candidate to
#      accepted/ and creates no skill; `reject` drops another candidate from the
#      listing; the section then disappears;
#   3. with a pk that predates `candidates`, or no pk at all, kbd-open prints no
#      skill section and exits 0;
#   4. the reflect template keeps `## Codify as Skill?` out of write-back
#      (memory-writeback.sh) while kbd-reflect reviews skill candidates only on an
#      explicit human instruction;
#   5. propose-skill-update.sh no longer claims to be called by evaluate-session.sh
#      and the pmpo-skill-creator integration doc describes the candidate flow.
#
# Isolation: scratch HOME, CODEX_HOME and PROMETHEUS_PLUGIN_ROOT; nothing real is
# written. Binary: TLI_PK_BIN (else PATH) must be pk >= 1.11.0 for steps 1-2; without
# it steps 3-5 still run and the script exits 2 (BLOCKED), never 0.
# Exit 0 pass, 1 fail, 2 BLOCKED. bash 3.2.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
KBD_OPEN="$ROOT/shared/scripts/kbd-open.sh"
WRITEBACK="$ROOT/shared/scripts/memory-writeback.sh"
PROPOSE="$ROOT/shared/scripts/propose-skill-update.sh"
REFLECT_SKILL="$ROOT/skills/process/kbd-process-orchestrator/skills/kbd-reflect/SKILL.md"
REFLECT_PROMPT="$ROOT/skills/process/kbd-process-orchestrator/prompts/reflect.md"
INTEGRATION_DOC="$ROOT/skills/process/kbd-process-orchestrator/references/integrations/pmpo-skill-creator.md"
PK_BIN="${TLI_PK_BIN:-$(command -v pk || true)}"

for tool in python3 jq git; do command -v "$tool" >/dev/null 2>&1 || { echo "BLOCKED: $tool missing" >&2; exit 2; }; done

S="$(mktemp -d "${TMPDIR:-/tmp}/tli-c3b.XXXXXX")"
S="$(cd "$S" && pwd -P)"
cleanup() { rm -rf "$S"; }
trap cleanup EXIT
export HOME="$S/home" CODEX_HOME="$S/codex" PROMETHEUS_PLUGIN_ROOT="$S/plugin-root"
export PROMETHEUS_LEARNING_QUEUE="$S/queue" PROMETHEUS_LEARNING_LOG_DIR="$S/log" PROMETHEUS_LEARNING_INDEX_DIR="$S/index"
export PROMETHEUS_PROJECT_ID_SKIP_RUNTIME=1
unset PROMETHEUS_LEARNING_PK PROMETHEUS_USER_ID PROMETHEUS_PROJECT_ID PROMETHEUS_HARNESS PK_KB_DIR CLAUDE_PLUGIN_ROOT PLUGIN_ROOT
mkdir -p "$HOME" "$CODEX_HOME" "$PROMETHEUS_PLUGIN_ROOT" "$S/bin" "$S/proj"
git config --global user.email c3b-fixture@example.invalid
git config --global user.name Fixture

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }
version_ok() { "$1" --version 2>/dev/null | grep -Eq '1\.(1[1-9]|[2-9][0-9])\.'; }
snapshot() { ( cd "$S/proj" && PATH="$1" bash "$KBD_OPEN" 2>/dev/null ); }
BASEPATH="/usr/bin:/bin:/usr/sbin:/sbin"
d="$(dirname "$(command -v python3)")"; case ":$BASEPATH:" in *":$d:"*) ;; *) BASEPATH="$BASEPATH:$d" ;; esac

# a pk that predates `candidates`: rejects every subcommand
cat > "$S/bin/pk" <<'SH'
#!/bin/sh
exit 2
SH
chmod +x "$S/bin/pk"

# --- 3. older / absent pk -----------------------------------------------------
out="$(snapshot "$S/bin:$BASEPATH")"; rc=$?
[ "$rc" = 0 ] || fail "kbd-open exited $rc with a pk that has no candidates subcommand"
printf '%s' "$out" | grep -q 'Skill candidates' && fail "older pk printed a skill section"
out="$(snapshot "$BASEPATH")"; rc=$?
[ "$rc" = 0 ] && ! printf '%s' "$out" | grep -q 'Skill candidates' || fail "no pk at all must also be silent and exit 0"
ok "older or absent pk: no skill section, exit 0"

# --- 4. reflect template and write-back ---------------------------------------
grep -q '^### 11. Codify as Skill?' "$REFLECT_PROMPT" || fail "reflect template lost the Codify as Skill? dimension"
grep -q '^## Codify as Skill?' "$REFLECT_PROMPT" || fail "reflect template lost the Codify as Skill? output section"
grep -q 'pk candidates list --kind skill' "$REFLECT_PROMPT" || fail "reflect template does not mention skill candidates"
grep -q 'pk candidates list --kind skill' "$REFLECT_SKILL" || fail "kbd-reflect has no skill candidate review step"
grep -q 'explicit human instruction' "$REFLECT_SKILL" || fail "kbd-reflect review is not gated on a human instruction"
grep -q 'pk candidates accept --kind skill' "$REFLECT_SKILL" || fail "kbd-reflect does not name the accept command"
grep -q 'never creates or edits a skill' "$REFLECT_SKILL" || fail "kbd-reflect does not state that accept creates nothing"

A="$S/proj"; git -C "$A" init -q; mkdir -p "$A/.prometheus"
printf '{"projectId":"project:c3b-a"}\n' > "$A/.prometheus/project.json"
printf 'x\n' > "$A/f.txt"; git -C "$A" add -A >/dev/null; git -C "$A" commit -qm fixture
PHASE="$A/.kbd-orchestrator/phases/p1"; mkdir -p "$PHASE"
echo '{ "phase": "p1" }' > "$A/.kbd-orchestrator/current-waypoint.json"
echo '{ "phase": "p1" }' > "$PHASE/progress.json"
printf '# Reflection - p1\n\n## Delta\n1. Missed.\n\n## Root Cause\n1. Assumed.\n\n## Lessons Learned\n- LESSONTOKEN keep command lines exact in runbooks.\n\n## Codify as Skill?\n- CODIFYTOKEN deploy-runbook pattern\n' > "$PHASE/reflection.md"
( cd "$A" && printf '{"tool_input":{"file_path":"%s"}}' "$PHASE/reflection.md" | PATH="$S/bin:$PATH" bash "$WRITEBACK" >/dev/null 2>&1 ) || fail "memory-writeback exited non-zero"
find "$PROMETHEUS_LEARNING_QUEUE" -type f -name '*.json' -exec cat {} + 2>/dev/null | grep -q LESSONTOKEN || fail "the lesson was not written back (the exclusion check would be vacuous)"
find "$PROMETHEUS_LEARNING_QUEUE" -type f -name '*.json' -exec cat {} + 2>/dev/null | grep -q CODIFYTOKEN && fail "Codify as Skill? was written back"
ok "reflect: Codify as Skill? excluded from write-back; skill candidate review is human-gated"

# --- 5. propose-skill-update header and integration doc ------------------------
grep -q 'Called by evaluate-session.sh' "$PROPOSE" && fail "propose-skill-update.sh still claims to be called by evaluate-session.sh"
sed -n 1,8p "$PROPOSE" | grep -q 'MANUAL entry point' || fail "propose-skill-update.sh header does not say it is manual"
grep -q 'propose-skill-update' "$ROOT/shared/scripts/evaluate-session.sh" && fail "evaluate-session.sh calls propose-skill-update.sh; the header correction is wrong"
grep -q 'called by `evaluate-session.sh`' "$ROOT/skills/process/pmpo-skill-creator/SKILL.md" && fail "pmpo-skill-creator SKILL.md repeats the false claim"
grep -q '^## Skill Candidates' "$INTEGRATION_DOC" || fail "integration doc lacks the skill candidate flow"
grep -q 'pk candidates accept --kind skill' "$INTEGRATION_DOC" || fail "integration doc does not name the accept command"
grep -q 'not\*\* part of this flow' "$INTEGRATION_DOC" || fail "integration doc does not correct the propose-skill-update claim"
ok "propose-skill-update.sh is documented as the manual entry point; integration doc describes candidates"

# --- 1/2. kbd-open with the real pk -------------------------------------------
if [ -x "$PK_BIN" ] && version_ok "$PK_BIN"; then
  PKDIR="$(dirname "$PK_BIN")"
  out="$(snapshot "$PKDIR:$BASEPATH")"; rc=$?
  [ "$rc" = 0 ] || fail "kbd-open exited $rc with pk $("$PK_BIN" --version)"
  printf '%s' "$out" | grep -q 'Skill candidates' && fail "section printed with no pending skill candidate"
  ok "pk >= 1.11.0 with nothing pending: silent"

  CAND="$HOME/.prometheus/skill-candidates/pending"; mkdir -p "$CAND"
  EVID='{"workflowId":"w1","projectId":"project:alpha","projectRoot":"/alpha","sessionId":"s1","recordedAt":"2026-10-04T00:00:00Z","similarity":1.0,"corrections":[]}'
  cat > "$CAND/skill-new-c3b0000000000001.json" <<JSON
{"schemaVersion":1,"id":"skill-new-c3b0000000000001","kind":"skill","candidateType":"new-skill","state":"pending","reasons":["recurrence"],
 "title":"New skill: deploy the staging gateway","summary":"deploy the staging gateway","sessionCount":3,"projectCount":2,"invokedSkills":[],
 "evidence":[$EVID,$EVID,$EVID],"createdAt":"2026-10-04T00:00:00Z","updatedAt":"2026-10-04T00:00:00Z"}
JSON
  cat > "$CAND/skill-upd-c3b0000000000002.json" <<JSON
{"schemaVersion":1,"id":"skill-upd-c3b0000000000002","kind":"skill","candidateType":"skill-update","state":"pending","skillName":"kbd-reflect",
 "teamId":null,"roleId":null,"reasons":["post-skill-corrections"],"title":"Update kbd-reflect: 2 correction(s) after use by an unattributed agent",
 "evidence":[$EVID,$EVID],"createdAt":"2026-10-04T00:00:01Z","updatedAt":"2026-10-04T00:00:01Z"}
JSON
  out="$(snapshot "$PKDIR:$BASEPATH")"; rc=$?
  [ "$rc" = 0 ] || fail "kbd-open exited $rc with pending skill candidates"
  printf '%s' "$out" | grep -q '## Skill candidates awaiting review (2)' || fail "both candidates not counted: $out"
  printf '%s' "$out" | grep -q 'skill-new-c3b0000000000001' || fail "new-skill candidate lacks its full id"
  printf '%s' "$out" | grep -q 'skill-upd-c3b0000000000002' || fail "skill-update candidate lacks its full id"
  printf '%s' "$out" | grep -q 'deploy the staging gateway' || fail "new-skill title missing"
  printf '%s' "$out" | grep -q '3 evidence, new-skill' || fail "new-skill line lacks evidence count and type"
  printf '%s' "$out" | grep -q '2 evidence, skill-update' || fail "update line lacks evidence count and type"
  printf '%s' "$out" | grep -q 'Promotion candidates' && fail "skill candidates leaked into the promotion section"
  ok "kbd-open lists the new-skill and the skill-update candidate with full ids"

  acc="$("$PK_BIN" candidates accept --kind skill skill-new-c3b0000000000001 2>&1)" || fail "pk candidates accept --kind skill failed: $acc"
  printf '%s' "$acc" | grep -q '/pmpo-skill-creator' || fail "accept did not print a /pmpo-skill-creator invocation: $acc"
  [ -f "$HOME/.prometheus/skill-candidates/accepted/skill-new-c3b0000000000001.json" ] || fail "accepted candidate not moved to accepted/"
  [ ! -e "$HOME/.claude/skills" ] && [ ! -e "$CODEX_HOME/skills" ] || fail "accept created a skill directory"
  acc="$("$PK_BIN" candidates accept --kind skill skill-upd-c3b0000000000002 --update kbd-reflect 2>&1)" || fail "accept --update failed: $acc"
  printf '%s' "$acc" | grep -q -- '/pmpo-skill-creator --update kbd-reflect' || fail "accept --update did not print the update invocation: $acc"
  out="$(snapshot "$PKDIR:$BASEPATH")"
  printf '%s' "$out" | grep -q 'Skill candidates' && fail "accepted candidates are still listed"
  ok "accept prints the pmpo-skill-creator invocation, moves the candidate, creates no skill; the section clears"

  cat > "$CAND/skill-new-c3b0000000000003.json" <<JSON
{"schemaVersion":1,"id":"skill-new-c3b0000000000003","kind":"skill","candidateType":"new-skill","state":"pending","reasons":["recurrence"],
 "title":"New skill: rotate keys","summary":"rotate keys","evidence":[$EVID],"createdAt":"2026-10-04T00:00:02Z","updatedAt":"2026-10-04T00:00:02Z"}
JSON
  out="$(snapshot "$PKDIR:$BASEPATH")"
  printf '%s' "$out" | grep -q 'skill-new-c3b0000000000003' || fail "third candidate not listed"
  "$PK_BIN" candidates reject --kind skill skill-new-c3b0000000000003 --reason test >/dev/null 2>&1 || fail "pk candidates reject failed"
  out="$(snapshot "$PKDIR:$BASEPATH")"
  printf '%s' "$out" | grep -q 'Skill candidates' && fail "rejected candidate is still listed"
  ok "after a human rejects it the candidate leaves the section"
  echo "PASS: $pass checks"; exit 0
fi
echo "BLOCKED: pk >= 1.11.0 not found (set TLI_PK_BIN); candidate-listing checks not run ($pass checks passed)" >&2
exit 2
