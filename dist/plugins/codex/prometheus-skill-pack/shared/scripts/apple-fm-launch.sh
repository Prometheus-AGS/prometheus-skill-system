#!/bin/bash
# apple-fm-launch.sh — launchd entry for ai.prometheus.apple-fm.
# Runs Apple's on-device Foundation Model as a loopback OpenAI-compatible server
# (`apfel --serve`). Bash 3.2 compatible (launchd uses /bin/bash).
set -u
PORT="${PROMETHEUS_APPLE_FM_PORT:-11437}"
APFEL="$(command -v apfel 2>/dev/null || true)"
for candidate in /opt/homebrew/bin/apfel /usr/local/bin/apfel; do
  [ -n "$APFEL" ] && break
  [ -x "$candidate" ] && APFEL="$candidate"
done
if [ -z "$APFEL" ]; then
  echo "apple-fm-launch: apfel not found; run shared/scripts/apple-foundation-models.sh --install" >&2
  exit 1
fi
exec "$APFEL" --serve --host 127.0.0.1 --port "$PORT"
