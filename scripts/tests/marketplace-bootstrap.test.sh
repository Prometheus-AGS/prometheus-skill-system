#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SOURCE="$ROOT/dist/plugins/claude/prometheus-skill-pack"
FIXTURE="$(mktemp -d "${TMPDIR:-/tmp}/prometheus-marketplace-bootstrap.XXXXXX")"
cleanup() {
  status=$?
  if [[ "$status" -ne 0 ]]; then
    for output in first.out first.err second.out second.err install.out install.err hook.out hook.err verify.out; do
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
mkdir -p "$PAYLOAD" "$TEST_HOME" "$FIXTURE/consumer"
cp -R "$SOURCE/." "$PAYLOAD/"

fixture_env() {
  env -i PATH="$PATH" HOME="$TEST_HOME" USERPROFILE="$TEST_HOME" \
    CODEX_HOME="$TEST_HOME/.codex" CLAUDE_CONFIG_DIR="$TEST_HOME/.claude" \
    XDG_CONFIG_HOME="$TEST_HOME/.config" XDG_DATA_HOME="$TEST_HOME/.local/share" \
    XDG_CACHE_HOME="$TEST_HOME/.cache" XDG_STATE_HOME="$TEST_HOME/.local/state" \
    PROMETHEUS_PLUGIN_ROOT="$PLUGIN_ROOT" CLAUDE_PLUGIN_ROOT="$PAYLOAD" \
    GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null "$@"
}

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
VERSION="$(node -p 'require(process.argv[1]).releaseVersion' "$PAYLOAD/skill-system.json")"

# Both retained bootstrap commands and current hook commands must leave a clean
# installation untouched. Only the explicit installer below may create it.
status=0
fixture_env bash "$PAYLOAD/shared/scripts/bootstrap-hook-runtime.sh" \
  --source-root "$PAYLOAD" --expected-bundle "$BUNDLE" \
  >"$FIXTURE/first.out" 2>"$FIXTURE/first.err" || status=$?
test "$status" = 78
grep -q 'automatic activation disabled' "$FIXTURE/first.err"
test ! -e "$PLUGIN_ROOT"

status=0
fixture_env node "$PAYLOAD/scripts/hook-entry.mjs" --bundle "$BUNDLE" \
  --hook sessionstart-kbd-control --harness claude-code \
  >"$FIXTURE/second.out" 2>"$FIXTURE/second.err" || status=$?
test "$status" = 78
grep -q 'NOT_ACTIVATED' "$FIXTURE/second.err"
test ! -e "$PLUGIN_ROOT"

fixture_env node "$PAYLOAD/scripts/install-plugin-generation.js" \
  --source-root "$PAYLOAD" --plugin-root "$PLUGIN_ROOT" --home "$TEST_HOME" --targets all \
  >"$FIXTURE/install.out" 2>"$FIXTURE/install.err"

RUNNER="$PLUGIN_ROOT/runtime/v1/run-hook"
test -x "$RUNNER"
fixture_env bash "$RUNNER" --bundle "$BUNDLE" --resolve-only >/dev/null

cd "$FIXTURE/consumer"
fixture_env node "$PAYLOAD/scripts/hook-entry.mjs" --bundle "$BUNDLE" \
  --hook sessionstart-kbd-control --harness claude-code \
  < /dev/null >"$FIXTURE/hook.out" 2>"$FIXTURE/hook.err"
test ! -s "$FIXTURE/hook.err"

GENERATION="$(basename "$(readlink "$PLUGIN_ROOT/current")")"
fixture_env node "$PAYLOAD/scripts/install-plugin-generation.js" \
  --source-root "$PAYLOAD" --plugin-root "$PLUGIN_ROOT" --home "$TEST_HOME" --targets all --verify \
  >"$FIXTURE/verify.out"
grep -q "$GENERATION" "$FIXTURE/verify.out"

receipt_count="$(find "$PLUGIN_ROOT/receipts/$GENERATION" -type f -name '*.json' | wc -l | tr -d ' ')"
test "$receipt_count" = 14
test "$(node -p 'require(process.argv[1]).sourceVersion' "$PLUGIN_ROOT/current/manifest.json")" = "$VERSION"
test "$(node -p 'require(process.argv[1]).version' "$PAYLOAD/.claude-plugin/plugin.json")" = "$VERSION"
test ! -e "$PLUGIN_ROOT/pointers/activation.pending.json"

printf 'marketplace explicit activation verified bundle=%s generation=%s receipts=%s\n' \
  "$BUNDLE" "$GENERATION" "$receipt_count"
