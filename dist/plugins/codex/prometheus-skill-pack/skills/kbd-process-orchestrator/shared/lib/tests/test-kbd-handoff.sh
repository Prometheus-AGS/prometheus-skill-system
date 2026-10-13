#!/usr/bin/env bash
# shared/lib/tests/test-kbd-handoff.sh — lease-free cross-harness handoff packet.
#
# Runs the real kbd-handoff.sh against a sandbox project (git repo, waypoint,
# progress, plan, AGENTS.md, agent team) and a sandbox HOME with installed
# kbd-* skills. `prometheus` is a PATH fixture that prints a canned
# `kbd status --json` document, so boundary obligations are deterministic.
set -euo pipefail
cd "$(dirname "$0")"
SKILL_ROOT="$(cd ../../.. && pwd -P)"
REPO_ROOT="$(cd "$SKILL_ROOT/../../.." && pwd -P)"
HANDOFF="$SKILL_ROOT/skills/kbd-handoff/kbd-handoff.sh"
SCHEMA="$SKILL_ROOT/references/schemas/harness-handoff.schema.json"

pass() { printf 'pass: %s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || fail "jq required"
command -v git >/dev/null 2>&1 || fail "git required"
[ -x "$HANDOFF" ] || fail "kbd-handoff.sh missing or not executable"

SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT
PROJ="$SANDBOX/proj"
export HOME="$SANDBOX/home"
export PATH="$SANDBOX/bin:$PATH"
export KBD_FAKE_STATUS="$SANDBOX/status.json"
export PROMETHEUS_HARNESS=claude-code
unset KBD_ORCHESTRATOR_ROOT || true

# --- fixtures ---------------------------------------------------------------
mkdir -p "$SANDBOX/bin" "$PROJ/.kbd-orchestrator/phases/p1" "$PROJ/.agent-team/team-a" \
  "$PROJ/.kimi-code/agents" "$PROJ/.prometheus" "$PROJ/openspec/changes/c1"
printf '%s\n' '#!/usr/bin/env bash' \
  '[ "${KBD_FAKE_FAIL:-0}" = 1 ] && exit 1' \
  'case " $* " in *" status "*) cat "$KBD_FAKE_STATUS" ;; *) exit 64 ;; esac' >"$SANDBOX/bin/prometheus"
chmod +x "$SANDBOX/bin/prometheus"

jq -n '{schemaVersion:"5", generatedBy:"kbd-runtime", path:["p1"], activePhaseId:"p1", phase:"p1",
        stageId:"execute", stageStatus:"IN_PROGRESS", lifecycle:"running", revision:41, planRevision:2,
        nextChange:"c1", nextTask:"1.2", exactNextCommand:"/kbd-execute p1"}' \
  >"$PROJ/.kbd-orchestrator/current-waypoint.json"
printf 'POSITION REMINDER\nNext work: change c1, task 1.2 (derived from task state)\n' \
  >"$PROJ/.kbd-orchestrator/position-reminder.txt"
jq -n '{schemaVersion:"2", phase:"p1",
        completion:{implementation:{completed:1,total:3,status:"IN_PROGRESS"}},
        changes:[{id:"c0",status:"COMPLETE"},
                 {id:"c1",status:"IN_PROGRESS",next_task_pending:"Wire the settings route"},
                 {id:"c2",status:"PENDING"}]}' \
  >"$PROJ/.kbd-orchestrator/phases/p1/progress.json"
printf '%s\n' '# Plan p1' '' '## 1. Changes' '' '- c0, c1, c2' '' '## 2. Operator gates' '' \
  '- OG-1: confirm the storage decision before c2.' '' '## 3. Rounds' '' '- one round' \
  >"$PROJ/.kbd-orchestrator/phases/p1/plan.md"
printf '# Assessment p1\n' >"$PROJ/.kbd-orchestrator/phases/p1/assessment.md"
printf '%s\n' '# AGENTS.md' '' '## Stability: What You Must Not Break' '' \
  '- **Never break the public API.** Thousands of clients depend on it.' \
  '- Do not change the default hash. It forks the address space.' '' \
  '## Go Style' '' '- prefer the standard library' >"$PROJ/AGENTS.md"
