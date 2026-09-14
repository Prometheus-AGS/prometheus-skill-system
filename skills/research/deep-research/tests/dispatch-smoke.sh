#!/usr/bin/env bash
# dispatch-smoke.sh — change-drt-001, task 1.
#
# Answers the question analyze decision D-03a left open: can this machine
# dispatch two research workers CONCURRENTLY, each seeing only its own brief?
# Every later change in this phase assumes it can, so the assumption is tested
# before anything is built on it.
#
# Two dispatch paths exist (analysis D-03). This script proves the one a shell
# can prove:
#
#   B  process workers  — a headless harness CLI child per thread, the path
#                         substrate/prometheus-research/src/job/daemon.rs
#                         already spawns. TESTED HERE.
#   A  in-session subagents — subagents inside a harness session. A bash script
#                         cannot dispatch one; only the harness can. NOT tested
#                         here, and this script says so rather than implying
#                         coverage it does not have.
#
# Modes:
#   (default)          fixture workers — proves the concurrency mechanism
#   --with-harness     additionally spawns two REAL harness children, if one is
#                      on PATH, proving the mechanism works with the actual
#                      binary the daemon uses
#
# Exit 0 only when every assertion holds. bash 3.2 compatible (C-05).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
FIXTURE="$HERE/fixtures/thread-smoke/worker.sh"
WITH_HARNESS=0
[ "${1:-}" = "--with-harness" ] && WITH_HARNESS=1

PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   - $1"; }
bad()  { FAIL=$((FAIL+1)); echo "  FAIL - $1" >&2; }
check(){ d="$1"; shift; if "$@" >/dev/null 2>&1; then ok "$d"; else bad "$d"; fi; }

TMP="$(mktemp -d "${TMPDIR:-/tmp}/dispatch-smoke-XXXXXX")"
trap 'rm -rf "$TMP"' EXIT
[ -x "$FIXTURE" ] || { echo "dispatch-smoke: fixture worker missing at $FIXTURE" >&2; exit 2; }

# --------------------------------------------------------------------------
echo "== 1. two fixture workers, dispatched concurrently =="
THREADS="$TMP/threads"; mkdir -p "$THREADS"
HOLD=2

# Each worker is given ONLY its own brief. SIBLING_TASK is deliberately never
# exported: if a worker reports it, the dispatch leaked sibling context.
THREAD_BRIEF_TASK="investigate t01 topic" bash "$FIXTURE" t01 "$THREADS" "$HOLD" &
P1=$!
THREAD_BRIEF_TASK="investigate t02 topic" bash "$FIXTURE" t02 "$THREADS" "$HOLD" &
P2=$!
wait "$P1"; R1=$?
wait "$P2"; R2=$?

check "both workers exit 0" test "$R1" -eq 0 -a "$R2" -eq 0
check "both workers report complete" bash -c 'test "$(cat "$1/t01/status")" = complete && test "$(cat "$1/t02/status")" = complete' _ "$THREADS"

S1=$(cat "$THREADS/t01/started_at"); E1=$(cat "$THREADS/t01/ended_at")
S2=$(cat "$THREADS/t02/started_at"); E2=$(cat "$THREADS/t02/ended_at")

# Concurrency: the intervals must intersect. Serial execution cannot produce
# an intersection when each worker holds its slot for HOLD seconds.
LATEST_START=$S1; [ "$S2" -gt "$LATEST_START" ] && LATEST_START=$S2
EARLIEST_END=$E1; [ "$E2" -lt "$EARLIEST_END" ] && EARLIEST_END=$E2
check "the two executions overlap in wall-clock time (concurrent, not serial)" \
  test "$LATEST_START" -lt "$EARLIEST_END"

# Cross-check: total elapsed is closer to one hold than to two.
SPAN_START=$S1; [ "$S2" -lt "$SPAN_START" ] && SPAN_START=$S2
SPAN_END=$E1;   [ "$E2" -gt "$SPAN_END" ]   && SPAN_END=$E2
TOTAL=$(( SPAN_END - SPAN_START ))
check "total elapsed (${TOTAL}s) is under a serial run (>= $((HOLD*2))s)" test "$TOTAL" -lt $(( HOLD * 2 ))

echo "== 2. each worker received only its own brief =="
check "t01 received its own task" grep -q 'brief_task=investigate t01 topic' "$THREADS/t01/received.txt"
check "t02 received its own task" grep -q 'brief_task=investigate t02 topic' "$THREADS/t02/received.txt"
check "t01 did not receive t02's task" bash -c '! grep -q "t02 topic" "$1/t01/received.txt"' _ "$THREADS"
check "t02 did not receive t01's task" bash -c '! grep -q "t01 topic" "$1/t02/received.txt"' _ "$THREADS"
check "no sibling context leaked through the environment" \
  bash -c 'grep -q "sibling_env_leak=none" "$1/t01/received.txt" && grep -q "sibling_env_leak=none" "$1/t02/received.txt"' _ "$THREADS"

# Scratch-state isolation, required by the spec scenario ("neither file contains
# the other's marker"). Threads share a package directory by design, so the
# claim under test is not "a sibling's file is unreadable" — it never is. It is
# that a worker handed only its own brief cannot NAME a peer, because nothing
# told it one exists.
check "neither worker can name a peer thread (no scratch state leaked)" \
  bash -c 'grep -qx none "$1/t01/peer_probe" && grep -qx none "$1/t02/peer_probe"' _ "$THREADS"
