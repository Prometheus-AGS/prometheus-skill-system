#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/prometheus-entrypoints.XXXXXX")"
trap 'rm -rf "$TMP_ROOT"' EXIT

CODEX_HOME="$TMP_ROOT/codex"
mkdir -p "$CODEX_HOME/skills/kbd-process-orchestrator"
printf 'fixture-generation\n' > "$CODEX_HOME/skills/kbd-process-orchestrator/.prometheus-generation"
CODEX_HOME="$CODEX_HOME" bash "$REPO_ROOT/scripts/codex-sync-skills.sh" --dry-run \
  >"$TMP_ROOT/codex.out" 2>"$TMP_ROOT/codex.err"
grep -Fq 'kbd-process-orchestrator is managed by the active immutable generation' "$TMP_ROOT/codex.out"
if grep -Fq 'kbd-process-orchestrator exists and is not pack-owned' "$TMP_ROOT/codex.out"; then
  echo "FAIL: generation-owned Codex copy was classified as a user collision" >&2
  exit 1
fi

node - "$REPO_ROOT/scripts/install-platforms.ts" <<'NODE'
const fs = require('fs');
const source = fs.readFileSync(process.argv[2], 'utf8');
if (!source.includes("scope === 'global'")) throw new Error('global scope branch is missing');
if (!source.includes('install-plugin-generation.js')) throw new Error('global installer does not delegate to immutable generations');
if (!source.includes("scope === 'global' ? 'npm run validate:codex' : 'npm run build:codex'")) {
  throw new Error('global Codex install can rewrite generated artifacts after activation');
}
NODE

if rg -n 'launchctl bootstrap.*codex-skills-sync|skills-sync agent loaded' \
  "$REPO_ROOT/scripts/install-skills-flat.sh"; then
  echo "FAIL: flat installer still registers the legacy Codex watcher" >&2
  exit 1
fi
grep -Fq 'legacy skills-sync agent removed' "$REPO_ROOT/scripts/install-skills-flat.sh"

echo "PASS: global entrypoints use immutable generations and legacy Codex ownership is preserved"
