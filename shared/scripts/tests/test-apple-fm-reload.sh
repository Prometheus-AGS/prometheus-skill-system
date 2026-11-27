#!/bin/bash
# test-apple-fm-reload.sh — the Apple Foundation Models install step must leave
# the service loaded and healthy every time it runs.
#
# Runs the real apple-foundation-models.sh --install against the real launchd
# and the real apfel, with a scratch HOME so the rendered plist, the logs, the
# liter-llm config and models.toml are all scratch files. The job label is
# fixed, so the machine's own ai.prometheus.apple-fm service is replaced while
# the test runs and put back, from its own plist, when the test exits.
#
#   1. a changed job definition restarts the service, eight times in a row, and
#      it is loaded and healthy after every one (the unload race lost it in
#      about one restart in six);
#   2. an unchanged definition with a healthy service is left running: same
#      process, no restart;
#   3. an unchanged definition whose job is not loaded is started.
#
# Exit 0 pass, 1 fail, 2 BLOCKED (not an eligible Mac, or apfel absent). bash 3.2.
set -u

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
SCRIPT="$ROOT/shared/scripts/apple-foundation-models.sh"
LABEL="ai.prometheus.apple-fm"
DOMAIN="gui/$(id -u)"
PORT="${PROMETHEUS_APPLE_FM_PORT:-11437}"
RESTARTS=8

blocked() { echo "BLOCKED: $*" >&2; exit 2; }
[ "$(uname -s)" = "Darwin" ] || blocked "not macOS"
command -v launchctl >/dev/null 2>&1 || blocked "launchctl missing"
command -v curl >/dev/null 2>&1 || blocked "curl missing"
bash "$SCRIPT" --detect | grep -q '"eligible":true' || blocked "this Mac is not eligible for Apple Foundation Models"
bash "$SCRIPT" --detect | grep -q '"apfel":""' && blocked "apfel is not installed"

loaded() { launchctl print "$DOMAIN/$LABEL" >/dev/null 2>&1; }
healthy() { curl -fsS -m 3 "http://127.0.0.1:$PORT/health" >/dev/null 2>&1; }
pid_of() { launchctl print "$DOMAIN/$LABEL" 2>/dev/null | awk '$1 == "pid" && $2 == "=" {print $3; exit}'; }
unload() {
  local i=0
  launchctl bootout "$DOMAIN/$LABEL" >/dev/null 2>&1 || true
  while loaded && [ "$i" -lt 40 ]; do sleep 0.5; i=$((i + 1)); done
}

REAL_PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
WAS_LOADED=0; loaded && WAS_LOADED=1
S="$(mktemp -d)"
restore() {
  unload
  if [ "$WAS_LOADED" -eq 1 ] && [ -f "$REAL_PLIST" ]; then
    local i=0
    while [ "$i" -lt 5 ]; do launchctl bootstrap "$DOMAIN" "$REAL_PLIST" >/dev/null 2>&1 && break; sleep 1; i=$((i + 1)); done
    i=0
    while ! healthy && [ "$i" -lt 30 ]; do sleep 1; i=$((i + 1)); done
    healthy || echo "WARNING: the machine's own $LABEL service did not come back; run: launchctl bootstrap $DOMAIN $REAL_PLIST" >&2
  fi
  rm -rf "$S"
}
trap restore EXIT

export HOME="$S/home"
mkdir -p "$HOME"
unset PROMETHEUS_APPLE_FM PROMETHEUS_APPLE_FM_FORCE_INELIGIBLE LITER_LLM_CONFIG
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
install() { bash "$SCRIPT" --install 2>&1; }

pass=0
fail() { echo "FAIL: $*" >&2; exit 1; }
ok() { pass=$((pass + 1)); echo "ok - $*"; }

# --- 1. a changed definition restarts the service, repeatedly ------------------
n=1
last_pid=""
while [ "$n" -le "$RESTARTS" ]; do
  rm -f "$PLIST"                       # absent plist = changed definition = restart path
  out="$(install)" || fail "install $n exited non-zero: $out"
  loaded || fail "install $n left the job unloaded: $out"
  healthy || fail "install $n left the service unhealthy: $out"
  printf '%s' "$out" | grep -q 'did not become healthy' && fail "install $n reported an unhealthy service: $out"
  pid="$(pid_of)"
  [ -n "$pid" ] || fail "install $n: the job has no process"
  [ "$pid" != "$last_pid" ] || fail "install $n did not restart the service (pid $pid twice)"
  last_pid="$pid"
  n=$((n + 1))
done
ok "$RESTARTS consecutive restarts each left the job loaded and healthy"

# --- 2. unchanged and healthy: left running -------------------------------------
before="$(pid_of)"
out="$(install)" || fail "unchanged install exited non-zero: $out"
printf '%s' "$out" | grep -q 'left running' || fail "an unchanged, healthy service was not reported as left running: $out"
[ "$(pid_of)" = "$before" ] || fail "an unchanged, healthy service was restarted (pid $before -> $(pid_of))"
healthy || fail "the service is unhealthy after an unchanged install"
ok "an unchanged definition with a healthy service is left running (pid $before)"

# --- 3. unchanged but not loaded: started -----------------------------------------
unload
loaded && fail "could not unload the job for the scenario"
out="$(install)" || fail "install with the job unloaded exited non-zero: $out"
loaded || fail "an unloaded job was not started: $out"
healthy || fail "the started service is unhealthy: $out"
ok "an unchanged definition whose job is not loaded is started"

echo "test-apple-fm-reload: $pass passed"