printf '# Gotchas\n' >"$PROJ/.prometheus/gotchas.md"
printf '# c1\n' >"$PROJ/openspec/changes/c1/proposal.md"
jq -n '{schemaVersion:1, manifest:".agent-team/team-a/team.json"}' >"$PROJ/.agent-team/project-routing.json"
jq -n '{id:"team-a", roles:[{id:"api-builder"},{id:"reviewer"}]}' >"$PROJ/.agent-team/team-a/team.json"
printf '# api-builder\n' >"$PROJ/.kimi-code/agents/api-builder.md"
printf '# reviewer\n' >"$PROJ/.kimi-code/agents/reviewer.md"

git -C "$PROJ" init -q
git -C "$PROJ" -c user.name=fixture -c user.email=fixture@example.invalid add -A
git -C "$PROJ" -c user.name=fixture -c user.email=fixture@example.invalid commit -q -m fixture

# Installed kimi-code skills: three valid, plus a kbd-handoff link to a removed path.
for s in kbd-apply kbd-status kbd-process-orchestrator; do
  mkdir -p "$HOME/.kimi-code/skills/$s"
  printf -- '---\nname: %s\n---\n' "$s" >"$HOME/.kimi-code/skills/$s/SKILL.md"
done
ln -s "$SANDBOX/removed/skills/kbd-handoff" "$HOME/.kimi-code/skills/kbd-handoff"

status_with() { # <boundaryObligations json>
  jq -n --argjson ob "$1" '{revision:41, planRevision:2, lifecycle:"running",
    activePath:{phaseId:"p1", stageId:"execute", changeId:"c1", taskId:"1.2"},
    exactNextWork:"/kbd-execute p1", boundaryObligations:$ob,
    phases:{p1:{id:"p1", status:"in_progress", changes:{
              c0:{id:"c0", status:"complete"}, c1:{id:"c1", status:"in_progress"}, c2:{id:"c2", status:"pending"}}},
            p0:{id:"p0", status:"pending", changes:{old:{id:"old", status:"pending"}}},
            pz:{id:"pz", status:"complete", changes:{}}},
    completion:{implementation:{completed:7,total:20}},
    blockers:{b1:{id:"b1", summary:"Waiting on OG-1", resolved:false},
              b2:{id:"b2", summary:"old", resolved:true}}, decisions:{}}' >"$KBD_FAKE_STATUS"
}
state_sum() {
  cat "$PROJ/.kbd-orchestrator/current-waypoint.json" "$PROJ/.kbd-orchestrator/phases/p1/progress.json" \
    "$PROJ/.kbd-orchestrator/position-reminder.txt" | cksum
}
OUT="$PROJ/.kbd-orchestrator/phases/p1/harness-handoffs"
BEFORE="$(state_sum)"
cd "$PROJ"

# --- Test 1: unknown harness is rejected ------------------------------------
status_with '{}'
set +e; err="$("$HANDOFF" --to notepad </dev/null 2>&1 >/dev/null)"; rc=$?; set -e
[ "$rc" = 2 ] || fail "unknown harness: expected exit 2, got $rc"
printf '%s' "$err" | grep -q "unknown harness 'notepad'" || fail "unknown harness: message missing: $err"
printf '%s' "$err" | grep -q "kimi-code" || fail "unknown harness: supported list missing"
set +e; "$HANDOFF" </dev/null >/dev/null 2>&1; rc=$?; set -e
[ "$rc" = 2 ] || fail "missing --to: expected exit 2, got $rc"
[ ! -e "$OUT" ] || fail "unknown harness wrote $OUT"
pass "unknown or missing harness exits 2 and writes nothing"

# --- Test 2: half-closed boundaries refuse (exit 1) ------------------------
refuses() { # <label> <obligations json> <expected repair fragment>
  status_with "$2"
  set +e; err="$("$HANDOFF" --to kimi-code </dev/null 2>&1 >/dev/null)"; rc=$?; set -e
  [ "$rc" = 1 ] || fail "$1: expected exit 1, got $rc: $err"
  printf '%s' "$err" | grep -q -- "$3" || fail "$1: repair command missing: $err"
  [ ! -e "$OUT" ] || fail "$1: refusal wrote $OUT"
  [ "$(state_sum)" = "$BEFORE" ] || fail "$1: refusal changed KBD state"
}
refuses "open task" '{"task:c1/1.1":{"boundary":"task","subject":"c1/1.1","exactSignal":"Starting task 1 out of 4"}}' \
  "guard evaluate --boundary task --edge after --subject c1/1.1 --json --repair-projections"
refuses "completed change" '{"change:c0":{"boundary":"change","subject":"c0","exactSignal":"Starting change 1 out of 3: c0"}}' \
  "guard evaluate --boundary change --edge after --subject c0 --json --repair-projections"
