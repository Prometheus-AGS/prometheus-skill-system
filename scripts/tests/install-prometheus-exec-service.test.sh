#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP_ROOT="$(mktemp -d)"
trap 'rm -rf "$TMP_ROOT"' EXIT INT TERM

mkdir -p "$TMP_ROOT/bin" "$TMP_ROOT/home" "$TMP_ROOT/state"

cat >"$TMP_ROOT/bin/prometheus-exec" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
case "${1:-}" in
  --version) printf '%s\n' 'prometheus-exec 1.7.0' ;;
  init)
    while [[ $# -gt 0 ]]; do
      if [[ "$1" == "--identity" ]]; then
        mkdir -p "$(dirname "$2")"
        printf '%s\n' '{}' >"$2"
        exit 0
      fi
      shift
    done
    exit 2
    ;;
  *) exit 2 ;;
esac
SH
chmod +x "$TMP_ROOT/bin/prometheus-exec"

cat >"$TMP_ROOT/bin/plutil" <<'SH'
#!/usr/bin/env bash
exit 0
SH
chmod +x "$TMP_ROOT/bin/plutil"

cat >"$TMP_ROOT/bin/launchctl" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
state="${PROMETHEUS_EXEC_SERVICE_TEST_STATE:?}"
case "${1:-}" in
  bootout)
    rm -f "$state/loaded"
    exit 0
    ;;
  print)
    [[ -f "$state/loaded" ]]
    ;;
  bootstrap)
    count=0
    [[ -f "$state/bootstrap-count" ]] && count="$(cat "$state/bootstrap-count")"
    count=$((count + 1))
    printf '%s\n' "$count" >"$state/bootstrap-count"
    if [[ "$count" -eq 1 ]]; then
      exit 5
    fi
    touch "$state/loaded"
    ;;
  kickstart)
    [[ -f "$state/loaded" ]]
    ;;
  *) exit 2 ;;
esac
SH
chmod +x "$TMP_ROOT/bin/launchctl"

PATH="$TMP_ROOT/bin:$PATH" \
HOME="$TMP_ROOT/home" \
PROMETHEUS_EXEC_BIN="$TMP_ROOT/bin/prometheus-exec" \
PROMETHEUS_EXEC_ROOT="$TMP_ROOT/home/.prometheus/exec" \
PROMETHEUS_EXEC_SOCKET="$TMP_ROOT/home/.prometheus/run/prometheus-exec.sock" \
PROMETHEUS_EXEC_IDENTITY="$TMP_ROOT/home/.prometheus/exec/identity.json" \
PROMETHEUS_EXEC_PLUGIN_ROOT="$TMP_ROOT/home/.prometheus/plugins/prometheus-skill-pack" \
PROMETHEUS_EXEC_LOG_DIR="$TMP_ROOT/home/.prometheus/logs" \
PROMETHEUS_EXEC_LAUNCH_AGENT="$TMP_ROOT/home/Library/LaunchAgents/ai.prometheus.exec.plist" \
PROMETHEUS_EXEC_SERVICE_TEST_STATE="$TMP_ROOT/state" \
bash "$REPO_ROOT/scripts/install-prometheus-exec-service.sh" >"$TMP_ROOT/install.out"

grep -Fq 'service installed and verified' "$TMP_ROOT/install.out"
[[ "$(cat "$TMP_ROOT/state/bootstrap-count")" == "2" ]]
[[ -f "$TMP_ROOT/state/loaded" ]]
[[ -f "$TMP_ROOT/home/.prometheus/exec/identity.json" ]]
grep -Fq "$TMP_ROOT/bin/prometheus-exec" \
  "$TMP_ROOT/home/Library/LaunchAgents/ai.prometheus.exec.plist"

echo 'PASS: prometheus-exec service retries launchd bootstrap after asynchronous bootout'
