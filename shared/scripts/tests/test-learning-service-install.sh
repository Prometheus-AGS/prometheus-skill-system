#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
OUTPUT="$(mktemp -d)"
trap 'rm -rf "$OUTPUT"' EXIT

bash "$ROOT/scripts/install-mcp-services.sh" --render-only "$OUTPUT" >/dev/null

for plist in \
  "$OUTPUT/ai.prometheus.learning-worker.plist" \
  "$OUTPUT/ai.prometheus.hooks-logrotate.plist"; do
  plutil -lint "$plist" >/dev/null
  ! grep -q '__[A-Z_]*__' "$plist"
done

grep -q '<key>WatchPaths</key>' "$OUTPUT/ai.prometheus.learning-worker.plist"
grep -q '<key>StartInterval</key>' "$OUTPUT/ai.prometheus.learning-worker.plist"
grep -q '<key>StartCalendarInterval</key>' "$OUTPUT/ai.prometheus.hooks-logrotate.plist"
grep -q '/Users/' "$OUTPUT/prometheus-hooks.conf"
grep -q 'rotate 30' "$OUTPUT/prometheus-hooks.conf"
grep -q 'create 0600' "$OUTPUT/prometheus-hooks.conf"
grep -q 'compress' "$OUTPUT/prometheus-hooks.conf"
! grep -q '~/' "$OUTPUT/prometheus-hooks.conf"

for unit in \
  ai.prometheus.learning-worker.service \
  ai.prometheus.learning-worker.path \
  ai.prometheus.learning-worker.timer \
  ai.prometheus.hooks-logrotate.service \
  ai.prometheus.hooks-logrotate.timer; do
  test -s "$OUTPUT/$unit"
  ! grep -q '__[A-Z_]*__' "$OUTPUT/$unit"
done

grep -q '\.hooks.lock' "$ROOT/shared/scripts/rotate-hooks-log.sh"

RECOVERY_OUTPUT=$(bash "$ROOT/scripts/install-mcp-services.sh" --learning-recovery --dry-run)
grep -q 'ai.prometheus.pk-cherry' <<<"$RECOVERY_OUTPUT"
grep -q 'ai.prometheus.learning-worker' <<<"$RECOVERY_OUTPUT"
grep -q 'ai.prometheus.hooks-logrotate' <<<"$RECOVERY_OUTPUT"
! grep -q 'sovereign-sync' <<<"$RECOVERY_OUTPUT"
printf 'Learning worker and locked hook rotation definitions render correctly.\n'
