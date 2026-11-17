#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/prometheus-native-refresh.XXXXXX")"
trap 'rm -rf "$TMP_ROOT"' EXIT

TEST_HOME="$TMP_ROOT/home"
FAKE_BIN="$TMP_ROOT/bin"
LOG="$TMP_ROOT/commands.log"
GENERATION="fixture-generation"
SOURCE_KBD="$REPO_ROOT/skills/process/kbd-process-orchestrator/skills/kbd-init"
PLUGIN_ROOT="$TEST_HOME/.prometheus/plugins/prometheus-skill-pack"
# Read from the repository, never pinned here: a pinned version goes stale the
# next time the release or a plugin is bumped, and then this test stops at the
# minimum-version check before it reaches anything it is meant to cover.
PACK_VERSION="$(node -p 'require(process.argv[1]).releaseVersion' "$REPO_ROOT/skill-system.json")"
PROCESS_VERSION="$(node -p 'require(process.argv[1]).version' "$REPO_ROOT/skills/process/.claude-plugin/plugin.json")"
mkdir -p "$FAKE_BIN" "$PLUGIN_ROOT/generations/$GENERATION/skills/process/kbd-process-orchestrator/skills"
cp -R "$SOURCE_KBD" "$PLUGIN_ROOT/generations/$GENERATION/skills/process/kbd-process-orchestrator/skills/kbd-init"
cp -R "$SOURCE_KBD" "$PLUGIN_ROOT/generations/$GENERATION/skills/kbd-init"
printf '{"sourceVersion":"%s"}\n' "$PACK_VERSION" > "$PLUGIN_ROOT/generations/$GENERATION/manifest.json"
ln -s "generations/$GENERATION" "$PLUGIN_ROOT/current"

UMBRELLA_INSTALL="$TEST_HOME/native/umbrella"
PROCESS_INSTALL="$TEST_HOME/native/process"
mkdir -p "$UMBRELLA_INSTALL/skills" "$PROCESS_INSTALL/kbd-process-orchestrator/skills"
cp -R "$SOURCE_KBD" "$UMBRELLA_INSTALL/skills/kbd-init"
# A hook carrier is verified against the hook file this checkout would install.
mkdir -p "$UMBRELLA_INSTALL/hooks"
cp "$REPO_ROOT/dist/plugins/claude/prometheus-skill-pack/hooks/hooks.json" "$UMBRELLA_INSTALL/hooks/hooks.json"
cp -R "$SOURCE_KBD" "$PROCESS_INSTALL/kbd-process-orchestrator/skills/kbd-init"

cat > "$TMP_ROOT/claude-before.json" <<JSON
[
  {"id":"prometheus-skill-pack@prometheus-skill-pack","scope":"user","version":"1.6.2","installPath":"$TEST_HOME/native/old-umbrella"},
  {"id":"prometheus-process-skills@prometheus-skill-pack","scope":"user","version":"1.5.0","installPath":"$TEST_HOME/native/old-process"}
]
JSON
cat > "$TMP_ROOT/claude-after.json" <<JSON
[
  {"id":"prometheus-skill-pack@prometheus-skill-pack","scope":"user","version":"$PACK_VERSION","installPath":"$UMBRELLA_INSTALL"},
  {"id":"prometheus-process-skills@prometheus-skill-pack","scope":"user","version":"$PROCESS_VERSION","installPath":"$PROCESS_INSTALL"}
]
JSON

cat > "$FAKE_BIN/claude" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'claude %s\n' "$*" >> "${PROMETHEUS_NATIVE_TEST_LOG:?}"
if [[ "$*" == "plugin list --json" ]]; then
  if [[ -f "${PROMETHEUS_NATIVE_TEST_UPDATED:?}" ]]; then
    if [[ -f "${PROMETHEUS_NATIVE_TEST_STALE_SEEN:?}" ]]; then
      cat "${PROMETHEUS_NATIVE_TEST_AFTER:?}"
    else
      touch "${PROMETHEUS_NATIVE_TEST_STALE_SEEN:?}"
      cat "${PROMETHEUS_NATIVE_TEST_BEFORE:?}"
    fi
  else
    cat "${PROMETHEUS_NATIVE_TEST_BEFORE:?}"
  fi
  exit 0
fi
if [[ "$*" == "plugin marketplace update prometheus-skill-pack" ]]; then
  exit 0
fi
if [[ "$*" == plugin\ uninstall\ --scope\ user\ * ]]; then
  exit 0
