#!/usr/bin/env bash
# change-cpc-009: prove the --exclude mechanism itself still works after
# sovereign-sync's removal, against real service names that still exist —
# not a fixture naming a service no script manages anymore. Also proves the
# control-plane-extension row (the optional, discovered replacement concept)
# is excludable the same way.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

install_output="$(bash "$ROOT/scripts/install-mcp-services.sh" \
  --dry-run --exclude surface-bridge 2>&1)"
if grep -q 'rendering ai.prometheus.surface-bridge' <<<"$install_output"; then
  echo "excluded installer output still rendered surface-bridge" >&2
  exit 1
fi

render_dir="$TMP/rendered"
render_output="$(bash "$ROOT/scripts/install-mcp-services.sh" \
  --render-only "$render_dir" --exclude surface-bridge 2>&1)"
if grep -q 'rendering ai.prometheus.surface-bridge' <<<"$render_output" || \
   find "$render_dir" -type f -name '*surface-bridge*' -print -quit | grep -q .; then
  echo "excluded render still wrote surface-bridge definitions" >&2
  exit 1
fi

health_output="$(bash "$ROOT/scripts/check-mcp-health.sh" \
  --json --exclude control-plane-extension)"
if grep -q 'control-plane-extension' <<<"$health_output"; then
  echo "excluded health output still referenced control-plane-extension" >&2
  exit 1
fi

services_output="$(bash "$ROOT/scripts/prometheus-services.sh" doctor \
  --exclude control-plane-extension 2>&1)"
if grep -q 'control-plane-extension' <<<"$services_output"; then
  echo "excluded services doctor still referenced control-plane-extension" >&2
  exit 1
fi

# change-cpc-009's own acceptance scenario: no doctor item or health probe may
# mention the removed daemon by name. The one deliberate exception is
# install-mcp-services.sh's one-time legacy-migration line, which archives a
# stale com.prometheusags.sovereign-sync LaunchAgent left by an old pack
# install — cleanup of an operator's leftover state, not ongoing daemon
# management, and it is filtered out here rather than silently exempted.
combined="$install_output
$render_output
$health_output
$services_output"
unexpected="$(grep -i 'sovereign-sync' <<<"$combined" | grep -v 'migrate legacy service com.prometheusags.sovereign-sync' || true)"
if [ -n "$unexpected" ]; then
  echo "a service script still mentions sovereign-sync after change-cpc-009:" >&2
  echo "$unexpected" >&2
  exit 1
fi

echo "service exclusion fixtures passed"
