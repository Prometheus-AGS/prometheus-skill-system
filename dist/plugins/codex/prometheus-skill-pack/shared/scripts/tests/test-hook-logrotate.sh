#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
FIXTURE="$(mktemp -d)"
trap 'rm -rf "$FIXTURE"' EXIT
PROM_DIR="$FIXTURE/.prometheus"
mkdir -p "$PROM_DIR/logrotate"

printf '%s\n' '{"event":"first"}' > "$PROM_DIR/hooks.log"
chmod 600 "$PROM_DIR/hooks.log"
printf '%s\n' \
  "$PROM_DIR/hooks.log {" \
  '  size 1' \
  '  rotate 30' \
  '  compress' \
  '  delaycompress' \
  '  missingok' \
  '  notifempty' \
  '  create 0600' \
  '}' > "$PROM_DIR/logrotate/prometheus-hooks.conf"

run_rotation() {
  HOME="$FIXTURE" \
  PROMETHEUS_HOME_DIR="$PROM_DIR" \
  PROMETHEUS_LOGROTATE_BIN="$(command -v logrotate)" \
  PROMETHEUS_FLOCK_BIN="$(command -v flock)" \
  "$ROOT/shared/scripts/rotate-hooks-log.sh" --force
}

run_rotation
printf '%s\n' '{"event":"second"}' >> "$PROM_DIR/hooks.log"
run_rotation

test "$(stat -f '%Lp' "$PROM_DIR/hooks.log")" = 600
test -f "$PROM_DIR/hooks.log.1"
test -f "$PROM_DIR/hooks.log.2.gz"
jq -e . "$PROM_DIR/hooks.log.1" >/dev/null
gzip -cd "$PROM_DIR/hooks.log.2.gz" | jq -e . >/dev/null
printf 'Locked hook rotation retains 30 archives and compresses on the second cycle.\n'
