#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REAL_NODE="$(command -v node)"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/prometheus-update-test.XXXXXX")"
trap 'rm -rf "$TMP_ROOT"' EXIT

FIXTURE_REPO="$TMP_ROOT/repo"
FIXTURE_HOME="$TMP_ROOT/home"
FAKE_BIN="$TMP_ROOT/bin"
REMOTE="$TMP_ROOT/remote.git"
mkdir -p "$FIXTURE_REPO/scripts" "$FIXTURE_HOME" "$FAKE_BIN"
cp "$REPO_ROOT/scripts/update-skill-pack.sh" "$FIXTURE_REPO/scripts/update-skill-pack.sh"
touch "$FIXTURE_REPO/scripts/generate-harness-adapters.js"
touch "$FIXTURE_REPO/scripts/build-codex-plugin.js"
touch "$FIXTURE_REPO/scripts/install-plugin-generation.js"

cat > "$FIXTURE_REPO/scripts/refresh-native-plugin-installs.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'refresh %s\n' "$*" >> "${PROMETHEUS_TEST_LOG:?}"
if [[ "${PROMETHEUS_TEST_REFRESH_FAIL:-0}" == "1" ]]; then
  echo "fixture native refresh failed" >&2
  exit 1
fi
SH
chmod +x "$FIXTURE_REPO/scripts/refresh-native-plugin-installs.sh"

cat > "$FAKE_BIN/node" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
real_node="${PROMETHEUS_TEST_REAL_NODE:?}"
entry="${1:-}"
case "$entry" in
  *generate-harness-adapters.js)
    printf 'generated-check %s\n' "$*" >> "${PROMETHEUS_TEST_LOG:?}"
    [[ "${PROMETHEUS_TEST_STALE_GENERATED:-0}" != "1" ]]
    ;;
  *build-codex-plugin.js)
    printf 'codex-check %s\n' "$*" >> "${PROMETHEUS_TEST_LOG:?}"
    ;;
  *install-plugin-generation.js)
    printf 'installer %s\n' "$*" >> "${PROMETHEUS_TEST_LOG:?}"
    if [[ " $* " == *" --verify "* ]]; then
      echo fixture-generation
      exit 0
    fi
    [[ " $* " == *" --require-clean-source "* ]]
    expected=""
    plugin_root=""
    while [[ $# -gt 0 ]]; do
      case "$1" in
        --expected-source-commit) expected="$2"; shift 2 ;;
        --plugin-root) plugin_root="$2"; shift 2 ;;
        *) shift ;;
      esac
    done
    [[ -n "$expected" && -n "$plugin_root" ]]
    if [[ "${PROMETHEUS_TEST_SOURCE_CHANGED:-0}" == "1" ]]; then
      echo "install-plugin-generation: source provenance changed while staging payload" >&2
      exit 1
    fi
    mkdir -p "$plugin_root/generations/fixture-generation"
    "$real_node" -e '
const fs=require("fs"), path=require("path");
const [root,commit]=process.argv.slice(1);
fs.writeFileSync(path.join(root,"generations/fixture-generation/manifest.json"), JSON.stringify({
 generation:"fixture-generation",
 sourceProvenance:{sourceCommit:commit,sourceTreeState:"clean"},
 targetPayloads:Array.from({length:14},(_,index)=>({target:String(index)}))
})+"\n");
' "$plugin_root" "$expected"
    ln -sfn generations/fixture-generation "$plugin_root/current"
    echo fixture-generation
    ;;
  -) exec "$real_node" "$@" ;;
  *) exec "$real_node" "$@" ;;
esac
SH
chmod +x "$FAKE_BIN/node"

git -C "$FIXTURE_REPO" init -b main >/dev/null
git -C "$FIXTURE_REPO" config user.name "Update Fixture"
git -C "$FIXTURE_REPO" config user.email "fixture@example.invalid"
git -C "$FIXTURE_REPO" add .
git -C "$FIXTURE_REPO" commit -m "fixture" >/dev/null
git init --bare "$REMOTE" >/dev/null
git -C "$FIXTURE_REPO" remote add origin "$REMOTE"
git -C "$FIXTURE_REPO" push -u origin main >/dev/null