check "t01's marker file is its own" grep -qx "marker-t01" "$THREADS/t01/scratch.txt"
check "t02's marker file is its own" grep -qx "marker-t02" "$THREADS/t02/scratch.txt"
check "neither marker file contains the other's marker" \
  bash -c '! grep -q "marker-t02" "$1/t01/scratch.txt" && ! grep -q "marker-t01" "$1/t02/scratch.txt"' _ "$THREADS"

# Negative control: with a peer deliberately injected, the probe MUST trip.
# Without this, the isolation assertion could pass by never being exercised.
NEG="$TMP/neg"; mkdir -p "$NEG"
THREAD_BRIEF_TASK="negative control" THREAD_PEERS="t99" bash "$FIXTURE" t01 "$NEG" 0 >/dev/null 2>&1
check "the isolation probe is not vacuous: an injected peer trips it" \
  bash -c 'grep -q "^leaked:" "$1/t01/peer_probe"' _ "$NEG"

# --------------------------------------------------------------------------
echo "== 3. real harness children (the binary the daemon actually spawns) =="
HARNESS=""
for h in claude codex; do command -v "$h" >/dev/null 2>&1 && { HARNESS="$h"; break; }; done

if [ "$WITH_HARNESS" -eq 0 ]; then
  echo "  skip - not requested; re-run with --with-harness to spawn real children"
  echo "         (default mode proves the concurrency mechanism, not the binary)"
elif [ -z "$HARNESS" ]; then
  # --with-harness was explicitly requested and the gate cannot run. Exiting 0
  # here would report an unrun gate as a pass, which is the one thing the
  # verification rules forbid. Exit 2 == BLOCKED, distinct from 1 == failed.
  echo "  BLOCKED - no harness binary on PATH (looked for claude, then codex)." >&2
  echo "            --with-harness was requested, so this is BLOCKED, not skipped." >&2
  echo "            Install a harness or drop --with-harness to run fixtures only." >&2
  echo >&2
  echo "dispatch-smoke: $PASS passed, $FAIL failed, real-harness gate BLOCKED" >&2
  exit 2
else
  H="$TMP/harness"; mkdir -p "$H"
  # Mirrors daemon.rs:249-262: the same flags, so this proves the real path.
  spawn_one() {
    _tid="$1"; _out="$H/$_tid"; mkdir -p "$_out"
    date -u +%s > "$_out/started_at"
    # No `|| true`: a harness child that fails must fail the gate. Its exit
    # status is recorded and asserted below.
    _rc=0
    if [ "$HARNESS" = "claude" ]; then
      ( cd "$_out" && claude -p "Write exactly the word ${_tid} to a file named out.txt in the current directory, then stop." \
          --permission-mode bypassPermissions >"$_out/harness.log" 2>&1 ) || _rc=$?
    else
      ( cd "$_out" && codex exec --dangerously-bypass-hook-trust --full-auto \
          "Write exactly the word ${_tid} to a file named out.txt in the current directory, then stop." \
          >"$_out/harness.log" 2>&1 ) || _rc=$?
    fi
    printf '%s\n' "$_rc" > "$_out/exit_code"
    date -u +%s > "$_out/ended_at"
  }
  echo "  using harness: $HARNESS"
  spawn_one h01 & Q1=$!
  spawn_one h02 & Q2=$!
  W1=0; wait "$Q1" || W1=$?
  W2=0; wait "$Q2" || W2=$?

  check "both spawn wrappers returned cleanly" test "$W1" -eq 0 -a "$W2" -eq 0
  check "both harness children exited 0" \
    bash -c 'test "$(cat "$1/h01/exit_code")" -eq 0 && test "$(cat "$1/h02/exit_code")" -eq 0' _ "$H"
  check "both harness children produced a log" test -s "$H/h01/harness.log" -a -s "$H/h02/harness.log"
  check "both harness children wrote the file they were asked for" \
    bash -c 'test -s "$1/h01/out.txt" && test -s "$1/h02/out.txt"' _ "$H"
  check "each wrote its own id, not the other's" \
    bash -c 'grep -q h01 "$1/h01/out.txt" && grep -q h02 "$1/h02/out.txt" && ! grep -q h02 "$1/h01/out.txt"' _ "$H"
  HS1=$(cat "$H/h01/started_at"); HE1=$(cat "$H/h01/ended_at")
  HS2=$(cat "$H/h02/started_at"); HE2=$(cat "$H/h02/ended_at")
  HL=$HS1; [ "$HS2" -gt "$HL" ] && HL=$HS2
  HE=$HE1; [ "$HE2" -lt "$HE" ] && HE=$HE2
  check "two real harness children overlapped in wall-clock time" test "$HL" -lt "$HE"
fi

# --------------------------------------------------------------------------
echo
echo "dispatch-smoke: $PASS passed, $FAIL failed"
if [ "$FAIL" -eq 0 ]; then
  echo "VERDICT: concurrent dispatch of isolated workers is AVAILABLE via the process path (strategy B)."
  echo "         Strategy A (in-session subagents) is NOT covered by this script — only a harness can dispatch one."
else
  echo "VERDICT: concurrent dispatch FAILED. Analysis D-03a's fallback applies:" >&2
  echo "         strategy A is rejected, drt-007 becomes blocking, drt-003's director is re-scoped." >&2
fi
[ "$FAIL" -eq 0 ]
