#!/bin/bash
# test-cadence-refresh-procedure.sh — integration test for the delivery-cadence
# skill-pack refresh procedure (skills/process/delivery-cadence/scripts/refresh-skill-pack.sh).
#
# The procedure runs as a real process under /bin/bash (3.2) against a scratch git
# deploy worktree that has a real submodule and a real origin. Only the install,
# update and kickstart steps are replaced by recording stubs (REFRESH_TEST_MODE=1);
# the machine is never refreshed. Scratch HOME; nothing real is touched.
# Exit 0 pass, 1 fail, 2 BLOCKED. bash 3.2.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
PROC="$ROOT/skills/process/delivery-cadence/scripts/refresh-skill-pack.sh"
blocked() { echo "BLOCKED: $*" >&2; exit 2; }
command -v git >/dev/null 2>&1 || blocked "git missing"
command -v python3 >/dev/null 2>&1 || blocked "python3 missing"
[ -f "$PROC" ] || blocked "refresh-skill-pack.sh missing"
[ -x /bin/bash ] || blocked "/bin/bash missing"

S="$(mktemp -d)"
trap 'chmod -R u+rw "$S" 2>/dev/null; rm -rf "$S"' EXIT
export HOME="$S/home"; mkdir -p "$HOME"
export GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null
export GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=protocol.file.allow GIT_CONFIG_VALUE_0=always
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@example.test GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@example.test
export REFRESH_TEST_MODE=1 REFRESH_HEALTH_URL="http://127.0.0.1:9/health"
export STUBLOG="$S/stub.log"
export REFRESH_UPDATE_CMD='echo "update $(git rev-parse HEAD)" >> "$STUBLOG"'
export REFRESH_INSTALL_CMD='echo "install $(git rev-parse HEAD)" >> "$STUBLOG"'
export REFRESH_KICKSTART_CMD='echo "kickstart $1" >> "$STUBLOG"'
: > "$STUBLOG"

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }
g() { git -C "$1" "${@:2}"; }

# --- fixtures: submodule repo, upstream repo pinning it, bare origin, deploy clone
SUB="$S/sub"; UP="$S/up"; ORIGIN="$S/origin.git"; DEPLOY="$S/deploy"
git init -q "$SUB" -b main && echo a > "$SUB/f" && g "$SUB" add f && g "$SUB" commit -qm a || exit 1
SUB_A="$(g "$SUB" rev-parse HEAD)"
git init -q "$UP" -b main && g "$UP" submodule add -q "$SUB" mod && g "$UP" commit -qm base || exit 1
git clone -q --bare "$UP" "$ORIGIN" && g "$UP" remote add origin "$ORIGIN" && g "$UP" fetch -q origin || exit 1
git clone -q "$ORIGIN" "$DEPLOY" && g "$DEPLOY" submodule update -q --init || exit 1
# upstream advances: submodule pin moves to a new commit
echo b >> "$SUB/f" && g "$SUB" commit -qam b && SUB_B="$(g "$SUB" rev-parse HEAD)"
g "$UP" submodule update -q --init --remote 2>/dev/null || exit 1
g "$UP" add mod && echo x > "$UP/new" && g "$UP" add new && g "$UP" commit -qm advance && g "$UP" push -q origin main || exit 1
ORIGIN_HEAD="$(g "$UP" rev-parse HEAD)"
BASE_HEAD="$(g "$DEPLOY" rev-parse HEAD)"
[ "$BASE_HEAD" != "$ORIGIN_HEAD" ] || fail "fixture: deploy should start behind origin"

state() { printf '%s' "$2" > "$S/$1.json"; }
state s7 '{"activeIterationId":"i7","iterations":[{"id":"i6","index":6},{"id":"i7","index":7}]}'
state s8 '{"activeIterationId":"i8","iterations":[{"id":"i7","index":7},{"id":"i8","index":8}]}'
state garbage '{"activeIterationId":"i1","iterations":[{"id":"i1","index":"seven"}]}'
state noactive '{"activeIterationId":null,"iterations":[{"id":"i1","index":1}]}'
printf 'not json' > "$S/broken.json"
cp "$S/s7.json" "$S/unreadable.json"; chmod 000 "$S/unreadable.json"

run() { /bin/bash "$PROC" --deploy "$DEPLOY" --services "svc.one,svc.two" "$@"; }
lines() { wc -l < "$STUBLOG" | tr -d ' '; }

# --- bad input: exit 2 and no work
for st in "$S/missing.json" "$S/garbage.json" "$S/broken.json" "$S/noactive.json"; do
  : > "$STUBLOG"
  run --mode auto --state "$st" >/dev/null 2>"$S/err"; rc=$?
  [ "$rc" -eq 2 ] || fail "auto with $(basename "$st") exited $rc, expected 2"
  [ -s "$S/err" ] || fail "no message for $(basename "$st")"
  [ "$(lines)" -eq 0 ] || fail "work done for $(basename "$st")"
  [ "$(g "$DEPLOY" rev-parse HEAD)" = "$BASE_HEAD" ] || fail "deploy moved for $(basename "$st")"
