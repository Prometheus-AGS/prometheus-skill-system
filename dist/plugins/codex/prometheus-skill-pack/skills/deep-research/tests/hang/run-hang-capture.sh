#!/usr/bin/env bash
# run-hang-capture.sh — instrumented repeat-run capture harness for the
# intermittent stage-10 hang.
#
# Change:      change-drt-008-hang-capture-harness
# Phase:       deep-research-onyx-parity › stage-10-hang-investigation
# Purpose:     loop driver-contract.sh --scenario full-run up to --runs K (or
#              --stop-on-hang), with scoped pre-run cleanup, a per-run
#              timeout, and — on timeout — a LIVE capture (ps process tree,
#              macOS `sample` stacks, output-volume observation) taken BEFORE
#              the process tree is torn down by recorded PID.
#
# Attribution note: the 90 s default timeout matches the EXTERNAL timeout(1)
# wrapper under which the hang was observed (exit 124); the driver suite
# itself imposes no per-scenario timeout (verified by grep on
# driver-contract.sh — no timeout/124 logic).
#
# bash 3.2 compatible (C-05). set -euo pipefail; non-zero exit on failure.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
DRIVER="$HERE/../driver-contract.sh"

RUNS=12
TIMEOUT=90
SCENARIO="full-run"
OUT_DIR="$HERE/captures"
STOP_ON_HANG=0
XTRACE=0

usage() {
  cat <<EOF
usage: $0 [--runs N] [--timeout S] [--scenario NAME] [--out DIR] [--stop-on-hang] [--xtrace]
  --runs N        campaign length (default 12)
  --timeout S     per-run timeout in seconds (default 90; external-wrapper
                  attribution — the suite has no internal timeout)
  --scenario NAME driver scenario (default full-run)
  --out DIR       capture output dir (default $OUT_DIR)
  --stop-on-hang  stop after the first captured hang (default: capture and
                  continue, marking post-capture runs potentially perturbed)
  --xtrace        DIAGNOSTIC-HUNT MODE: propagate xtrace to the driver chain
                  (traces to a per-run .xtrace file) so a hang names its last
                  executed command even when stdout buffering eats the log —
                  defeats the 2026-09-09 diagnostic blindness. NOTE: the
                  fixture environment captures child stderr into files the
                  scenario asserts on, so under --xtrace assertion counts are
                  UNRELIABLE (observed 3/12) — use for recurrence hunting,
                  not for certification runs, which need accurate counts.
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --runs)       RUNS="$2"; shift 2 ;;
    --timeout)    TIMEOUT="$2"; shift 2 ;;
    --scenario)   SCENARIO="$2"; shift 2 ;;
    --out)        OUT_DIR="$2"; shift 2 ;;
    --stop-on-hang) STOP_ON_HANG=1; shift ;;
    --xtrace)     XTRACE=1; shift ;;
    -h|--help)    usage; exit 0 ;;
    *)            echo "$0: unknown arg: $1" >&2; usage >&2; exit 1 ;;
  esac
done

[ -f "$DRIVER" ] || { echo "$0: driver not found: $DRIVER" >&2; exit 2; }
mkdir -p "$OUT_DIR"
CAMPAIGN="$OUT_DIR/campaign-$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "$CAMPAIGN"
SUMMARY="$CAMPAIGN/summary.tsv"
printf 'run\tstart_utc\tduration_s\texit\tassertions\tloadavg\tresult\n' > "$SUMMARY"

log() { printf '[hang-capture] %s\n' "$*" >&2; }

# --- scoped cleanup: kill the recorded PID tree from a prior run ----------------
# Kills ONLY processes recorded in this campaign's PID files (root plus the
# descendant snapshot refreshed each poll tick) and any descendants still
# walkable from a live root — never a broad, name-based kill. The descendant
# snapshot matters because SIGKILLed children reparent to launchd and become
# unwalkable from the root; killing from the recorded list still reaches them.
kill_recorded_tree() { # <pid-file>
  local pf="$1" root="" recorded="" p d all=""
  [ -f "$pf" ] || return 0
  root="$(sed -n '1p' "$pf" 2>/dev/null || true)"
  recorded="$(sed -n '2,$p' "$pf" 2>/dev/null || true)"
  rm -f "$pf"
  all="$recorded"
  case "$root" in
    ''|*[!0-9]*) : ;;  # no usable root: fall back to the recorded list only
    *)
      if kill -0 "$root" 2>/dev/null; then
        all="$all $(descendants_of "$root")"
      fi
      all="$all $root"
      ;;
  esac
  # TERM everything first (children usually honor TERM while still parented)
  for p in $all; do kill "$p" 2>/dev/null || true; done
  sleep 1
  # re-walk if the root survived, then KILL every survivor on the list
  case "$root" in
    ''|*[!0-9]*) : ;;
    *)
      if kill -0 "$root" 2>/dev/null; then
        for d in $(descendants_of "$root"); do all="$all $d"; done
      fi
      ;;
  esac
  for p in $all; do kill -9 "$p" 2>/dev/null || true; done
  sleep 1
  for p in $all; do
    if kill -0 "$p" 2>/dev/null; then log "cleanup: WARNING pid $p survived teardown"; fi
  done
  log "cleanup: tore down recorded tree (root=${root:-none})"
}