fi
if [[ "$*" == plugin\ install\ --scope\ user\ * ]]; then
  touch "${PROMETHEUS_NATIVE_TEST_UPDATED:?}"
  exit 0
fi
if [[ "$*" == plugin\ update\ --scope\ user\ * ]]; then
  touch "${PROMETHEUS_NATIVE_TEST_UPDATED:?}"
  exit 0
fi
exit 2
SH

cat > "$FAKE_BIN/codex" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'codex %s\n' "$*" >> "${PROMETHEUS_NATIVE_TEST_LOG:?}"
if [[ "$*" == "plugin marketplace list --json" ]]; then
  printf '{"marketplaces":[{"name":"prometheus-skill-pack"}]}\n'
  exit 0
fi
if [[ "$*" == plugin\ add\ *\ --json ]]; then
  printf '{"installed":true}\n'
  exit 0
fi
if [[ "$*" == "plugin list --json" ]]; then
  printf '{"installed":[
    {"pluginId":"prometheus-skill-pack@prometheus-skill-pack","version":"%s","installed":true,"enabled":true,"source":{"path":"%s/dist/plugins/codex/prometheus-skill-pack"}},
    {"pluginId":"prometheus-process-skills@prometheus-skill-pack","version":"%s","installed":true,"enabled":true,"source":{"path":"%s/skills/process"}}
  ]}\n' "${PROMETHEUS_NATIVE_TEST_PACK_VERSION:?}" "${PROMETHEUS_NATIVE_TEST_REPO:?}" "${PROMETHEUS_NATIVE_TEST_PROCESS_VERSION:?}" "${PROMETHEUS_NATIVE_TEST_REPO:?}"
  exit 0
fi
exit 2
SH

cat > "$FAKE_BIN/kimi-installer" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'kimi install\n' >> "${PROMETHEUS_NATIVE_TEST_LOG:?}"
dest="$HOME/Library/Application Support/kimi-desktop/daimon-share/daimon/plugin-packages/prometheus-skill-pack/skills"
mkdir -p "$dest"
cp -R "${PROMETHEUS_NATIVE_TEST_REPO:?}/skills/process/kbd-process-orchestrator/skills/kbd-init" "$dest/kbd-init"
SH

cat > "$FAKE_BIN/launchctl" <<'SH'
#!/usr/bin/env bash
printf 'launchctl %s\n' "$*" >> "${PROMETHEUS_NATIVE_TEST_LOG:?}"
exit 0
SH
chmod +x "$FAKE_BIN/claude" "$FAKE_BIN/codex" "$FAKE_BIN/kimi-installer" "$FAKE_BIN/launchctl"

kimi_packages="$TEST_HOME/Library/Application Support/kimi-desktop/daimon-share/daimon/plugin-packages"
mkdir -p "$kimi_packages" "$TEST_HOME/Library/LaunchAgents"
touch "$TEST_HOME/Library/LaunchAgents/ai.prometheus.codex-skills-sync.plist"

export PATH="$FAKE_BIN:$PATH"
export PROMETHEUS_NATIVE_TEST_LOG="$LOG"
export PROMETHEUS_NATIVE_TEST_UPDATED="$TMP_ROOT/updated"
export PROMETHEUS_NATIVE_TEST_STALE_SEEN="$TMP_ROOT/stale-seen"
export PROMETHEUS_NATIVE_TEST_BEFORE="$TMP_ROOT/claude-before.json"
export PROMETHEUS_NATIVE_TEST_AFTER="$TMP_ROOT/claude-after.json"
export PROMETHEUS_NATIVE_TEST_REPO="$REPO_ROOT"
export PROMETHEUS_NATIVE_TEST_PACK_VERSION="$PACK_VERSION"
export PROMETHEUS_NATIVE_TEST_PROCESS_VERSION="$PROCESS_VERSION"

if ! HOME="$TEST_HOME" \
  PROMETHEUS_CLAUDE_BIN="$FAKE_BIN/claude" \
  PROMETHEUS_CODEX_BIN="$FAKE_BIN/codex" \
  PROMETHEUS_KIMI_INSTALLER="$FAKE_BIN/kimi-installer" \
  PROMETHEUS_CLAUDE_LIST_DELAY_SECONDS=0 \
  bash "$REPO_ROOT/scripts/refresh-native-plugin-installs.sh" \
    --source-root "$REPO_ROOT" --generation "$GENERATION" --force \
    >"$TMP_ROOT/refresh.out" 2>"$TMP_ROOT/refresh.err"; then
  cat "$TMP_ROOT/refresh.out"
  cat "$TMP_ROOT/refresh.err" >&2
  exit 1
