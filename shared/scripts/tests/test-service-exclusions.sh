#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

install_output="$(bash "$ROOT/scripts/install-mcp-services.sh" \
  --dry-run --exclude sovereign-sync 2>&1)"
if grep -q 'sovereign-sync' <<<"$install_output"; then
  echo "excluded installer output referenced sovereign-sync" >&2
  exit 1
fi

render_dir="$TMP/rendered"
render_output="$(bash "$ROOT/scripts/install-mcp-services.sh" \
  --render-only "$render_dir" --exclude sovereign-sync 2>&1)"
if grep -q 'sovereign-sync' <<<"$render_output" || \
   find "$render_dir" -type f -name '*sovereign-sync*' -print -quit | grep -q .; then
  echo "excluded render rewrote sovereign-sync definitions" >&2
  exit 1
fi

health_output="$(bash "$ROOT/scripts/check-mcp-health.sh" \
  --json --exclude sovereign-sync)"
if grep -q 'sovereign-sync' <<<"$health_output"; then
  echo "excluded health output referenced sovereign-sync" >&2
  exit 1
fi

services_output="$(bash "$ROOT/scripts/prometheus-services.sh" doctor \
  --exclude sovereign-sync 2>&1)"
if grep -q 'sovereign-sync' <<<"$services_output"; then
  echo "excluded services doctor referenced sovereign-sync" >&2
  exit 1
fi

echo "service exclusion fixtures passed"
