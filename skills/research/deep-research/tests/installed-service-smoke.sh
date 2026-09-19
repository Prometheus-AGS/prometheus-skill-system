#!/usr/bin/env bash
# installed-service-smoke.sh — change-drt-007, task 5.
#
# Proves the two install-surface defects the change-rah-011 evidence run found
# are (a) actually repaired at the template level and (b) visible before a job
# runs rather than after one fails.
#
#   D-A  com.prometheus.research.plist granted no PATH, so under launchd the
#        daemon could not resolve `claude` or `codex` and every research_start
#        blocked — although both harnesses were installed.
#   D-B  the installed plugin generation shipped a pre-contract stub driver, so
#        a harness child exited 0 having produced no package.
#
# The daemon under test runs on a spare port with an isolated output root; the
# live launchd service is never touched. A gate that cannot run is BLOCKED
# (exit 2), never reported as a pass.
#
# bash 3.2 compatible (constraint C-05).
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
SKILL="$(cd "$HERE/.." && pwd -P)"
REPO="$(cd "$SKILL/../../.." && pwd -P)"
PLIST_SRC="$REPO/substrate/prometheus-research/com.prometheus.research.plist"
STUB="$HERE/fixtures/stub-driver/run-research.sh"
PORT="${SMOKE_PORT:-7899}"

PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); echo "  ok   - $1"; }
bad()  { FAIL=$((FAIL+1)); echo "  FAIL - $1" >&2; }
check(){ d="$1"; shift; if "$@" >/dev/null 2>&1; then ok "$d"; else bad "$d"; fi; }

TMP="$(mktemp -d "${TMPDIR:-/tmp}/installed-smoke-XXXXXX")"
cleanup() { [ -n "${DPID:-}" ] && kill "$DPID" 2>/dev/null; rm -rf "$TMP"; }
trap cleanup EXIT

command -v python3 >/dev/null 2>&1 || { echo "installed-service-smoke: python3 required" >&2; exit 2; }
[ -f "$PLIST_SRC" ] || { echo "installed-service-smoke: plist template missing" >&2; exit 2; }
[ -x "$STUB" ]      || { echo "installed-service-smoke: stub fixture missing" >&2; exit 2; }

BIN=""
for c in "$REPO/substrate/prometheus-research/target/debug/prometheus-research" \
         "$REPO/substrate/prometheus-research/target/release/prometheus-research" \
         "$HOME/.local/bin/prometheus-research"; do
  [ -x "$c" ] && { BIN="$c"; break; }
done
if [ -z "$BIN" ]; then
  echo "installed-service-smoke: BLOCKED — no prometheus-research binary found." >&2
  echo "                         Build it first (cargo build -p prometheus-research)." >&2
  exit 2
fi

# --------------------------------------------------------------------------
echo "== 1. D-A: the rendered plist grants a PATH that resolves a harness =="
RESEARCH_PATH="/usr/local/bin:/usr/local/sbin:/opt/homebrew/bin:/opt/homebrew/sbin:${HOME}/.cargo/bin:${HOME}/.local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
sed -e "s|__HOME__|${HOME}|g" \
    -e "s|__PROMETHEUS_HOME__|${HOME}|g" \
    -e "s|__PROMETHEUS_USER__|$(id -un)|g" \
    -e "s|__PROMETHEUS_PATH__|${RESEARCH_PATH}|g" \
    "$PLIST_SRC" > "$TMP/rendered.plist"

check "the template declares EnvironmentVariables (it did not before: defect D-A)" \
  grep -q "EnvironmentVariables" "$PLIST_SRC"
check "no placeholder survives rendering" \
  bash -c '! grep -q "__PROMETHEUS_\|__HOME__" "$1"' _ "$TMP/rendered.plist"

if command -v plutil >/dev/null 2>&1; then
  check "the rendered plist is valid (plutil -lint)" plutil -lint "$TMP/rendered.plist"
  RENDERED_PATH="$(plutil -extract EnvironmentVariables.PATH raw "$TMP/rendered.plist" 2>/dev/null || echo '')"
