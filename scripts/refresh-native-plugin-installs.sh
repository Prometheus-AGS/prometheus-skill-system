#!/usr/bin/env bash
# Refresh native plugin surfaces through their supported installers. This script
# never writes Claude or Codex cache directories directly.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GENERATION=""
FORCE=false
CLAUDE_BIN="${PROMETHEUS_CLAUDE_BIN:-claude}"
CODEX_BIN="${PROMETHEUS_CODEX_BIN:-codex}"
KIMI_INSTALLER="${PROMETHEUS_KIMI_INSTALLER:-}"
PLUGIN_ROOT="${HOME}/.prometheus/plugins/prometheus-skill-pack"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --source-root) REPO_ROOT="${2:-}"; shift 2 ;;
        --generation) GENERATION="${2:-}"; shift 2 ;;
        --force) FORCE=true; shift ;;
        *) echo "Unknown argument: $1" >&2; exit 2 ;;
    esac
done

if [[ -z "$GENERATION" ]]; then
    echo "refresh-native-plugin-installs: --generation is required" >&2
    exit 2
fi
KIMI_INSTALLER="${KIMI_INSTALLER:-$REPO_ROOT/scripts/install-kimi-desktop-plugin.sh}"

read -r RELEASE_VERSION MINIMUM_ACTIVE_VERSION < <(node - "$REPO_ROOT/skill-system.json" <<'NODE'
const fs = require('fs');
const contract = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
process.stdout.write(`${contract.releaseVersion} ${contract.minimumActiveVersion}\n`);
NODE
)