done
if [ "$(id -u)" -ne 0 ]; then
  run --mode auto --state "$S/unreadable.json" >/dev/null 2>&1; rc=$?
  [ "$rc" -eq 2 ] || fail "unreadable state exited $rc, expected 2"
fi
run --mode auto >/dev/null 2>&1; rc=$?; [ "$rc" -eq 2 ] || fail "auto without --state exited $rc"
ok "missing, garbage, unreadable and inactive state exit 2 with no work"

# --- verify (iteration 8): nothing changes, summary has the keys
: > "$STUBLOG"
SUBHEAD_BEFORE="$(g "$DEPLOY/mod" rev-parse HEAD)"
OUT="$(run --mode auto --state "$S/s8.json" 2>"$S/err")" || fail "verify exited $? ($(cat "$S/err"))"
[ "$(lines)" -eq 0 ] || fail "verify invoked stubs"
[ "$(g "$DEPLOY" rev-parse HEAD)" = "$BASE_HEAD" ] || fail "verify moved HEAD"
[ "$(g "$DEPLOY/mod" rev-parse HEAD)" = "$SUBHEAD_BEFORE" ] || fail "verify moved submodule"
printf '%s' "$OUT" | python3 -c '
import json,sys
d=json.load(sys.stdin)
assert d["mode"]=="verify" and d["iteration"]==8, d
for k in ("sourceCommit","versions","health"): assert k in d, k
assert d["sourceCommit"]==sys.argv[1], d["sourceCommit"]
' "$BASE_HEAD" || fail "verify summary malformed: $OUT"
ok "iteration 8 selects verify; HEAD, submodule and stub log unchanged; summary keys present"

# --- dirty worktree: exit 1 before any install step
echo dirt > "$DEPLOY/untracked"
: > "$STUBLOG"
run --mode full >/dev/null 2>"$S/err"; rc=$?
[ "$rc" -eq 1 ] || fail "dirty worktree exited $rc, expected 1"
[ "$(lines)" -eq 0 ] || fail "dirty worktree reached install steps"
[ "$(g "$DEPLOY" rev-parse HEAD)" = "$BASE_HEAD" ] || fail "dirty worktree was moved"
rm -f "$DEPLOY/untracked"
ok "dirty worktree exits 1 before any step"

# --- diverged worktree: exit 1
g "$DEPLOY" checkout -q -b local-div && echo l > "$DEPLOY/local" && g "$DEPLOY" add local && g "$DEPLOY" commit -qm local || exit 1
: > "$STUBLOG"
run --mode full >/dev/null 2>"$S/err"; rc=$?
[ "$rc" -eq 1 ] || fail "diverged worktree exited $rc, expected 1"
[ "$(lines)" -eq 0 ] || fail "diverged worktree reached install steps"
g "$DEPLOY" reset -q --hard "$BASE_HEAD" && g "$DEPLOY" checkout -q main && g "$DEPLOY" branch -qD local-div
ok "diverged worktree exits 1 before any step"

# --- full (iteration 7): fast-forward, submodule at the recorded commit, stubs in order
: > "$STUBLOG"
OUT="$(run --mode auto --state "$S/s7.json" 2>"$S/err")" || fail "full exited $? ($(cat "$S/err"))"
[ "$(g "$DEPLOY" rev-parse HEAD)" = "$ORIGIN_HEAD" ] || fail "deploy not fast-forwarded"
[ "$(g "$DEPLOY/mod" rev-parse HEAD)" = "$SUB_B" ] || fail "submodule not at recorded commit"
[ -z "$(g "$DEPLOY" status --porcelain)" ] || fail "deploy worktree not clean after full refresh"
EXPECT="update $ORIGIN_HEAD
install $ORIGIN_HEAD
kickstart svc.one
kickstart svc.two"
[ "$(cat "$STUBLOG")" = "$EXPECT" ] || fail "stub calls differ: $(cat "$STUBLOG")"
printf '%s' "$OUT" | python3 -c '
import json,sys
d=json.load(sys.stdin)
assert d["mode"]=="full" and d["iteration"]==7, d
for k in ("sourceCommit","versions","health"): assert k in d, k
assert d["sourceCommit"]==sys.argv[1]
' "$ORIGIN_HEAD" || fail "full summary malformed: $OUT"
[ "$SUB_A" != "$SUB_B" ] || fail "fixture: submodule pins should differ"
ok "iteration 7 selects full; deploy fast-forwarded, submodule at recorded commit, clean status"

# --- a failing step is loud
export REFRESH_INSTALL_CMD='exit 3'
: > "$STUBLOG"
run --mode full >/dev/null 2>"$S/err"; rc=$?
[ "$rc" -eq 1 ] || fail "failing install exited $rc, expected 1"
[ "$(lines)" -eq 1 ] || fail "kickstart ran after a failed install"
ok "a failed install exits 1 and skips kickstart"

echo "all $pass checks passed"