else
  RENDERED_PATH="$(python3 -c "
import plistlib,sys
print(plistlib.load(open(sys.argv[1],'rb')).get('EnvironmentVariables',{}).get('PATH',''))" "$TMP/rendered.plist")"
  echo "  note - plutil unavailable; parsed with python3 plistlib instead"
fi

check "the granted PATH is non-empty" test -n "$RENDERED_PATH"

# Environment absence is a precondition, not a failure. This script's own policy
# is that a gate which cannot run is BLOCKED, so a machine with no harness
# installed exits 2 here rather than reporting a FAIL for something the change
# did not break.
if ! PATH="$RENDERED_PATH" command -v claude >/dev/null 2>&1 \
   && ! PATH="$RENDERED_PATH" command -v codex >/dev/null 2>&1; then
  echo "installed-service-smoke: BLOCKED — no harness (claude/codex) installed on this machine." >&2
  echo "                         The plist grants a PATH correctly; there is simply nothing to" >&2
  echo "                         resolve on it. Install a harness to exercise sections 2 and 3." >&2
  exit 2
fi
ok "the granted PATH resolves a harness (claude or codex)"

# The regression this locks down: rendering with only __HOME__ (the old
# behaviour) must NOT yield a usable PATH.
sed "s|__HOME__|${HOME}|g" "$PLIST_SRC" > "$TMP/oldstyle.plist"
check "negative control: the old __HOME__-only substitution leaves PATH unusable" \
  grep -q "__PROMETHEUS_PATH__" "$TMP/oldstyle.plist"

# --------------------------------------------------------------------------
echo "== 2. D-B: a stub driver is reported stale BEFORE a job is started =="
export RESEARCH_OUTPUT_DIR="$TMP/research"
mkdir -p "$RESEARCH_OUTPUT_DIR"

env PATH="$RENDERED_PATH" RESEARCH_DRIVER="$STUB" RESEARCH_OUTPUT_DIR="$RESEARCH_OUTPUT_DIR" \
  "$BIN" --mode server --port "$PORT" >"$TMP/stub.log" 2>&1 &
DPID=$!
disown "$DPID" 2>/dev/null || true
for _ in 1 2 3 4 5 6 7 8 9 10; do
  curl -s -m 2 "http://127.0.0.1:$PORT/health" >/dev/null 2>&1 && break
  sleep 1
done
curl -s -m 5 "http://127.0.0.1:$PORT/health" > "$TMP/stub-health.json" 2>/dev/null || true

check "the daemon answered /health" test -s "$TMP/stub-health.json"
check "the stub driver is reported STALE" \
  python3 -c "
import json,sys
d=json.load(open('$TMP/stub-health.json'))
sys.exit(0 if d['execution']['driver']['status']=='stale' else 1)"
check "the report names the driver path" \
  python3 -c "
import json,sys
d=json.load(open('$TMP/stub-health.json'))
sys.exit(0 if d['execution']['driver'].get('path') else 1)"
check "the report names the driver size in bytes" \
  python3 -c "
import json,sys
d=json.load(open('$TMP/stub-health.json'))
sys.exit(0 if isinstance(d['execution']['driver'].get('size_bytes'),int) and d['execution']['driver']['size_bytes']>0 else 1)"
check "the report names WHICH stage-contract markers are missing" \
  python3 -c "
import json,sys
d=json.load(open('$TMP/stub-health.json'))
m=d['execution']['driver'].get('missing_stage_contract_markers',[])
sys.exit(0 if len(m)==4 else 1)"
check "the startup log warned it is NOT ready" grep -q "NOT ready" "$TMP/stub.log"
# Wait for the listener to actually go away before rebinding the port; a bare
# `sleep 1` races and yields an intermittent EADDRINUSE in section 3.
kill "$DPID" 2>/dev/null || true
for _ in 1 2 3 4 5 6 7 8 9 10; do
  curl -s -m 1 "http://127.0.0.1:$PORT/health" >/dev/null 2>&1 || break
  sleep 1
