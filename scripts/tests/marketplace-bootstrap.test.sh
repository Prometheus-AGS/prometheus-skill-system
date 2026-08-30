#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SOURCE="$ROOT/dist/plugins/claude/prometheus-skill-pack"
FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/prometheus-marketplace-bootstrap.XXXXXX")"
cleanup() {
  status=$?
  if [[ "$status" -ne 0 ]]; then
    for output in first.out first.err second.out second.err; do
      if [[ -f "$FIXTURE/$output" ]]; then
        printf '%s:\n' "$output" >&2
        sed -n '1,240p' "$FIXTURE/$output" >&2
      fi
    done
  fi
  rm -rf "$FIXTURE"
  trap - EXIT
  exit "$status"
}
trap cleanup EXIT

PAYLOAD="$FIXTURE/payload"
TEST_HOME="$FIXTURE/home"
PLUGIN_ROOT="$TEST_HOME/.prometheus/plugins/prometheus-skill-pack"
mkdir -p "$PAYLOAD" "$TEST_HOME" "$PLUGIN_ROOT/.bootstrap-lock"
cp -R "$SOURCE/." "$PAYLOAD/"

for required in \
  scripts/install-plugin-generation.js \
  scripts/lib/skill-system.js \
  package.json \
  skill-system.json \
  config/prometheus-exec-component.json \
  shared/harnesses/generated/release-manifest.json; do
  test -f "$PAYLOAD/$required"
done
test ! -e "$PAYLOAD/.git"
test ! -e "$PAYLOAD/dist"

BUNDLE="$(node -e 'const fs=require("fs"); const p=process.argv[1]; process.stdout.write(JSON.parse(fs.readFileSync(p)).bundleId)' "$PAYLOAD/shared/harnesses/generated/release-manifest.json")"
printf '999999\n' > "$PLUGIN_ROOT/.bootstrap-lock/pid"

HOME="$TEST_HOME" PROMETHEUS_PLUGIN_ROOT="$PLUGIN_ROOT" \
  bash "$PAYLOAD/shared/scripts/bootstrap-hook-runtime.sh" \
    --source-root "$PAYLOAD" --expected-bundle "$BUNDLE" >"$FIXTURE/first.out" 2>"$FIXTURE/first.err" &
first_pid=$!
HOME="$TEST_HOME" PROMETHEUS_PLUGIN_ROOT="$PLUGIN_ROOT" \
  bash "$PAYLOAD/shared/scripts/bootstrap-hook-runtime.sh" \
    --source-root "$PAYLOAD" --expected-bundle "$BUNDLE" >"$FIXTURE/second.out" 2>"$FIXTURE/second.err" &
second_pid=$!
wait "$first_pid"
wait "$second_pid"

RUNNER="$PLUGIN_ROOT/runtime/v1/run-hook"
test -x "$RUNNER"
HOME="$TEST_HOME" "$RUNNER" --bundle "$BUNDLE" --resolve-only >/dev/null

GENERATION="$(basename "$(readlink "$PLUGIN_ROOT/current")")"
HOME="$TEST_HOME" node "$PAYLOAD/scripts/install-plugin-generation.js" \
  --source-root "$PAYLOAD" --plugin-root "$PLUGIN_ROOT" --home "$TEST_HOME" --verify \
  | grep -q "$GENERATION"

receipt_count="$(find "$PLUGIN_ROOT/receipts/$GENERATION" -type f -name '*.json' | wc -l | tr -d ' ')"
test "$receipt_count" = 14
test "$(node -p 'require(process.argv[1]).sourceVersion' "$PLUGIN_ROOT/current/manifest.json")" = "1.8.0"
test "$(node -p 'require(process.argv[1]).version' "$PAYLOAD/.claude-plugin/plugin.json")" = "1.8.0"

if rg -q 'NOT_ACTIVATED' "$FIXTURE/first.err" "$FIXTURE/second.err"; then
  printf 'marketplace bootstrap unexpectedly reported NOT_ACTIVATED\n' >&2
  exit 1
fi

printf 'marketplace bootstrap verified bundle=%s generation=%s receipts=%s\n' \
  "$BUNDLE" "$GENERATION" "$receipt_count"
