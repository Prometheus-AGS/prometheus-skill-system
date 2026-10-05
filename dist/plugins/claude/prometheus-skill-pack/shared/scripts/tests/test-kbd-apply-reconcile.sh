#!/bin/bash
# test-kbd-apply-reconcile.sh — integration test for kbd-apply `mark-done` ledger sync
# and `reconcile` (detect + --repair) against the INSTALLED `prometheus kbd` runtime.
#
# Everything happens in a scratch project root (mktemp -d) with a scratch HOME,
# PROMETHEUS_DATA_DIR and a throwaway device key. It never touches this repository's
# .kbd-orchestrator or the operator's ~/.prometheus. kbd-apply is the real script,
# the backend is native-kbd (tasks.json), the ledger is the real signed runtime.
# Exit 0 pass, 1 fail, 2 BLOCKED. bash 3.2.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
APPLY="$ROOT/skills/process/kbd-process-orchestrator/skills/kbd-apply/kbd-apply.sh"
blocked() { echo "BLOCKED: $*" >&2; exit 2; }
[ -f "$APPLY" ] || blocked "kbd-apply.sh missing"
command -v prometheus >/dev/null 2>&1 || blocked "prometheus CLI not on PATH"
command -v jq >/dev/null 2>&1 || blocked "jq missing"
command -v node >/dev/null 2>&1 || blocked "node missing (device key generation)"
command -v git >/dev/null 2>&1 || blocked "git missing"
[ -x /bin/bash ] || blocked "/bin/bash missing"

S="$(mktemp -d)"
trap 'rm -rf "$S"' EXIT
export HOME="$S/home"; mkdir -p "$HOME"
export PROMETHEUS_DATA_DIR="$S/data"; mkdir -p "$PROMETHEUS_DATA_DIR"
export PROMETHEUS_DEVICE_KEY_FILE="$S/device-key.json"
export PROMETHEUS_KBD_CONTROL_PLANE=0 PROMETHEUS_CONTROL_ENDPOINT="http://127.0.0.1:1"
export PROMETHEUS_HARNESS="kbd-reconcile-test" OPENSPEC_TELEMETRY=0 DO_NOT_TRACK=1
unset KBD_ORCHESTRATOR_ROOT CLAUDE_PLUGIN_ROOT PROMETHEUS_PLUGIN_ROOT
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }

# A private, fresh signer: the canonical CLI cannot consult the host keychain here.
node -e '
const {generateKeyPairSync,createHash}=require("crypto");const fs=require("fs");
const {privateKey,publicKey}=generateKeyPairSync("ed25519");
const pr=privateKey.export({format:"jwk"}),pu=publicKey.export({format:"jwk"});
fs.writeFileSync(process.argv[1],JSON.stringify({schemaVersion:"1",
 keyId:"ed25519:"+createHash("sha256").update(Buffer.from(pu.x,"base64url")).digest("hex"),
 privateKey:Buffer.from(pr.d,"base64url").toString("base64")}),{mode:0o600});' "$PROMETHEUS_DEVICE_KEY_FILE" \
  || fail "could not generate the scratch device key"

