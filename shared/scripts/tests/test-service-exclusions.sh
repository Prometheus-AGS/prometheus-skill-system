#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

install_output="$(bash "$ROOT/scripts/install-mcp-services.sh" \
  --dry-run --learning-recovery --exclude sovereign-sync 2>&1)"
if grep -q 'sovereign-sync' <<<"$install_output"; then
  echo "excluded installer output referenced sovereign-sync" >&2
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