command_available() {
    [[ "$1" == */* ]] && [[ -x "$1" ]] && return 0
    command -v "$1" >/dev/null 2>&1
}

compare_kbd_payload() {
    local installed="$1"
    node - "$REPO_ROOT/skills/process/kbd-process-orchestrator/skills/kbd-init" "$installed" <<'NODE'
const crypto = require('crypto');
const fs = require('fs');
const path = require('path');
const [source, installed] = process.argv.slice(2);

function collect(root, relative = '') {
  const entries = [];
  for (const name of fs.readdirSync(path.join(root, relative)).sort()) {
    const child = path.join(relative, name);
    const absolute = path.join(root, child);
    const stat = fs.lstatSync(absolute);
    if (stat.isDirectory()) entries.push(...collect(root, child));
    else if (stat.isFile()) {
      entries.push({
        path: child.split(path.sep).join('/'),
        sha256: crypto.createHash('sha256').update(fs.readFileSync(absolute)).digest('hex'),
        executable: Boolean(stat.mode & 0o111),
      });
    }
  }
  return entries;
}

if (!fs.existsSync(installed)) throw new Error(`installed kbd-init payload is missing: ${installed}`);
const expected = JSON.stringify(collect(source));
const actual = JSON.stringify(collect(installed));
if (expected !== actual) throw new Error(`installed kbd-init payload differs: ${installed}`);
NODE
}

active="$(readlink "$PLUGIN_ROOT/current" 2>/dev/null || true)"
if [[ "$active" != "generations/$GENERATION" ]]; then
    echo "refresh-native-plugin-installs: active generation mismatch: ${active:-missing}" >&2
    exit 1
fi
node - "$PLUGIN_ROOT/current/manifest.json" "$MINIMUM_ACTIVE_VERSION" "$RELEASE_VERSION" <<'NODE'
const fs = require('fs');
const [manifestFile, minimum, release] = process.argv.slice(2);
const version = JSON.parse(fs.readFileSync(manifestFile, 'utf8')).sourceVersion;
const parts = value => value.split('.').map(Number);
const compare = (left, right) => {
  const a = parts(left);
  const b = parts(right);
  for (let index = 0; index < 3; index += 1) if (a[index] !== b[index]) return a[index] - b[index];
  return 0;
};
if (compare(version, minimum) < 0) throw new Error(`active generation ${version} is below minimum ${minimum}`);
if (version !== release) throw new Error(`active generation ${version} is not source release ${release}`);
NODE
compare_kbd_payload "$PLUGIN_ROOT/current/skills/kbd-init"
echo "  ✅ immutable generation payload verified"

if [[ "$(uname -s)" == "Darwin" ]]; then
    legacy_label="ai.prometheus.codex-skills-sync"
    legacy_plist="$HOME/Library/LaunchAgents/$legacy_label.plist"
    launchctl bootout "gui/$(id -u)/$legacy_label" 2>/dev/null || true
    if [[ -e "$legacy_plist" ]]; then
        rm -f "$legacy_plist"
        echo "  ✅ removed legacy Codex skill-sync LaunchAgent"
    else
        echo "  — legacy Codex skill-sync LaunchAgent already absent"
    fi
fi

# Kimi Desktop uses an app-managed copied package rather than a flat skill root.
kimi_packages="$HOME/Library/Application Support/kimi-desktop/daimon-share/daimon/plugin-packages"
if [[ -d "$kimi_packages" ]]; then
    bash "$KIMI_INSTALLER"
    compare_kbd_payload "$kimi_packages/prometheus-skill-pack/skills/kbd-init"
    echo "  ✅ Kimi Desktop plugin payload refreshed"
else
    echo "  — Kimi Desktop not installed; native package refresh skipped"
fi

# Claude owns its versioned plugin cache. Refresh only through the CLI and then
# validate the CLI-reported install path; old immutable versions remain intact.
if command_available "$CLAUDE_BIN"; then
    claude_before="$($CLAUDE_BIN plugin list --json)"
    installed_affected="$(node -e '
const rows=JSON.parse(process.argv[1]);
const wanted=new Set(["prometheus-skill-pack@prometheus-skill-pack","prometheus-process-skills@prometheus-skill-pack"]);
for (const row of rows) if (wanted.has(row.id) && row.scope === "user") console.log(row.id);
' "$claude_before")"
    if [[ -n "$installed_affected" ]]; then
        "$CLAUDE_BIN" plugin marketplace update prometheus-skill-pack
        while IFS= read -r plugin_id; do
            [[ -n "$plugin_id" ]] || continue
            if $FORCE; then
                # Same-version package contents are immutable in Claude's cache.
                # A supported uninstall/install cycle is required for every
                # affected plugin whose source bytes may have changed.
                "$CLAUDE_BIN" plugin uninstall --scope user "$plugin_id"
                "$CLAUDE_BIN" plugin install --scope user "$plugin_id"
            else
                "$CLAUDE_BIN" plugin update --scope user "$plugin_id"
            fi
        done <<< "$installed_affected"

        claude_after=""
        claude_versions_ready=false
        claude_list_attempts="${PROMETHEUS_CLAUDE_LIST_ATTEMPTS:-10}"
        claude_list_delay="${PROMETHEUS_CLAUDE_LIST_DELAY_SECONDS:-1}"
        for ((attempt = 1; attempt <= claude_list_attempts; attempt++)); do
            claude_after="$($CLAUDE_BIN plugin list --json)"
            if node - "$claude_after" "$REPO_ROOT" <<'NODE'
const fs = require('fs');
const path = require('path');
const [payload, root] = process.argv.slice(2);
const rows = JSON.parse(payload);
const expected = new Map([
  ['prometheus-skill-pack@prometheus-skill-pack', JSON.parse(fs.readFileSync(path.join(root, 'skill-system.json'))).releaseVersion],
  ['prometheus-process-skills@prometheus-skill-pack', JSON.parse(fs.readFileSync(path.join(root, 'skills/process/.claude-plugin/plugin.json'))).version],
]);
for (const [id, version] of expected) {
  const row = rows.find(candidate => candidate.id === id && candidate.scope === 'user');
  if (row && row.version !== version) process.exit(1);
}
NODE
            then
                claude_versions_ready=true
                break
            fi
            if ((attempt < claude_list_attempts)); then
                sleep "$claude_list_delay"
            fi
        done
        if ! $claude_versions_ready; then
            echo "refresh-native-plugin-installs: Claude plugin registry did not converge after $claude_list_attempts checks" >&2
        fi
        mapfile_path="$(mktemp "${TMPDIR:-/tmp}/prometheus-claude-installs.XXXXXX")"
        node - "$claude_after" "$REPO_ROOT" > "$mapfile_path" <<'NODE'
const fs = require('fs');
const path = require('path');
const [payload, root] = process.argv.slice(2);
const rows = JSON.parse(payload);
const expected = new Map([
  ['prometheus-skill-pack@prometheus-skill-pack', JSON.parse(fs.readFileSync(path.join(root, 'skill-system.json'))).releaseVersion],
  ['prometheus-process-skills@prometheus-skill-pack', JSON.parse(fs.readFileSync(path.join(root, 'skills/process/.claude-plugin/plugin.json'))).version],
]);
for (const [id, version] of expected) {
  const row = rows.find(candidate => candidate.id === id && candidate.scope === 'user');
  if (!row) continue;
  if (id.startsWith('prometheus-skill-pack@') && row.enabled === false) throw new Error(`${id}: Claude plugin is disabled`);
  if (row.version !== version) throw new Error(`${id}: expected ${version}, found ${row.version}`);
  console.log(`${id}\t${row.installPath}`);
}
NODE
        while IFS=$'\t' read -r plugin_id install_path; do
            case "$plugin_id" in
                prometheus-skill-pack@*)
                    compare_kbd_payload "$install_path/skills/kbd-init"
                    ;;
                prometheus-process-skills@*)
                    compare_kbd_payload "$install_path/kbd-process-orchestrator/skills/kbd-init"
                    ;;
            esac
        done < "$mapfile_path"
        rm -f "$mapfile_path"
        echo "  ✅ Claude installed plugins refreshed through the plugin CLI"
        echo "    Restart Claude Code to load the refreshed versions."
    else
        echo "  — Claude Prometheus plugins are not installed; cache refresh skipped"
    fi
else
    echo "  — Claude CLI not installed; cache refresh skipped"
fi

# Codex local marketplaces resolve the repository directly rather than copying
# a versioned cache. Verify the registered source and affected plugin versions.
if command_available "$CODEX_BIN"; then
    codex_marketplaces="$($CODEX_BIN plugin marketplace list --json)"
    if node -e '
const data=JSON.parse(process.argv[1]);
process.exit(data.marketplaces?.some(row => row.name === "prometheus-skill-pack") ? 0 : 1);
' "$codex_marketplaces"; then
        codex_before="$($CODEX_BIN plugin list --json)"
        installed_codex_affected="$(node -e '
const rows=JSON.parse(process.argv[1]).installed ?? [];
const wanted=new Set(["prometheus-skill-pack@prometheus-skill-pack","prometheus-process-skills@prometheus-skill-pack"]);
for (const row of rows) if (wanted.has(row.pluginId) && row.installed) console.log(row.pluginId);
' "$codex_before")"
        while IFS= read -r plugin_id; do
            [[ -n "$plugin_id" ]] || continue
            "$CODEX_BIN" plugin add "$plugin_id" --json >/dev/null
        done <<< "$installed_codex_affected"

        codex_plugins="$($CODEX_BIN plugin list --json)"
        node - "$codex_plugins" "$REPO_ROOT" <<'NODE'
const fs = require('fs');
const path = require('path');
const [payload, root] = process.argv.slice(2);
const rows = JSON.parse(payload).installed ?? [];
const expected = new Map([
  ['prometheus-skill-pack@prometheus-skill-pack', JSON.parse(fs.readFileSync(path.join(root, 'skill-system.json'))).releaseVersion],
  ['prometheus-process-skills@prometheus-skill-pack', JSON.parse(fs.readFileSync(path.join(root, 'skills/process/.claude-plugin/plugin.json'))).version],
]);
for (const [id, version] of expected) {
  const row = rows.find(candidate => candidate.pluginId === id);
  if (!row?.installed || !row.enabled) throw new Error(`${id}: Codex plugin is not installed and enabled`);
  if (row.version !== version) throw new Error(`${id}: expected ${version}, found ${row.version}`);
  const source = path.resolve(row.source?.path ?? '');
  const expectedRoot = id.startsWith('prometheus-process-skills@')
    ? path.join(root, 'skills/process')
    : path.join(root, 'dist/plugins/codex/prometheus-skill-pack');
  if (source !== expectedRoot) throw new Error(`${id}: unexpected local source ${source}`);
}
NODE
        echo "  ✅ Codex local marketplace and affected plugins refreshed and verified"
        echo "    Start a new Codex session to load the refreshed source."
    else
        echo "  — Codex Prometheus marketplace is not installed; verification skipped"
    fi
else
    echo "  — Codex CLI not installed; marketplace verification skipped"
fi

if $FORCE; then
    echo "  ℹ️  --force requested; all detected native surfaces were refreshed and reverified"
fi