export PATH="$FAKE_BIN:$PATH"
export PROMETHEUS_TEST_REAL_NODE="$REAL_NODE"
export PROMETHEUS_TEST_LOG="$TMP_ROOT/commands.log"
receipt="$FIXTURE_HOME/.prometheus/skill-pack-install-ref"
mkdir -p "$(dirname "$receipt")"

printf 'old-receipt\n' > "$receipt"
touch "$FIXTURE_REPO/dirty.txt"
if HOME="$FIXTURE_HOME" bash "$FIXTURE_REPO/scripts/update-skill-pack.sh" --force \
  >"$TMP_ROOT/dirty.out" 2>"$TMP_ROOT/dirty.err"; then
  echo "FAIL: updater accepted a dirty source with --force" >&2
  exit 1
fi
grep -Fq 'refusing update from a dirty source tree (before pull)' "$TMP_ROOT/dirty.err"
[[ "$(cat "$receipt")" == "old-receipt" ]]
rm -f "$FIXTURE_REPO/dirty.txt"

printf 'old-receipt\n' > "$receipt"
if PROMETHEUS_TEST_STALE_GENERATED=1 HOME="$FIXTURE_HOME" \
  bash "$FIXTURE_REPO/scripts/update-skill-pack.sh" >"$TMP_ROOT/stale.out" 2>"$TMP_ROOT/stale.err"; then
  echo "FAIL: updater accepted stale generated artifacts" >&2
  exit 1
fi
[[ "$(cat "$receipt")" == "old-receipt" ]]
if grep -Fq 'installer ' "$PROMETHEUS_TEST_LOG"; then
  echo "FAIL: installer ran after stale generated-artifact failure" >&2
  exit 1
fi

: > "$PROMETHEUS_TEST_LOG"
printf 'old-receipt\n' > "$receipt"
if PROMETHEUS_TEST_SOURCE_CHANGED=1 HOME="$FIXTURE_HOME" \
  bash "$FIXTURE_REPO/scripts/update-skill-pack.sh" >"$TMP_ROOT/race.out" 2>"$TMP_ROOT/race.err"; then
  echo "FAIL: updater accepted source provenance changing during staging" >&2
  exit 1
fi
grep -Fq 'source provenance changed while staging payload' "$TMP_ROOT/race.err"
[[ "$(cat "$receipt")" == "old-receipt" ]]

: > "$PROMETHEUS_TEST_LOG"
printf 'old-receipt\n' > "$receipt"
if PROMETHEUS_TEST_REFRESH_FAIL=1 HOME="$FIXTURE_HOME" \
  bash "$FIXTURE_REPO/scripts/update-skill-pack.sh" >"$TMP_ROOT/refresh.out" 2>"$TMP_ROOT/refresh.err"; then
  echo "FAIL: updater accepted a detected native refresh failure" >&2
  exit 1
fi
grep -Fq 'fixture native refresh failed' "$TMP_ROOT/refresh.err"
[[ "$(cat "$receipt")" == "old-receipt" ]]

: > "$PROMETHEUS_TEST_LOG"
HOME="$FIXTURE_HOME" bash "$FIXTURE_REPO/scripts/update-skill-pack.sh" \
  >"$TMP_ROOT/success.out" 2>"$TMP_ROOT/success.err"
expected_commit="$(git -C "$FIXTURE_REPO" rev-parse HEAD)"
[[ "$(cat "$receipt")" == "$expected_commit" ]]
grep -Fq 'receipt advanced after all detected surfaces verified' "$TMP_ROOT/success.out"
grep -Fq -- '--require-clean-source' "$PROMETHEUS_TEST_LOG"
grep -Fq -- "--expected-source-commit $expected_commit" "$PROMETHEUS_TEST_LOG"
grep -Fq 'refresh --source-root' "$PROMETHEUS_TEST_LOG"

echo "PASS: updater clean-source, generated-artifact, provenance, and deferred-receipt policies"