# --- descendants of a pid --------------------------------------------------------
descendants_of() { # <pid> -> prints descendant pids (excluding root)
  local root="$1"
  ps -axo pid=,ppid= | awk -v r="$root" '
    { pp[$1] = $2 }
    END {
      # BFS from r using parent->child map collected above
      queue = r; seen[r] = 1
      while (queue != "") {
        n = split(queue, arr, ","); nextq = ""
        for (i = 1; i <= n; i++)
          for (p in pp)
            if (pp[p] == arr[i] && !seen[p]) { seen[p] = 1; print p; nextq = nextq "," p }
        queue = nextq
      }
    }'
}

# --- live capture BEFORE teardown ------------------------------------------------
capture_hang() { # <pid> <runlabel>
  local pid="$1" label="$2" d
  log "TIMEOUT on $label — capturing process tree rooted at $pid BEFORE teardown"
  ps -axo pid=,ppid=,etime=,stat=,command= > "$CAMPAIGN/$label-ps-full.txt"
  descendants_of "$pid" | sort -n > "$CAMPAIGN/$label-descendants.txt"
  # the materialized rooted tree (G1's artifact): ps rows for root+descendants,
  # each carrying its command line — no manual join needed downstream
  { echo "$pid"; cat "$CAMPAIGN/$label-descendants.txt"; } | sort -u | awk 'NR==FNR { want[$1]=1; next } want[$1]' - "$CAMPAIGN/$label-ps-full.txt" > "$CAMPAIGN/$label-tree.txt"
  { echo "root_pid=$pid"; echo "captured_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"; } > "$CAMPAIGN/$label-meta.txt"
  # macOS sample stacks for the root and each (capped) descendant
  if command -v sample >/dev/null 2>&1; then
    sample "$pid" 3 -file "$CAMPAIGN/$label-sample-root.txt" >/dev/null 2>&1 || true
    n=0
    while IFS= read -r d; do
      [ -n "$d" ] || continue
      n=$((n + 1)); [ "$n" -le 8 ] || break
      sample "$d" 3 -file "$CAMPAIGN/$label-sample-$d.txt" >/dev/null 2>&1 || true
    done < "$CAMPAIGN/$label-descendants.txt"
  else
    echo "sample(1) unavailable" > "$CAMPAIGN/$label-sample-UNAVAILABLE.txt"
  fi
  # output-volume observation at the hang site (~64 KB pipe-buffer hypothesis):
  # the run log size plus per-pipe kernel buffer sizes where lsof can see them.
  {
    echo "run_log_bytes=$(wc -c < "$CAMPAIGN/$label.log" 2>/dev/null || echo 0)"
    if command -v lsof >/dev/null 2>&1; then
      for p in "$pid" $(cat "$CAMPAIGN/$label-descendants.txt" 2>/dev/null); do
        lsof -p "$p" 2>/dev/null | awk -v pid="$p" '$5 == "PIPE" || $5 == "FIFO" {print pid, $NF}' || true
      done
    else
      echo "lsof unavailable"
    fi
  } > "$CAMPAIGN/$label-output-volumes.txt" 2>/dev/null || true
  log "capture complete: $CAMPAIGN/$label-*"
}

