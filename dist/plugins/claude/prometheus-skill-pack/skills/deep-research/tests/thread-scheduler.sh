#!/usr/bin/env bash
# thread-scheduler.sh — change-drt-002, task 4.
#
# Proves the scheduler's two load-bearing claims against the real binary and
# real child processes — no mock scheduler, no simulated concurrency:
#
#   1. The concurrency cap is enforced in CODE. Onyx states its "never more
#      than 3 in parallel" rule four times in prompt prose and enforces it
#      nowhere; a prompt is a request, not a bound. So the cap test
#      over-subscribes and reconstructs the true peak from the workers' own
#      timestamps, on the outside, rather than trusting the scheduler's count.
#
#   2. Every dispatch gets a ledger row. A ledger with silent holes cannot tell
#      a director what to re-dispatch, so complete/failed/timeout all appear.
#
# The timeout case carries a negative control: the hanging worker writes a
# SURVIVED marker after its sleep. If the scheduler failed to kill it, that
# marker appears and the test fails — otherwise "it timed out" could be
# reported by a scheduler that merely stopped waiting while the child ran on.
#
# Exit 0 only when every assertion holds; exit 2 when a prerequisite is missing
# (BLOCKED — never reported as a pass). bash 3.2 compatible (C-05).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
SKILL="$(cd "$HERE/.." && pwd -P)"
ROOT="$(cd "$SKILL/../../.." && pwd -P)"
CRATE="$ROOT/substrate/prometheus-research"
FIX="$HERE/fixtures/thread-workers"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); echo "  ok   - $1"; }
bad() { FAIL=$((FAIL+1)); echo "  FAIL - $1" >&2; }

command -v jq >/dev/null 2>&1 || { echo "thread-scheduler: BLOCKED — jq is required" >&2; exit 2; }
[ -d "$FIX" ] || { echo "thread-scheduler: BLOCKED — fixtures missing at $FIX" >&2; exit 2; }

