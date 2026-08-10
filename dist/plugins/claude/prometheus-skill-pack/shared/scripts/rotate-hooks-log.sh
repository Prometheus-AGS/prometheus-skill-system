#!/usr/bin/env bash
# Rotate hook JSONL under the same lock used by every hook writer.
set -euo pipefail
umask 077

PROMETHEUS_DIR="${PROMETHEUS_HOME_DIR:-$HOME/.prometheus}"
CONFIG="${PROMETHEUS_LOGROTATE_CONFIG:-$PROMETHEUS_DIR/logrotate/prometheus-hooks.conf}"
STATE="${PROMETHEUS_LOGROTATE_STATE:-$PROMETHEUS_DIR/logrotate/status}"
LOCK="$PROMETHEUS_DIR/.hooks.lock"
LOGROTATE_BIN="${PROMETHEUS_LOGROTATE_BIN:-$(command -v logrotate 2>/dev/null || true)}"
FLOCK_BIN="${PROMETHEUS_FLOCK_BIN:-$(command -v flock 2>/dev/null || true)}"

[ -x "$LOGROTATE_BIN" ] || { echo "logrotate binary unavailable" >&2; exit 1; }
[ -x "$FLOCK_BIN" ] || { echo "flock binary unavailable" >&2; exit 1; }
[ -r "$CONFIG" ] || { echo "logrotate config unavailable: $CONFIG" >&2; exit 1; }

mkdir -p "$PROMETHEUS_DIR/logrotate"
touch "$LOCK"
chmod 700 "$PROMETHEUS_DIR" "$PROMETHEUS_DIR/logrotate"
chmod 600 "$LOCK" "$CONFIG"

exec "$FLOCK_BIN" -x "$LOCK" "$LOGROTATE_BIN" --state "$STATE" "$@" "$CONFIG"