fi

[[ ! -e "$TEST_HOME/Library/LaunchAgents/ai.prometheus.codex-skills-sync.plist" ]]
grep -Fq 'claude plugin marketplace update prometheus-skill-pack' "$LOG"
grep -Fq 'claude plugin uninstall --scope user prometheus-skill-pack@prometheus-skill-pack' "$LOG"
grep -Fq 'claude plugin install --scope user prometheus-skill-pack@prometheus-skill-pack' "$LOG"
grep -Fq 'claude plugin uninstall --scope user prometheus-process-skills@prometheus-skill-pack' "$LOG"
grep -Fq 'claude plugin install --scope user prometheus-process-skills@prometheus-skill-pack' "$LOG"
grep -Fq 'codex plugin marketplace list --json' "$LOG"
grep -Fq 'codex plugin add prometheus-skill-pack@prometheus-skill-pack --json' "$LOG"
grep -Fq 'codex plugin add prometheus-process-skills@prometheus-skill-pack --json' "$LOG"
grep -Fq 'codex plugin list --json' "$LOG"
grep -Fq 'kimi install' "$LOG"
grep -Fq 'Claude installed plugins refreshed through the plugin CLI' "$TMP_ROOT/refresh.out"
grep -Fq 'Codex local marketplace and affected plugins refreshed and verified' "$TMP_ROOT/refresh.out"
[[ -f "$TMP_ROOT/stale-seen" ]]

if rg -n '\.claude/plugins/cache/' "$REPO_ROOT/scripts/refresh-native-plugin-installs.sh"; then
  echo "FAIL: native refresher contains a direct Claude cache path" >&2
  exit 1
fi

# A hook carrier whose cached hook file is stale is reinstalled WITHOUT --force,
# because `plugin update` does nothing for an unchanged version. A disabled
# plugin is left alone: reinstalling it would switch it back on.
STALE_RUNTIME="$TEST_HOME/native/runtime-stale"
FRESH_RUNTIME="$TEST_HOME/native/runtime-fresh"
mkdir -p "$STALE_RUNTIME/hooks" "$FRESH_RUNTIME/hooks"
printf '{"hooks":{}}\n' > "$STALE_RUNTIME/hooks/hooks.json"
cp "$REPO_ROOT/dist/plugins/claude/prometheus-runtime/hooks/hooks.json" "$FRESH_RUNTIME/hooks/hooks.json"
cat > "$TMP_ROOT/claude-before-runtime.json" <<JSON
[
  {"id":"prometheus-skill-pack@prometheus-skill-pack","scope":"user","enabled":false,"version":"$PACK_VERSION","installPath":"$UMBRELLA_INSTALL"},
  {"id":"prometheus-runtime@prometheus-skill-pack","scope":"user","enabled":true,"version":"$PACK_VERSION","installPath":"$STALE_RUNTIME"}
]
JSON
cat > "$TMP_ROOT/claude-after-runtime.json" <<JSON
[
  {"id":"prometheus-skill-pack@prometheus-skill-pack","scope":"user","enabled":false,"version":"$PACK_VERSION","installPath":"$UMBRELLA_INSTALL"},
  {"id":"prometheus-runtime@prometheus-skill-pack","scope":"user","enabled":true,"version":"$PACK_VERSION","installPath":"$FRESH_RUNTIME"}
]
JSON
: > "$LOG"
# The registry-lag simulation belongs to the first run; here the listing is current.
rm -f "$TMP_ROOT/updated"
touch "$TMP_ROOT/stale-seen"
# The fake Kimi installer copies into a fixed place; start each run from none.
rm -rf "$TEST_HOME/Library/Application Support/kimi-desktop/daimon-share/daimon/plugin-packages/prometheus-skill-pack"
if ! HOME="$TEST_HOME" \
  PROMETHEUS_NATIVE_TEST_BEFORE="$TMP_ROOT/claude-before-runtime.json" \
  PROMETHEUS_NATIVE_TEST_AFTER="$TMP_ROOT/claude-after-runtime.json" \
  PROMETHEUS_CLAUDE_BIN="$FAKE_BIN/claude" \
  PROMETHEUS_CODEX_BIN="$FAKE_BIN/codex" \
  PROMETHEUS_KIMI_INSTALLER="$FAKE_BIN/kimi-installer" \
  PROMETHEUS_CLAUDE_LIST_DELAY_SECONDS=0 \
  bash "$REPO_ROOT/scripts/refresh-native-plugin-installs.sh" \
    --source-root "$REPO_ROOT" --generation "$GENERATION" \
    >"$TMP_ROOT/runtime.out" 2>"$TMP_ROOT/runtime.err"; then
  cat "$TMP_ROOT/runtime.out"
  cat "$TMP_ROOT/runtime.err" >&2
  exit 1