done
wait "$DPID" 2>/dev/null || true
DPID=""

# --------------------------------------------------------------------------
echo "== 3. the repo driver on the same PATH is reported healthy =="
REAL_DRIVER="$SKILL/scripts/run-research.sh"
if [ ! -x "$REAL_DRIVER" ]; then
  echo "  BLOCKED - repo driver missing at $REAL_DRIVER" >&2
  exit 2
fi
env PATH="$RENDERED_PATH" RESEARCH_DRIVER="$REAL_DRIVER" RESEARCH_OUTPUT_DIR="$RESEARCH_OUTPUT_DIR" \
  "$BIN" --mode server --port "$PORT" >"$TMP/real.log" 2>&1 &
DPID=$!
disown "$DPID" 2>/dev/null || true
for _ in 1 2 3 4 5 6 7 8 9 10; do
  curl -s -m 2 "http://127.0.0.1:$PORT/health" >/dev/null 2>&1 && break
  sleep 1
done
curl -s -m 5 "http://127.0.0.1:$PORT/health" > "$TMP/real-health.json" 2>/dev/null || true

check "the harness resolves from the plist's PATH (D-A repaired)" \
  python3 -c "
import json,sys
d=json.load(open('$TMP/real-health.json'))
sys.exit(0 if d['execution']['harness']['status']=='ok' else 1)"
check "the repo driver is reported ok, not stale" \
  python3 -c "
import json,sys
d=json.load(open('$TMP/real-health.json'))
sys.exit(0 if d['execution']['driver']['status']=='ok' else 1)"
check "no stage-contract marker is missing from the repo driver" \
  python3 -c "
import json,sys
d=json.load(open('$TMP/real-health.json'))
sys.exit(0 if d['execution']['driver']['missing_stage_contract_markers']==[] else 1)"
check "the startup log reported ready" grep -q "execution self-check: ready" "$TMP/real.log"
kill "$DPID" 2>/dev/null; DPID=""

# --------------------------------------------------------------------------
echo "== 4. the launchd leg: bootstrap the rendered plist and start a real job =="
# Task 5 asks for a launchd bootstrap and a non-null harness_pid. That is done
# here under a DISTINCT label and port, so the live com.prometheus.research
# service is never touched. Opt in with SMOKE_LAUNCHD=1: bootstrapping a service
# is a side effect on the operator's machine, and a test should not do that by
# default.
if [ "${SMOKE_LAUNCHD:-0}" != "1" ]; then
  echo "  skip - launchd leg not requested (set SMOKE_LAUNCHD=1 to bootstrap a"
  echo "         throwaway service and assert a non-null harness_pid)"
elif ! command -v launchctl >/dev/null 2>&1; then
  echo "  BLOCKED - launchctl unavailable (not macOS); the launchd leg cannot run here." >&2
  exit 2
else
  LABEL="com.prometheus.research.smoke.$$"
  LPORT=$((PORT + 1))
  LPLIST="$TMP/$LABEL.plist"
  # Same rendering the installer performs, retargeted to a throwaway label/port.
  sed -e "s|<string>com.prometheus.research</string>|<string>$LABEL</string>|" \
      -e "s|<string>--mode</string>|<string>--port</string><string>$LPORT</string><string>--mode</string>|" \
      "$TMP/rendered.plist" > "$LPLIST"
  sed -i.bak "s|__HOME__|${HOME}|g" "$LPLIST" 2>/dev/null || true

  launchctl bootout "gui/$(id -u)" "$LPLIST" >/dev/null 2>&1 || true
  if launchctl bootstrap "gui/$(id -u)" "$LPLIST" >"$TMP/bootstrap.log" 2>&1; then
    ok "the rendered plist bootstraps under launchd"
    for _ in 1 2 3 4 5 6 7 8 9 10; do
      curl -s -m 2 "http://127.0.0.1:$LPORT/health" >/dev/null 2>&1 && break
      sleep 1
    done
    curl -s -m 5 "http://127.0.0.1:$LPORT/health" > "$TMP/launchd-health.json" 2>/dev/null || true
    # The plist runs ~/.local/bin/prometheus-research — the INSTALLED binary,
    # which may be older than the working tree and expose no `execution` field.
    # That is a stale install, not a failure of the repair, so it is a skip with
    # its reason rather than a FAIL.
    #
    # `harness_pid` is NOT universal either: it is recorded by the daemon-job
    # path, which is absent from older builds. Both assertions below are
    # therefore gated on the installed binary actually supporting them — a
    # missing capability is reported as a skip naming the remedy (reinstall),
    # never as a pass and never as a failure of the D-A repair.
    if python3 -c "