refuses "unknown change" '{"change:ghost":{"boundary":"change","subject":"ghost"}}' "--boundary change --edge after --subject ghost"
refuses "completed phase" '{"phase:pz":{"boundary":"phase","subject":"pz"}}' "--boundary phase --edge after --subject pz"
pass "open task, complete or unknown change/phase obligations refuse with exit 1 and repair commands"

# Active phase and the active in-progress change are ignored; a pending change
# of another phase is carried work (warned, not refused).
status_with '{"phase:p1":{"boundary":"phase","subject":"p1","exactSignal":"Starting phase 1 out of 1: p1"},
              "change:c1":{"boundary":"change","subject":"c1","exactSignal":"Starting change 2 out of 3: c1"},
              "change:old":{"boundary":"change","subject":"old","exactSignal":"Starting change 4 out of 9: old"}}'

# --- Test 3: dry run writes nothing -----------------------------------------
set +e; out="$("$HANDOFF" --to kimi-code --dry-run </dev/null 2>/dev/null)"; rc=$?; set -e
[ "$rc" = 0 ] || fail "dry run: expected exit 0, got $rc"
printf '%s' "$out" | grep -q '^Starting kbd-handoff — p1 to kimi-code$' || fail "dry run: Starting signal missing"
printf '%s' "$out" | grep -q '^# Handoff to kimi-code: p1$' || fail "dry run: packet not printed"
printf '%s' "$out" | grep -q '^Completed kbd-handoff — p1 dry run (no packet written)$' || fail "dry run: Completed signal missing"
[ ! -e "$OUT" ] || fail "dry run created $OUT"
[ "$(state_sum)" = "$BEFORE" ] || fail "dry run changed KBD state"
[ -z "$(git status --porcelain)" ] || fail "dry run left files: $(git status --porcelain)"
pass "dry run prints the packet and writes nothing"

# --- Test 4: packet written with required sections + sidecar ----------------
set +e; out="$("$HANDOFF" --to kimi --repo "$PROJ" </dev/null 2>"$SANDBOX/err.txt")"; rc=$?; set -e
[ "$rc" = 0 ] || fail "packet: expected exit 0, got $rc: $(cat "$SANDBOX/err.txt")"
md="$(ls "$OUT"/kimi-code-*.md 2>/dev/null)"
js="$(ls "$OUT"/kimi-code-*.json 2>/dev/null)"
[ -n "$md" ] && [ "$(printf '%s\n' "$md" | wc -l | tr -d ' ')" = 1 ] || fail "packet: expected one .md, got: $md"
[ -n "$js" ] && [ "${md%.md}" = "${js%.json}" ] || fail "packet: sidecar missing or misnamed: $js"
case "$(basename "$md")" in kimi-code-[0-9]*T[0-9]*Z.md) ;; *) fail "packet name not <harness>-<UTC>.md: $md" ;; esac
[ ! -e "$PROJ/.kbd-orchestrator/phases/p1/handoffs" ] || fail "packet touched the stage handoffs/ directory"
for heading in '## Position' '## Repositories' '## Read in order' '## Pending operator gates and decisions' \
  '## Task execution rules' '## Never do' '## Target harness readiness' '## Open prerequisites' '## Start prompt'; do
  grep -qx "$heading" "$md" || fail "packet: section '$heading' missing"
done
grep -q 'Implementation (this phase, `completion.implementation`): 1 of 3' "$md" || fail "packet: N of T wrong"
grep -q 'Next: change `c1`, task 1.2' "$md" || fail "packet: next change/task wrong"
grep -q 'Wire the settings route' "$md" || fail "packet: next task text missing"
grep -q 'OG-1: confirm the storage decision' "$md" || fail "packet: operator gate excerpt missing"
grep -q 'Waiting on OG-1' "$md" || fail "packet: unresolved blocker missing"
grep -q -- '- Never break the public API.' "$md" || fail "packet: AGENTS.md hard rule not quoted"
grep -q -- '- Do not change the default hash.' "$md" || fail "packet: AGENTS.md plain rule not quoted"
! grep -q 'prefer the standard library' "$md" || fail "packet: never-do list pulled a non-rule section"
grep -q 'kbd-apply/kbd-apply.sh begin-task' "$md" || fail "packet: kbd-apply rule missing"
grep -q '`openspec/changes/c1`' "$md" || fail "packet: next OpenSpec change not in read list"
grep -q 'no upstream' "$md" || fail "packet: unpushed state not reported"
grep -q 'carried work: change old was started and not finished' "$md" || fail "packet: carried work not reported"
! grep -q 'carried work: .*c1' "$md" || fail "packet: active change reported as carried work"
printf '%s' "$out" | grep -q "^Completed kbd-handoff — p1 packet .kbd-orchestrator/phases/p1/harness-handoffs/kimi-code-" \
  || fail "packet: Completed signal missing: $out"
