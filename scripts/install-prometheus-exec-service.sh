#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXEC_BIN="${PROMETHEUS_EXEC_BIN:-${HOME}/.local/bin/prometheus-exec}"
EXEC_ROOT="${PROMETHEUS_EXEC_ROOT:-${HOME}/.prometheus/exec}"
EXEC_SOCKET="${PROMETHEUS_EXEC_SOCKET:-${HOME}/.prometheus/run/prometheus-exec.sock}"
EXEC_IDENTITY="${PROMETHEUS_EXEC_IDENTITY:-${EXEC_ROOT}/identity.json}"
PLUGIN_ROOT="${PROMETHEUS_EXEC_PLUGIN_ROOT:-${HOME}/.prometheus/plugins/prometheus-skill-pack}"
LOG_DIR="${PROMETHEUS_EXEC_LOG_DIR:-${HOME}/.prometheus/logs}"
LAUNCH_AGENT="${PROMETHEUS_EXEC_LAUNCH_AGENT:-${HOME}/Library/LaunchAgents/ai.prometheus.exec.plist}"
TEMPLATE="${REPO_ROOT}/shared/launchagents/ai.prometheus.exec.plist"
DRY_RUN=false
LOAD=true

while [ "$#" -gt 0 ]; do
    case "$1" in
        --dry-run) DRY_RUN=true ;;
        --no-load) LOAD=false ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done

if $DRY_RUN; then
    echo "[dry-run] would verify $EXEC_BIN, create a private identity if absent, install $LAUNCH_AGENT, and load ai.prometheus.exec"
    exit 0
fi

[ -x "$EXEC_BIN" ] || { echo "prometheus-exec is not installed: $EXEC_BIN" >&2; exit 1; }
[ "$("$EXEC_BIN" --version)" = "prometheus-exec 1.7.0" ] || {
    echo "installed prometheus-exec version is not 1.7.0" >&2
    exit 1
}
mkdir -p "$EXEC_ROOT" "$(dirname "$EXEC_SOCKET")" "$LOG_DIR" "$(dirname "$LAUNCH_AGENT")"
chmod 700 "$EXEC_ROOT" "$(dirname "$EXEC_SOCKET")" "$LOG_DIR"
if [ ! -f "$EXEC_IDENTITY" ]; then
    "$EXEC_BIN" init --identity "$EXEC_IDENTITY" >/dev/null
fi

escape_sed() { printf '%s' "$1" | sed 's/[&|]/\\&/g'; }
stage="$(mktemp "$(dirname "$LAUNCH_AGENT")/.prometheus-exec.plist.XXXXXX")"
trap 'rm -f "$stage"' EXIT INT TERM
sed \
    -e "s|__PROMETHEUS_EXEC_BIN__|$(escape_sed "$EXEC_BIN")|g" \
    -e "s|__PROMETHEUS_EXEC_SOCKET__|$(escape_sed "$EXEC_SOCKET")|g" \
    -e "s|__PROMETHEUS_EXEC_STATE__|$(escape_sed "$EXEC_ROOT")|g" \
    -e "s|__PROMETHEUS_EXEC_IDENTITY__|$(escape_sed "$EXEC_IDENTITY")|g" \
    -e "s|__PROMETHEUS_PLUGIN_ROOT__|$(escape_sed "$PLUGIN_ROOT")|g" \
    -e "s|__PROMETHEUS_LOG_DIR__|$(escape_sed "$LOG_DIR")|g" \
    "$TEMPLATE" >"$stage"
plutil -lint "$stage" >/dev/null
chmod 644 "$stage"
mv -f "$stage" "$LAUNCH_AGENT"
stage=""

if $LOAD; then
    uid="$(id -u)"
    launchctl bootout "gui/${uid}/ai.prometheus.exec" >/dev/null 2>&1 || true
    launchctl bootstrap "gui/${uid}" "$LAUNCH_AGENT"
    launchctl kickstart -k "gui/${uid}/ai.prometheus.exec"
    launchctl print "gui/${uid}/ai.prometheus.exec" >/dev/null
fi

echo "prometheus-exec service installed and verified: $LAUNCH_AGENT"