P="$S/project"; mkdir -p "$P/.kbd-orchestrator"
cd "$P" || exit 1
git init -q . || exit 1   # CLI project discovery walks ancestors: make the scratch root the boundary
case "$(pwd -P)" in "$(cd "$S" && pwd -P)"/*) ;; *) fail "scratch project is not under the scratch dir" ;; esac

k() { prometheus kbd --path . "$@"; }
need() { "$@" >/dev/null 2>"$S/err" || fail "$* -> $(tail -3 "$S/err")"; }
apply() { /bin/bash "$APPLY" "$@"; }

# --- bootstrap: runtime phase + native-kbd change with 3 tasks, all registered in the ledger
need k status --json
need k phase create --command-id boot-create --id phase-rec --title phase-rec
need k phase activate --command-id boot-activate --id phase-rec --exact-next-work "/kbd-execute phase-rec"
need k phase transition --command-id boot-start --id phase-rec --status in-progress
CH=change-rec
mkdir -p ".kbd-orchestrator/changes/$CH"
echo "# $CH" > ".kbd-orchestrator/changes/$CH/spec.md"
cat > ".kbd-orchestrator/changes/$CH/tasks.json" <<JSON
{"changeId":"$CH","schemaVersion":"1","tasks":[
 {"id":"1","title":"first task","done":false,"doneAt":null,"doneBy":null},
 {"id":"2","title":"second task","done":false,"doneAt":null,"doneBy":null},
 {"id":"3","title":"third task","done":false,"doneAt":null,"doneBy":null}]}
JSON
need k change register --command-id boot-change --phase phase-rec --id "$CH" --title "$CH"
for n in 1 2 3; do
  title="$(jq -r --arg n "$n" '.tasks[]|select(.id==$n)|.title' ".kbd-orchestrator/changes/$CH/tasks.json")"
  need k task register --command-id "boot-task-$n" --phase phase-rec --change "$CH" --id "$n" --title "$title" --sequence "$n"
done
[ "$(jq -r .generatedBy .kbd-orchestrator/current-waypoint.json)" = "kbd-runtime" ] || fail "runtime is not authoritative in the scratch project"
tasks_done() { jq -r --arg c "$CH" '[.changes[]|select(.id==$c)|.tasks_done][0]' .kbd-orchestrator/phases/phase-rec/progress.json; }
task_status() { k status --json | jq -r --arg c "$CH" --arg t "$1" '.phases["phase-rec"].changes[$c].tasks[$t].status'; }

# --- 1. clean baseline
apply reconcile phase-rec >"$S/out" 2>&1; rc=$?
[ "$rc" -eq 0 ] || fail "fresh project: reconcile exited $rc: $(cat "$S/out")"
ok "a project with no drift reconciles clean (exit 0)"

# --- 2. drift: flip task 1 done in tasks.json only, bypassing the ledger
tmp="$S/tasks.tmp"
jq '(.tasks[]|select(.id=="1")) |= (.done=true|.doneAt="2026-01-01T00:00:00Z"|.doneBy="test")' ".kbd-orchestrator/changes/$CH/tasks.json" > "$tmp" \
  && mv "$tmp" ".kbd-orchestrator/changes/$CH/tasks.json" || fail "could not flip task 1"
REV_BEFORE="$(k status --json | jq -r .revision)"
apply reconcile phase-rec >"$S/out" 2>"$S/err"; rc=$?
[ "$rc" -eq 1 ] || fail "drift: reconcile exited $rc, expected 1: $(cat "$S/out" "$S/err")"
grep -q "$CH" "$S/out" && grep -q "task 1" "$S/out" || fail "drift output does not name the change and task: $(cat "$S/out")"
[ "$(grep -c '^DRIFT' "$S/out")" -eq 1 ] || fail "expected exactly one drifted task line: $(cat "$S/out")"
[ "$(k status --json | jq -r .revision)" = "$REV_BEFORE" ] || fail "detection wrote to the ledger"
apply reconcile phase-rec --json 2>/dev/null | jq -e --arg c "$CH" '.clean==false and .drifted==1 and .drift[0].change==$c and .drift[0].task=="1"' >/dev/null \
  || fail "--json drift report malformed"
ok "task flipped done without the ledger: exit 1, names $CH task 1, read-only, --json agrees"

# --- 3. repair
apply reconcile phase-rec --repair >"$S/out" 2>"$S/err"; rc=$?
[ "$rc" -eq 0 ] || fail "--repair exited $rc: $(cat "$S/out" "$S/err")"
apply reconcile phase-rec >"$S/out" 2>&1; rc=$?
[ "$rc" -eq 0 ] || fail "after --repair, reconcile exited $rc: $(cat "$S/out")"
[ "$(task_status 1)" = "complete" ] || fail "task 1 is '$(task_status 1)' in the ledger after repair"
[ "$(tasks_done)" = "1" ] || fail "progress.json counts $(tasks_done) done after repair, expected 1"
ok "--repair replays the task; next reconcile exits 0; ledger complete and progress.json counts it"

# --- 4. mark-done leaves no drift
apply mark-done "$CH" 2 >"$S/out" 2>"$S/err" || fail "mark-done failed: $(cat "$S/out" "$S/err")"
grep -qi "hooks were not fired" "$S/out" || fail "mark-done did not say hooks were not fired: $(cat "$S/out")"
apply reconcile phase-rec >"$S/out" 2>&1; rc=$?
[ "$rc" -eq 0 ] || fail "after mark-done, reconcile exited $rc: $(cat "$S/out")"
[ "$(task_status 2)" = "complete" ] || fail "mark-done left task 2 '$(task_status 2)' in the ledger"
[ "$(tasks_done)" = "2" ] || fail "progress.json counts $(tasks_done) after mark-done, expected 2"
ok "mark-done syncs the ledger and progress.json; reconcile exits 0 and notes hooks were not fired"

# --- 5. default phase = active phase
apply reconcile >"$S/out" 2>&1; rc=$?
[ "$rc" -eq 0 ] || fail "reconcile with no phase exited $rc: $(cat "$S/out")"
ok "reconcile without a phase uses the active phase"

echo "all $pass checks passed"