HANGS=0
PERTURBED=0
i=1
while [ "$i" -le "$RUNS" ]; do
  label="run-$(printf '%02d' "$i")"
  # pre-run cleanup (independence for the N-bound): prior PID tree + fresh log
  kill_recorded_tree "$CAMPAIGN/last.pid"
  START="$CAMPAIGN/$label.log"
  LOAD="$(uptime | sed 's/.*averages*: //' | cut -d, -f1-3)"
  START_TS="$(date -u +%H:%M:%S)"
  T0=$SECONDS
  log "$label: starting (load: $LOAD) — driver --scenario $SCENARIO"
  set +e
  if [ "$XTRACE" -eq 1 ]; then
    # SHELLOPTS=xtrace at child startup; traces flow to each bash's stderr,
    # collected in the per-run .xtrace file. NOTE: traces pollute the
    # fixture's stderr-captured assertion files, so assertion counts under
    # --xtrace are UNRELIABLE (observed 3/12) — certification runs must use
    # standard mode; --xtrace is for recurrence hunting only.
    # A hang leaves the last-executed command at the xtrace tail even when
    # stdout buffering eats the log.
    env SHELLOPTS=xtrace bash "$DRIVER" --scenario "$SCENARIO" \
      > "$START" 2>> "$CAMPAIGN/$label.xtrace" &
  else
    bash "$DRIVER" --scenario "$SCENARIO" > "$START" 2>&1 &
  fi
  DPID=$!
  echo "$DPID" > "$CAMPAIGN/last.pid"
  RC=""
  ELAPSED=0
  while kill -0 "$DPID" 2>/dev/null; do
    ELAPSED=$((SECONDS - T0))
    # refresh the descendant snapshot every tick so cleanup can reach children
    # that later reparent when a parent dies or is killed
    { echo "$DPID"; descendants_of "$DPID"; } > "$CAMPAIGN/last.pid"
    if [ "$ELAPSED" -ge "$TIMEOUT" ]; then break; fi
    sleep 1
  done
  if kill -0 "$DPID" 2>/dev/null; then
    DUR="$ELAPSED"   # duration at the timeout instant — excludes capture/teardown time
    capture_hang "$DPID" "$label"
    HANGS=$((HANGS + 1))
    PERTURBED=1
    RESULT="HANG"
    # refresh the snapshot AFTER capture (sample calls block ~27s) by MERGING
    # with the pre-capture recorded list — never replacing it: a root that dies
    # during capture must not erase the PIDs still recorded from the last tick
    { cat "$CAMPAIGN/last.pid" 2>/dev/null; descendants_of "$DPID"; echo "$DPID"; } \
      | sort -un > "$CAMPAIGN/last.pid.tmp" && mv "$CAMPAIGN/last.pid.tmp" "$CAMPAIGN/last.pid"
    # scoped teardown AFTER capture
    kill_recorded_tree "$CAMPAIGN/last.pid"
    RC=124  # conventional external-timeout exit; result=HANG carries the semantics
  else
    wait "$DPID" || RC=$?
    [ -n "${RC:-}" ] || RC=0
    DUR=$((SECONDS - T0))
    if [ "$RC" -eq 0 ]; then RESULT="clean"; else RESULT="failed"; fi
    # post-capture runs are potentially perturbed by the earlier hang's residue
    if [ "${PERTURBED:-0}" -eq 1 ]; then RESULT="$RESULT-perturbed"; fi
  fi
  set -e
  ASSERTS="$(grep -cE 'PASS|not ok|FAIL' "$START" 2>/dev/null || true)"
  : "${ASSERTS:=0}"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$label" "$START_TS" "$DUR" "$RC" "$ASSERTS" "$LOAD" "$RESULT" >> "$SUMMARY"
  log "$label: $RESULT duration=${DUR}s exit=$RC assertions=$ASSERTS"
  if [ "$RESULT" = "HANG" ] && [ "$STOP_ON_HANG" -eq 1 ]; then
    log "stop-on-hang requested; stopping campaign"
    break
  fi
  i=$((i + 1))
done

kill_recorded_tree "$CAMPAIGN/last.pid"
# when --stop-on-hang fired, i is the hung run's index; otherwise the loop ran all RUNS
if [ "$STOP_ON_HANG" -eq 1 ] && [ "$i" -le "$RUNS" ]; then TOTAL="$i"; else TOTAL="$RUNS"; fi
{
  echo "runs=$TOTAL"
  echo "hangs=$HANGS"
  echo "failure_rate=$HANGS/$TOTAL"
  echo "scenario=$SCENARIO timeout=${TIMEOUT}s completed_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
} > "$CAMPAIGN/rate.txt"
log "campaign complete: $HANGS hang(s) in $TOTAL run(s) — $CAMPAIGN"
cat "$SUMMARY" >&2
exit 0
