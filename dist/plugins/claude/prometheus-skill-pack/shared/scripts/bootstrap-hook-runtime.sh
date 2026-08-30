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
# Acquire the bootstrap lock.
#
# This runs on the hot path of EVERY hook in EVERY session, so the failure mode
# matters more than the happy path. The lock is a mkdir mutex released by the
# EXIT trap below — but a SIGKILL, a crashed session, or a reboot leaves the
# directory behind with nothing to reap it. The previous implementation spun
# 600 times at 0.1s and then gave up, so one stale lock made every subsequent
# hook stall the full 60s before failing: the difference between "a hook did not
# run" and "the whole system feels broken".
#
# Fix: record the holder's PID inside the lock. A lock whose holder is gone is
# stale by definition and is broken immediately. A live holder is waited on, but
# for at most one installer transaction. A cold immutable generation installs
# all target receipts and can legitimately take tens of seconds on a busy host.
LOCK_PID_FILE="$LOCK/pid"
acquired=false
for _ in $(seq 1 600); do
  if mkdir "$LOCK" 2>/dev/null; then
    printf '%s\n' "$$" >"$LOCK_PID_FILE" 2>/dev/null || true
    acquired=true
    break
  fi

  # Lock exists. Break it if the holder is dead.
  holder="$(cat "$LOCK_PID_FILE" 2>/dev/null || true)"
  if [[ -z "$holder" ]]; then
    # No PID recorded yet. Either the holder is mid-write (a race we lose
    # harmlessly by retrying) or it died between mkdir and the write. Give it
    # one grace interval, then treat a still-empty lock as abandoned.
    sleep 0.1
    holder="$(cat "$LOCK_PID_FILE" 2>/dev/null || true)"
    [[ -n "$holder" ]] && continue
    rm -rf "$LOCK" 2>/dev/null || true
    continue
  fi

  if ! kill -0 "$holder" 2>/dev/null; then
    printf 'bootstrap-hook-runtime: breaking stale lock held by dead pid %s\n' \
      "$holder" >&2
    rm -rf "$LOCK" 2>/dev/null || true
    continue
  fi

  sleep 0.1
done
$acquired || {
  printf 'bootstrap-hook-runtime: could not acquire bootstrap lock (holder pid %s still alive)\n' \
    "$(cat "$LOCK_PID_FILE" 2>/dev/null || echo unknown)" >&2
  exit 75
}
trap 'rm -rf "$LOCK" 2>/dev/null || true' EXIT

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