printf '%s' "$out" | grep -q 'You are taking over KBD phase `p1`' || fail "packet: start prompt not printed"
[ "$(state_sum)" = "$BEFORE" ] || fail "packet run changed KBD state"
jq -e '.schemaVersion == "1" and .kind == "kbd-harness-handoff" and .harness == "kimi-code"
       and .fromHarness == "claude-code" and .statusVerified == true
       and .position.implementation == {completed: 1, total: 3}
       and .position.nextChange == "c1" and .position.nextTask == "1.2" and .position.revision == 41
       and (.readInOrder[0] | endswith(".md")) and (.readInOrder | index("AGENTS.md")) != null
       and (.repos | length) == 2 and .repos[0].clean == true and .repos[0].pushed == false
       and .targetChecks.teamDetected == true and .targetChecks.missingRoleDefinitions == []' "$js" >/dev/null \
  || fail "sidecar content wrong: $(jq -c . "$js")"
if node -e "require.resolve('ajv/dist/2020.js', {paths: [process.argv[1]]})" "$REPO_ROOT" >/dev/null 2>&1; then
  node -e '
    const [root, schemaPath, docPath] = process.argv.slice(1);
    const Ajv = require(require.resolve("ajv/dist/2020.js", { paths: [root] })).default;
    const fs = require("fs");
    const ajv = new Ajv({ allErrors: true, strict: false, formats: { "date-time": true } });
    const ok = ajv.validate(JSON.parse(fs.readFileSync(schemaPath, "utf8")), JSON.parse(fs.readFileSync(docPath, "utf8")));
    if (!ok) { console.error(JSON.stringify(ajv.errors)); process.exit(1); }
  ' "$REPO_ROOT" "$SCHEMA" "$js" || fail "sidecar does not validate against harness-handoff.schema.json"
  pass "sidecar validates against harness-handoff.schema.json"
else
  printf 'skip: ajv not installed under %s; schema validation not run\n' "$REPO_ROOT"
fi
pass "packet and sidecar written under harness-handoffs/ with every required section"

# --- Test 5: broken installed skill symlink is reported ---------------------
grep -q "kbd-handoff -> $SANDBOX/removed/skills/kbd-handoff" "$md" || fail "broken symlink not in packet"
jq -e --arg t "$SANDBOX/removed/skills/kbd-handoff" '.targetChecks.brokenSkills == ["kbd-handoff -> " + $t]' "$js" >/dev/null \
  || fail "broken symlink not in sidecar: $(jq -c .targetChecks "$js")"
grep -q "broken kbd-\* skills" "$SANDBOX/err.txt" || fail "broken symlink warning not printed"
pass "broken kbd-* skill symlink reported in packet, sidecar and warning"

# --- Test 6: missing native role definitions for the target are reported ----
mkdir -p "$HOME/.codex/skills"
set +e; out="$("$HANDOFF" --to codex --dry-run </dev/null 2>&1)"; rc=$?; set -e
[ "$rc" = 0 ] || fail "codex dry run: expected exit 0, got $rc"
printf '%s' "$out" | grep -q '.codex/agents/api_builder.toml' || fail "missing codex role definition not reported: $out"
pass "missing native role definitions reported for the target harness"

# --- Test 7: status failure falls back to projections ----------------------
set +e; out="$(KBD_FAKE_FAIL=1 "$HANDOFF" --to claude --dry-run </dev/null 2>&1)"; rc=$?; set -e
[ "$rc" = 0 ] || fail "status fallback: expected exit 0, got $rc"
printf '%s' "$out" | grep -q 'boundary obligations were NOT checked' || fail "status fallback: unchecked warning missing"
printf '%s' "$out" | grep -q 'Implementation (this phase, `completion.implementation`): 1 of 3' || fail "status fallback: projection counter missing"
pass "failed status falls back to projections and says obligations were unchecked"

printf '\nAll kbd-handoff tests passed.\n'