# Prefer an already-built binary; build only if absent, and never start a
# competing build (one Cargo build machine-wide).
BIN=""
for cand in "$CRATE/target/debug/prometheus-research" \
            "$HOME/.cargo/build-cache"/*/debug/prometheus-research; do
  [ -x "$cand" ] && { BIN="$cand"; break; }
done
if [ -z "$BIN" ]; then
  if pgrep -x cargo >/dev/null 2>&1; then
    echo "thread-scheduler: BLOCKED — no binary and another cargo build is active" >&2
    exit 2
  fi
  ( cd "$CRATE" && cargo build -q -p prometheus-research ) || {
    echo "thread-scheduler: BLOCKED — could not build prometheus-research" >&2; exit 2; }
  BIN="$(cd "$CRATE" && cargo metadata --format-version 1 --no-deps 2>/dev/null \
        | jq -r '.target_directory')/debug/prometheus-research"
fi
[ -x "$BIN" ] || { echo "thread-scheduler: BLOCKED — binary not executable: $BIN" >&2; exit 2; }

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

# Write one brief per thread. `budget.minutes` is omitted so the CLI flag
# governs; the brief-tightens-only rule is exercised separately below.
mkbrief() { # <pkg> <tid> <task>
  mkdir -p "$1/threads/$2"
  jq -n --arg t "$2" --arg k "$3" \
    '{thread_id:$t, task:$k, must_cover:[], avoid:[], recency_months:18,
      excluded_domains:[], budget:{cycles:8, max_sources:12}}' \
    > "$1/threads/$2/brief.json"
}

echo "== 1. terminal states: complete, failed, timeout all get a row =="
PKG="$WORK/states"; mkdir -p "$PKG"
mkbrief "$PKG" t01 "completes cleanly"
mkbrief "$PKG" t02 "exits non-zero"
mkbrief "$PKG" t03 "hangs forever"

# One worker per thread is not expressible through a single --worker flag, so
# run three single-thread packages and merge. Each still exercises the real
# dispatch path end to end.
for pair in "t01 complete.sh" "t02 failing.sh" "t03 hanging.sh"; do
  set -- $pair
  tid="$1"; script="$2"
  sub="$WORK/one-$tid"; mkdir -p "$sub"
  cp -R "$PKG/threads/$tid" "$sub/threads-tmp" 2>/dev/null || true
  mkdir -p "$sub/threads/$tid"; cp "$PKG/threads/$tid/brief.json" "$sub/threads/$tid/"
  rm -rf "$sub/threads-tmp"
  # 3s budget: ample for the fast workers, far short of the hang’s 300s sleep.
  "$BIN" threads run --package "$sub" --worker "$FIX/$script" \
        --max-parallel 2 --thread-seconds 3 --job-seconds 60 >/dev/null 2>&1 || true
  cp "$sub/threads/index.json" "$WORK/index-$tid.json" 2>/dev/null || true
done

st() { jq -r --arg t "$2" '.threads[] | select(.thread_id==$t) | .status' "$1" 2>/dev/null; }

[ "$(st "$WORK/index-t01.json" t01)" = "complete" ] \
  && ok "a clean worker is recorded complete" \
  || bad "a clean worker should be complete, got '$(st "$WORK/index-t01.json" t01)'"

[ "$(st "$WORK/index-t02.json" t02)" = "failed" ] \
  && ok "a non-zero exit is recorded failed, and the run continued" \
  || bad "a non-zero exit should be failed, got '$(st "$WORK/index-t02.json" t02)'"

jq -e '.threads[] | select(.thread_id=="t02") | .reason | test("code 3")' \
   "$WORK/index-t02.json" >/dev/null 2>&1 \
  && ok "the failed row names the exit code" \
  || bad "the failed row must name why it failed"

echo "== 2. a hung worker is KILLED, not merely abandoned =="
# A 3s budget against a 300s sleep: unambiguous, and fast enough for a gate.
# Proving a real kill needs a real process, so this leg spawns one.
PKG2="$WORK/hang"; mkdir -p "$PKG2"
mkbrief "$PKG2" t01 "hangs forever"
"$BIN" threads run --package "$PKG2" --worker "$FIX/hanging.sh" \
      --max-parallel 1 --thread-seconds 3 --job-seconds 60 >/dev/null 2>&1 || true

[ -f "$PKG2/hanging-started-t01" ] \
  && ok "the hanging worker really started (marker present)" \
  || bad "the hanging worker never started — the timeout proves nothing"

[ "$(st "$PKG2/threads/index.json" t01)" = "timeout" ] \
  && ok "the hung worker is recorded timeout" \
  || bad "expected timeout, got '$(st "$PKG2/threads/index.json" t01)'"

# The negative control. This marker can only exist if the child outlived its
# budget, which is exactly the pre-fix parking-lot failure mode.
[ ! -f "$PKG2/hanging-SURVIVED-t01" ] \
  && ok "the killed worker never reached its post-sleep marker (really dead)" \
  || bad "the worker SURVIVED its budget — the scheduler stopped waiting but did not kill"

sleep 1
pgrep -f "hanging.sh" >/dev/null 2>&1 \
  && bad "a hanging.sh process is still running after the scheduler returned" \
  || ok "no orphaned worker process remains"

echo "== 3. the concurrency cap is enforced in code =="
# Six tasks, cap of 2. If the cap were advisory all six would overlap.
PKG3="$WORK/cap"; mkdir -p "$PKG3"
for t in t01 t02 t03 t04 t05 t06; do mkbrief "$PKG3" "$t" "probe $t"; done
"$BIN" threads run --package "$PKG3" --worker "$FIX/concurrency-probe.sh" \
      --max-parallel 2 --thread-seconds 30 --job-seconds 60 >/dev/null 2>&1 || true

ROWS="$(jq '.threads | length' "$PKG3/threads/index.json" 2>/dev/null || echo 0)"
[ "$ROWS" = "6" ] \
  && ok "all six dispatches have a ledger row" \
  || bad "expected 6 ledger rows, got $ROWS"

DONE="$(jq '[.threads[] | select(.status=="complete")] | length' "$PKG3/threads/index.json" 2>/dev/null || echo 0)"
[ "$DONE" = "6" ] \
  && ok "all six completed despite the cap" \
  || bad "expected 6 complete, got $DONE"

# Reconstruct the true peak from the workers' own timestamps — computed
# outside the scheduler, so it cannot be fooled by a wrong internal counter.
PEAK="$(cat "$PKG3"/concurrency/*.log 2>/dev/null | awk '
  /^start/ { print $2, 1 }
  /^stop/  { print $2, -1 }
' | sort -n -k1 | awk '{ c += $2; if (c > m) m = c } END { print m+0 }')"

[ -n "$PEAK" ] && [ "$PEAK" -le 2 ] \
  && ok "observed peak concurrency was $PEAK, never above the cap of 2" \
  || bad "peak concurrency was $PEAK, which exceeds the cap of 2"

[ "$PEAK" -ge 2 ] \
  && ok "the run really was parallel (peak $PEAK), so the cap test is meaningful" \
  || bad "peak was $PEAK — the run was serial, so it does not test the cap"

echo "== 4. the job budget force-completes and records partial =="
# Six slow probes with a 1s job budget: the first tasks dispatch, then the job
# budget stops further dispatch. The undispatched ones must still get a row —
# recorded `partial` with the reason, because a director reading this ledger
# has to know they were planned and never ran. Silence would read as "never
# needed".
PKG4="$WORK/jobbudget"; mkdir -p "$PKG4"
for t in t01 t02 t03 t04 t05 t06; do mkbrief "$PKG4" "$t" "probe $t"; done
"$BIN" threads run --package "$PKG4" --worker "$FIX/concurrency-probe.sh" \
      --max-parallel 1 --thread-seconds 30 --job-seconds 1 >/dev/null 2>&1 || true

ROWS4="$(jq '.threads | length' "$PKG4/threads/index.json" 2>/dev/null || echo 0)"
[ "$ROWS4" = "6" ] \
  && ok "every planned thread has a row even when the job budget cut dispatch" \
  || bad "expected 6 rows after a job-budget cut, got $ROWS4"

PART="$(jq '[.threads[] | select(.status=="partial")] | length' "$PKG4/threads/index.json" 2>/dev/null || echo 0)"
[ "$PART" -ge 1 ] \
  && ok "undispatched threads are recorded partial ($PART of 6)" \
  || bad "expected at least one partial row, got $PART"

jq -e '[.threads[] | select(.status=="partial") | .reason | test("budget")] | all' \
   "$PKG4/threads/index.json" >/dev/null 2>&1 \
  && ok "each partial row names the budget as its reason" \
  || bad "a partial row does not explain itself"

jq -e '.budget_hit == true' "$PKG4/threads/index.json" >/dev/null 2>&1 \
  && ok "budget_hit is true on the ledger" \
  || bad "budget_hit must be true when a budget forced completion"

echo "== 5. the ledger shape matches thread-index.schema.json =="
jq -e '.schema_version | test("^1\\.")' "$PKG3/threads/index.json" >/dev/null 2>&1 \
  && ok "schema_version is 1.x" || bad "schema_version missing or wrong"
jq -e '.package_id | length > 0' "$PKG3/threads/index.json" >/dev/null 2>&1 \
  && ok "package_id is present" || bad "package_id missing"
jq -e '[.threads[] | select(.status != "complete" and (.reason == null or .reason == ""))] | length == 0' \
   "$WORK/index-t02.json" >/dev/null 2>&1 \
  && ok "every non-complete row carries a reason" \
  || bad "a non-complete row is missing its reason"
jq -e 'all(.threads[]; .thread_id | test("^t[0-9]{2,}$"))' "$PKG3/threads/index.json" >/dev/null 2>&1 \
  && ok "every thread_id matches the schema pattern" || bad "a thread_id violates ^t[0-9]{2,}$"

echo
echo "thread-scheduler: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
