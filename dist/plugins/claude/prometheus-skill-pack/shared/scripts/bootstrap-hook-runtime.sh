#!/usr/bin/env bash
# Acquire a verified immutable hook bundle from a native plugin payload.
# No business hook runs from the marketplace cache.
set -euo pipefail

SOURCE_ROOT=""
EXPECTED_BUNDLE=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --source-root) SOURCE_ROOT="${2:-}"; shift 2 ;;
    --expected-bundle) EXPECTED_BUNDLE="${2:-}"; shift 2 ;;
    *) printf 'bootstrap-hook-runtime: unknown argument: %s\n' "$1" >&2; exit 64 ;;
  esac
done

[[ -n "$SOURCE_ROOT" && -d "$SOURCE_ROOT" ]] || {
  printf '{"status":"NOT_ACTIVATED","reason":"source payload unavailable"}\n' >&2
  exit 78
}
[[ "$EXPECTED_BUNDLE" =~ ^[a-f0-9]{64}$ ]] || {
  printf '{"status":"NOT_ACTIVATED","reason":"invalid expected bundle"}\n' >&2
  exit 78
}

RELEASE_MANIFEST="$SOURCE_ROOT/shared/harnesses/generated/release-manifest.json"
INSTALLER="$SOURCE_ROOT/scripts/install-plugin-generation.js"
[[ -f "$RELEASE_MANIFEST" && -f "$INSTALLER" ]] || {
  printf '{"status":"NOT_ACTIVATED","reason":"bootstrap payload incomplete"}\n' >&2
  exit 78
}
ACTUAL_BUNDLE="$(awk -F'"' '/"bundleId"[[:space:]]*:/ { print $4; exit }' "$RELEASE_MANIFEST")"
[[ "$ACTUAL_BUNDLE" == "$EXPECTED_BUNDLE" ]] || {
  printf '{"status":"NOT_ACTIVATED","reason":"bundle identity mismatch"}\n' >&2
  exit 78
}

PLUGIN_ROOT="${PROMETHEUS_PLUGIN_ROOT:-$HOME/.prometheus/plugins/prometheus-skill-pack}"
LOCK="$PLUGIN_ROOT/.bootstrap-lock"
mkdir -p "$PLUGIN_ROOT"
acquired=false
for _ in $(seq 1 600); do
  if mkdir "$LOCK" 2>/dev/null; then
    acquired=true
    break
  fi
  sleep 0.1
done
$acquired || { printf 'bootstrap-hook-runtime: acquisition lock timed out\n' >&2; exit 75; }
trap 'rmdir "$LOCK" 2>/dev/null || true' EXIT

RUNNER="$PLUGIN_ROOT/runtime/v1/run-hook"
if [[ -x "$RUNNER" ]] && "$RUNNER" --bundle "$EXPECTED_BUNDLE" --resolve-only >/dev/null 2>&1; then
  exit 0
fi

node "$INSTALLER" \
  --source-root "$SOURCE_ROOT" \
  --plugin-root "$PLUGIN_ROOT" \
  --home "$HOME" \
  --expected-bundle "$EXPECTED_BUNDLE" >/dev/null

"$RUNNER" --bundle "$EXPECTED_BUNDLE" --resolve-only >/dev/null