fi
grep -Fq 'prometheus-runtime@prometheus-skill-pack holds a stale hook file' "$TMP_ROOT/runtime.out"
grep -Fq 'claude plugin uninstall --scope user prometheus-runtime@prometheus-skill-pack' "$LOG"
grep -Fq 'claude plugin install --scope user prometheus-runtime@prometheus-skill-pack' "$LOG"
if grep -Fq 'prometheus-skill-pack@prometheus-skill-pack' <(grep -F 'claude plugin ' "$LOG" | grep -v 'marketplace'); then
  echo "FAIL: a disabled plugin was refreshed" >&2
  exit 1
fi

# A carrier that still differs after the refresh fails the run instead of
# leaving hooks that will stop at the next prune.
: > "$LOG"
# The registry-lag simulation belongs to the first run; here the listing is current.
rm -f "$TMP_ROOT/updated"
touch "$TMP_ROOT/stale-seen"
# The fake Kimi installer copies into a fixed place; start each run from none.
rm -rf "$TEST_HOME/Library/Application Support/kimi-desktop/daimon-share/daimon/plugin-packages/prometheus-skill-pack"
if HOME="$TEST_HOME" \
  PROMETHEUS_NATIVE_TEST_BEFORE="$TMP_ROOT/claude-before-runtime.json" \
  PROMETHEUS_NATIVE_TEST_AFTER="$TMP_ROOT/claude-before-runtime.json" \
  PROMETHEUS_CLAUDE_BIN="$FAKE_BIN/claude" \
  PROMETHEUS_CODEX_BIN="$FAKE_BIN/codex" \
  PROMETHEUS_KIMI_INSTALLER="$FAKE_BIN/kimi-installer" \
  PROMETHEUS_CLAUDE_LIST_DELAY_SECONDS=0 \
  bash "$REPO_ROOT/scripts/refresh-native-plugin-installs.sh" \
    --source-root "$REPO_ROOT" --generation "$GENERATION" \
    >"$TMP_ROOT/still-stale.out" 2>"$TMP_ROOT/still-stale.err"; then
  echo "FAIL: a carrier that is still stale after the refresh must fail the run" >&2
  exit 1
fi
grep -Fq 'still holds a hook file that differs from this checkout' "$TMP_ROOT/still-stale.err"

MISSING_HOME="$TMP_ROOT/missing-home"
MISSING_PLUGIN="$MISSING_HOME/.prometheus/plugins/prometheus-skill-pack"
mkdir -p "$MISSING_PLUGIN/generations/$GENERATION/skills/process/kbd-process-orchestrator/skills"
cp -R "$SOURCE_KBD" "$MISSING_PLUGIN/generations/$GENERATION/skills/process/kbd-process-orchestrator/skills/kbd-init"
cp -R "$SOURCE_KBD" "$MISSING_PLUGIN/generations/$GENERATION/skills/kbd-init"
printf '{"sourceVersion":"%s"}\n' "$PACK_VERSION" > "$MISSING_PLUGIN/generations/$GENERATION/manifest.json"
ln -s "generations/$GENERATION" "$MISSING_PLUGIN/current"
HOME="$MISSING_HOME" \
PROMETHEUS_CLAUDE_BIN="$TMP_ROOT/missing-claude" \
PROMETHEUS_CODEX_BIN="$TMP_ROOT/missing-codex" \
bash "$REPO_ROOT/scripts/refresh-native-plugin-installs.sh" \
  --source-root "$REPO_ROOT" --generation "$GENERATION" \
  >"$TMP_ROOT/missing.out" 2>"$TMP_ROOT/missing.err"
grep -Fq 'Claude CLI not installed; cache refresh skipped' "$TMP_ROOT/missing.out"
grep -Fq 'Codex CLI not installed; marketplace verification skipped' "$TMP_ROOT/missing.out"
grep -Fq 'Kimi Desktop not installed; native package refresh skipped' "$TMP_ROOT/missing.out"

echo "PASS: native plugin refresh uses supported CLIs, verifies payloads, and skips absent tools"