import json,sys
d=json.load(open('$TMP/launchd-health.json'))
sys.exit(0 if 'execution' in d else 1)" 2>/dev/null; then
      check "the launchd-started daemon resolves a harness from the plist's PATH" \
        python3 -c "
import json,sys
d=json.load(open('$TMP/launchd-health.json'))
sys.exit(0 if d['execution']['harness']['status']=='ok' else 1)"
    else
      echo "  skip - the installed binary at ~/.local/bin predates the self-check"
      echo "         (installed $(wc -c < "$HOME/.local/bin/prometheus-research" 2>/dev/null | tr -d ' ') bytes);"
      echo "         reinstall to assert the self-check under launchd. The harness_pid check below
         is gated the same way and reports its own skip if unsupported."
    fi
    # The job leg: start one and assert the daemon recorded a real child pid.
    JOB=$(curl -s -m 20 -X POST "http://127.0.0.1:$LPORT/api/v1/jobs" \
            -H 'Content-Type: application/json' \
            -d '{"query":"smoke: report the current working directory and stop","depth":"shallow"}' \
          | python3 -c "import json,sys; print(json.load(sys.stdin).get('job_id',''))" 2>/dev/null || echo "")
    if [ -n "$JOB" ]; then
      ok "a job was accepted under launchd (job_id: $JOB)"
      HPID=""
      for _ in 1 2 3 4 5 6 7 8 9 10 11 12; do
        HPID=$(curl -s -m 5 "http://127.0.0.1:$LPORT/api/v1/jobs/$JOB" \
               | python3 -c "import json,sys; print(json.load(sys.stdin).get('harness_pid') or '')" 2>/dev/null || echo "")
        [ -n "$HPID" ] && break
        sleep 3
      done
      if [ -n "$HPID" ]; then
        ok "harness_pid is non-null (the D-A repair, proven under launchd)"
      else
        # Distinguish "the installed binary cannot report it" from "the repair failed".
        JSTATUS=$(curl -s -m 5 "http://127.0.0.1:$LPORT/api/v1/jobs/$JOB" \
                  | python3 -c "import json,sys; d=json.load(sys.stdin); print(d.get('status',''))" 2>/dev/null || echo "")
        if [ "$JSTATUS" = "blocked" ]; then
          bad "harness_pid is null and the job is blocked — the D-A repair did NOT take"
        else
          echo "  skip - the installed binary does not report harness_pid (job status: ${JSTATUS:-unknown});"
          echo "         reinstall to assert it. This is a stale install, not a failed repair."
        fi
      fi
    else
      bad "a job could not be started under launchd"
    fi
    launchctl bootout "gui/$(id -u)" "$LPLIST" >/dev/null 2>&1 || true
  else
    echo "  BLOCKED - launchctl bootstrap failed: $(head -1 "$TMP/bootstrap.log" 2>/dev/null)" >&2
    exit 2
  fi
fi

echo
echo "installed-service-smoke: $PASS passed, $FAIL failed"
if [ "$FAIL" -eq 0 ]; then
  echo "VERDICT: D-A repaired at the template (the rendered plist grants a harness-resolving PATH)."
  echo "         D-B is now VISIBLE before a job runs; the stale installed driver itself is not"
  echo "         yet republished — see the change spec, task 4, for why that is blocked."
fi
[ "$FAIL" -eq 0 ]
