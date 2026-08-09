#!/usr/bin/env bash

set -euo pipefail

SOURCE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PLUGIN_ROOT="${HOME}/.prometheus/plugins/prometheus-skill-pack"
MIGRATION_ROOT="${HOME}/.prometheus/migrations"
DRY_RUN=false
SKIP_NATIVE_REFRESH=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --source-root) SOURCE_ROOT="$(cd "${2:?missing value for --source-root}" && pwd)"; shift 2 ;;
        --plugin-root) PLUGIN_ROOT="${2:?missing value for --plugin-root}"; shift 2 ;;
        --dry-run) DRY_RUN=true; shift ;;
        --skip-native-refresh) SKIP_NATIVE_REFRESH=true; shift ;;
        *) echo "migrate-skill-system-1.7.0: unknown argument: $1" >&2; exit 2 ;;
    esac
done

read -r RELEASE MINIMUM < <(node - "$SOURCE_ROOT/skill-system.json" <<'NODE'
const fs = require('fs');
const contract = JSON.parse(fs.readFileSync(process.argv[2], 'utf8'));
process.stdout.write(`${contract.releaseVersion} ${contract.minimumActiveVersion}\n`);
NODE
)
if [[ "$RELEASE" != "1.7.0" || "$MINIMUM" != "1.7.0" ]]; then
    echo "migrate-skill-system-1.7.0: contract must declare release and minimum 1.7.0" >&2
    exit 1
fi

SOURCE_COMMIT="$(git -C "$SOURCE_ROOT" rev-parse HEAD)"
BEFORE_CURRENT="$(readlink "$PLUGIN_ROOT/current" 2>/dev/null || true)"
BEFORE_PREVIOUS="$(readlink "$PLUGIN_ROOT/previous" 2>/dev/null || true)"

if $DRY_RUN; then
    printf '%s\n' \
        "Migration dry run" \
        "  source commit: $SOURCE_COMMIT" \
        "  current: ${BEFORE_CURRENT:-missing}" \
        "  previous: ${BEFORE_PREVIOUS:-missing}" \
        "  release/minimum: $RELEASE/$MINIMUM" \
        "  actions: clean checkout, signed activation, target verification, native refresh, Claude prune, receipt-aware generation prune"
    exit 0
fi

WORKTREE_PARENT="$(mktemp -d "${TMPDIR:-/tmp}/prometheus-migration.XXXXXX")"
CHECKOUT="$WORKTREE_PARENT/source"
WORKTREE_ADDED=false
cleanup() {
    if $WORKTREE_ADDED; then git -C "$SOURCE_ROOT" worktree remove --force "$CHECKOUT" >/dev/null 2>&1 || true; fi
    rmdir "$WORKTREE_PARENT" >/dev/null 2>&1 || true
}
trap cleanup EXIT

git -C "$SOURCE_ROOT" worktree add --detach "$CHECKOUT" "$SOURCE_COMMIT" >/dev/null
WORKTREE_ADDED=true
git -C "$CHECKOUT" submodule update --init --recursive

NEW_GENERATION="$(node "$CHECKOUT/scripts/install-plugin-generation.js" \
    --source-root "$CHECKOUT" \
    --plugin-root "$PLUGIN_ROOT" \
    --expected-source-commit "$SOURCE_COMMIT" \
    --require-clean-source)"

rollback_after_refresh_failure() {
    node "$CHECKOUT/scripts/install-plugin-generation.js" \
        --source-root "$CHECKOUT" \
        --plugin-root "$PLUGIN_ROOT" \
        --rollback >/dev/null || true
}

