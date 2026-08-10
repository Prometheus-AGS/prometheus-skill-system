#!/usr/bin/env bash
# Stable hook runtime ABI. Hook commands bind to an immutable bundle id and
# never resolve through the mutable current generation.
set -euo pipefail

BUNDLE_ID=""
HOOK_ID=""
HARNESS=""
RESOLVE_ONLY=false

fail() {
  local code="$1"
  local message="$2"
  printf '{"status":"HOOK_RUNTIME_ERROR","code":"%s","message":"%s","bundle":"%s"}\n' \
    "$code" "$message" "$BUNDLE_ID" >&2
  exit 78
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bundle) BUNDLE_ID="${2:-}"; shift 2 ;;
    --hook) HOOK_ID="${2:-}"; shift 2 ;;
    --harness) HARNESS="${2:-}"; shift 2 ;;
    --resolve-only) RESOLVE_ONLY=true; shift ;;
    *) fail "INVALID_ARGUMENT" "unknown argument" ;;
  esac
done

[[ "$BUNDLE_ID" =~ ^[a-f0-9]{64}$ ]] || fail "INVALID_BUNDLE" "bundle id is not sha256"

PLUGIN_ROOT="${PROMETHEUS_PLUGIN_ROOT:-$HOME/.prometheus/plugins/prometheus-skill-pack}"
BUNDLE_LINK="$PLUGIN_ROOT/bundles/$BUNDLE_ID"
[[ -L "$BUNDLE_LINK" ]] || fail "NOT_ACTIVATED" "bundle index is missing"

GENERATION_ROOT="$(cd "$BUNDLE_LINK" 2>/dev/null && pwd -P)" || \
  fail "BROKEN_BUNDLE" "bundle index cannot be resolved"
GENERATIONS_ROOT="$(cd "$PLUGIN_ROOT/generations" 2>/dev/null && pwd -P)" || \
  fail "BROKEN_STORE" "generation store is missing"
case "$GENERATION_ROOT" in
  "$GENERATIONS_ROOT"/*) ;;
  *) fail "ESCAPING_BUNDLE" "bundle index escapes generation store" ;;
esac

GENERATION_NAME="${GENERATION_ROOT##*/}"
[[ "$GENERATION_NAME" =~ ^[a-f0-9]{64}$ ]] || \
  fail "INVALID_GENERATION" "generation directory is not sha256"
MANIFEST="$GENERATION_ROOT/manifest.json"
[[ -f "$MANIFEST" ]] || fail "MISSING_MANIFEST" "generation manifest is missing"

manifest_value() {
  local key="$1"
  awk -F'"' -v wanted="\"$key\"" '$0 ~ wanted "[[:space:]]*:" { print $4; exit }' "$MANIFEST"
}

MANIFEST_BUNDLE="$(manifest_value bundleId)"
MANIFEST_GENERATION="$(manifest_value generation)"
MANIFEST_ABI="$(manifest_value abi)"
DISPATCHER_PATH="$(manifest_value dispatcherPath)"
DISPATCHER_SHA="$(manifest_value dispatcherSha256)"

[[ "$MANIFEST_BUNDLE" == "$BUNDLE_ID" ]] || fail "BUNDLE_MISMATCH" "manifest bundle differs"
[[ "$MANIFEST_GENERATION" == "$GENERATION_NAME" ]] || \
  fail "GENERATION_MISMATCH" "manifest generation differs"
[[ "$MANIFEST_ABI" == "hook-runtime-v1" ]] || fail "ABI_MISMATCH" "unsupported dispatcher ABI"
[[ "$DISPATCHER_PATH" == "shared/scripts/generated/hook-dispatch-v1.sh" ]] || \
  fail "DISPATCHER_PATH" "dispatcher path is not allowlisted"

DISPATCHER="$GENERATION_ROOT/$DISPATCHER_PATH"
[[ -x "$DISPATCHER" ]] || fail "MISSING_DISPATCHER" "dispatcher is missing or not executable"
ACTUAL_DISPATCHER_SHA="$(shasum -a 256 "$DISPATCHER" | awk '{print $1}')"
[[ "$ACTUAL_DISPATCHER_SHA" == "$DISPATCHER_SHA" ]] || \
  fail "DISPATCHER_HASH" "dispatcher hash differs"

if $RESOLVE_ONLY; then
  printf '{"status":"ok","bundle":"%s","generation":"%s","abi":"hook-runtime-v1"}\n' \
    "$BUNDLE_ID" "$GENERATION_NAME"
  exit 0
fi

[[ -n "$HOOK_ID" ]] || fail "MISSING_HOOK" "hook id is required"
[[ -n "$HARNESS" ]] || fail "MISSING_HARNESS" "harness is required"
exec "$DISPATCHER" --hook "$HOOK_ID" --harness "$HARNESS"