REFRESHED_CLIENTS=()
if ! $SKIP_NATIVE_REFRESH; then
    if ! bash "$CHECKOUT/scripts/refresh-native-plugin-installs.sh" \
        --source-root "$CHECKOUT" --generation "$NEW_GENERATION" --force; then
        rollback_after_refresh_failure
        echo "migrate-skill-system-1.7.0: native refresh failed; restored prior verified generation" >&2
        exit 1
    fi
    command -v claude >/dev/null 2>&1 && REFRESHED_CLIENTS+=(claude)
    command -v codex >/dev/null 2>&1 && REFRESHED_CLIENTS+=(codex)
    [[ -d "$HOME/Library/Application Support/kimi-desktop/daimon-share/daimon/plugin-packages" ]] && REFRESHED_CLIENTS+=(kimi-desktop)
fi

node "$CHECKOUT/scripts/install-plugin-generation.js" \
    --source-root "$CHECKOUT" \
    --plugin-root "$PLUGIN_ROOT" \
    --verify >/dev/null

AFTER_CURRENT="$(readlink "$PLUGIN_ROOT/current")"
AFTER_PREVIOUS="$(readlink "$PLUGIN_ROOT/previous")"
node - "$PLUGIN_ROOT" "$AFTER_CURRENT" "$AFTER_PREVIOUS" <<'NODE'
const fs = require('fs');
const path = require('path');
for (const [label, relative] of [['current', process.argv[3]], ['previous', process.argv[4]]]) {
  const manifest = JSON.parse(fs.readFileSync(path.join(process.argv[2], relative, 'manifest.json'), 'utf8'));
  if (manifest.sourceVersion !== '1.7.0') throw new Error(`${label} is ${manifest.sourceVersion}, expected 1.7.0`);
}
NODE

CLAUDE_PRUNE="skipped"
if command -v claude >/dev/null 2>&1; then
    claude plugin prune --dry-run
    claude plugin prune --yes
    CLAUDE_PRUNE="completed"
fi

PRUNE_JSON="$(node "$CHECKOUT/scripts/install-plugin-generation.js" \
    --source-root "$CHECKOUT" \
    --plugin-root "$PLUGIN_ROOT" \
    --prune-obsolete)"

mkdir -p "$MIGRATION_ROOT"
chmod 700 "$MIGRATION_ROOT"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
RECEIPT="$MIGRATION_ROOT/skill-system-1.7.0-$STAMP.json"
REFRESHED_CSV="$(IFS=,; echo "${REFRESHED_CLIENTS[*]:-}")"
node - "$RECEIPT" "$SOURCE_COMMIT" "$BEFORE_CURRENT" "$BEFORE_PREVIOUS" "$AFTER_CURRENT" "$AFTER_PREVIOUS" "$NEW_GENERATION" "$REFRESHED_CSV" "$CLAUDE_PRUNE" "$PRUNE_JSON" <<'NODE'
const fs = require('fs');
const [file, sourceCommit, beforeCurrent, beforePrevious, afterCurrent, afterPrevious, generation, refreshed, claudePrune, pruneJson] = process.argv.slice(2);
const receipt = {
  schemaVersion: 'prometheus-skill-system-migration-v1',
  migration: 'minimum-active-1.7.0',
  completedAt: new Date().toISOString(),
  sourceCommit,
  before: { current: beforeCurrent || null, previous: beforePrevious || null },
  after: { current: afterCurrent, previous: afterPrevious },
  releaseVersion: '1.7.0',
  minimumActiveVersion: '1.7.0',
  activatedGeneration: generation,
  refreshedClients: refreshed ? refreshed.split(',') : [],
  retiredGenerations: JSON.parse(pruneJson).retired,
  verification: {
    signedGeneration: 'passed',
    sourceCommit: 'passed',
    skillIndex: 'passed',
    selectedTargets: 'passed',
    currentAndPreviousMinimum: 'passed',
    claudePrune
  },
  sessionReloadRequired: ['claude', 'codex']
};
fs.writeFileSync(file, `${JSON.stringify(receipt, null, 2)}\n`, { mode: 0o600 });
NODE
chmod 600 "$RECEIPT"

echo "$RECEIPT"
