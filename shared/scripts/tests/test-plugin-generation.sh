#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
INSTALLER="$ROOT/scripts/install-plugin-generation.js"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
SOURCE="$TMP/source"
PLUGIN_ROOT="$TMP/home/.prometheus/plugins/prometheus-skill-pack"

mkdir -p \
  "$SOURCE/skills/example/scripts" \
  "$SOURCE/agents" \
  "$SOURCE/hooks" \
  "$SOURCE/shared/scripts/tests/fixtures" \
  "$SOURCE/.claude-plugin" \
  "$SOURCE/.codex-plugin" \
  "$SOURCE/.agents/plugins"
printf '%s\n' '---' 'name: example' 'description: fixture' '---' > "$SOURCE/skills/example/SKILL.md"
printf '#!/usr/bin/env bash\nprintf "example\\n"\n' > "$SOURCE/skills/example/scripts/example.sh"
printf 'fixture\n' > "$SOURCE/shared/scripts/tests/fixtures/example.txt"
for script in karpathy-hook-dispatch detect-project-context memory-outbox-flush pk-health; do
  printf '#!/usr/bin/env bash\nprintf "%s-a\\n"\n' "$script" > "$SOURCE/shared/scripts/$script.sh"
  chmod +x "$SOURCE/shared/scripts/$script.sh"
done
for script in enqueue-learning-job enqueue-memory-operation; do
  printf '#!/usr/bin/env python3\nprint("%s-a")\n' "$script" > "$SOURCE/shared/scripts/$script.py"
  chmod +x "$SOURCE/shared/scripts/$script.py"
done
chmod +x "$SOURCE/skills/example/scripts/example.sh"
printf '{}\n' > "$SOURCE/hooks/hooks.json"
printf '{"name":"prometheus-skill-pack","version":"test"}\n' > "$SOURCE/.claude-plugin/plugin.json"
printf '{"name":"prometheus-skill-pack","version":"test"}\n' > "$SOURCE/.codex-plugin/plugin.json"
printf '{"plugins":[]}\n' > "$SOURCE/.agents/plugins/marketplace.json"
printf '{}\n' > "$SOURCE/.mcp.json"

mkdir -p "$TMP/home/.codex/skills/example" "$TMP/home/.minimax/skills/example"
mkdir -p "$TMP/home/.codex/skills/user-owned"
mkdir -p "$TMP/legacy-checkout/skills"
ln -s "$SOURCE/skills/example" "$TMP/legacy-checkout/skills/example"
mkdir -p "$TMP/home/.claude/skills"
ln -s "$TMP/legacy-checkout/skills/example" "$TMP/home/.claude/skills/example"
printf 'keep\n' > "$TMP/home/.codex/skills/user-owned/marker.txt"
printf 'stale\n' > "$TMP/home/.codex/skills/example/SKILL.md"
printf 'source=legacy\n' > "$TMP/home/.codex/skills/example/.prometheus-pack"
printf 'stale\n' > "$TMP/home/.minimax/skills/example/SKILL.md"
printf '{"platform":"minimax"}\n' > "$TMP/home/.minimax/skills/example/_meta.json"

node "$INSTALLER" --source-root "$SOURCE" --plugin-root "$PLUGIN_ROOT" --home "$TMP/home" >/dev/null
FIRST="$(readlink "$PLUGIN_ROOT/current")"
FIRST_HASH="${FIRST##*/}"
[[ -f "$PLUGIN_ROOT/generations/$FIRST_HASH/manifest.json" ]]
[[ "$(jq '.targetPayloads | length' "$PLUGIN_ROOT/generations/$FIRST_HASH/manifest.json")" == 14 ]]
[[ "$(jq -r '.files[] | select(.path == "shared/scripts/karpathy-hook-dispatch.sh") | .mode' "$PLUGIN_ROOT/generations/$FIRST_HASH/manifest.json")" == "0755" ]]
for script in karpathy-hook-dispatch detect-project-context memory-outbox-flush pk-health; do
  [[ "$(readlink "$PLUGIN_ROOT/stable/$script.sh")" == "../current/shared/scripts/$script.sh" ]]
  cmp "$SOURCE/shared/scripts/$script.sh" "$PLUGIN_ROOT/stable/$script.sh"
done
[[ "$(HOME="$TMP/home" bash "$PLUGIN_ROOT/stable/karpathy-hook-dispatch.sh")" == "karpathy-hook-dispatch-a" ]]
[[ ! -e "$TMP/home/.codex/skills/prometheus-example" ]]
[[ ! -e "$TMP/home/.minimax/skills/prometheus-example" ]]
[[ ! -e "$TMP/home/.claude/skills/prometheus-example" ]]
node "$INSTALLER" --plugin-root "$PLUGIN_ROOT" --verify >/dev/null
echo '[PASS] first generation is hash-verified and exposes 14 validated target payloads'

printf '#!/usr/bin/env bash\nprintf "karpathy-hook-dispatch-b\\n"\n' > "$SOURCE/shared/scripts/karpathy-hook-dispatch.sh"
printf '%s\n' '---' 'name: example' 'description: fixture-b' '---' > "$SOURCE/skills/example/SKILL.md"
chmod +x "$SOURCE/shared/scripts/karpathy-hook-dispatch.sh"
node "$INSTALLER" --source-root "$SOURCE" --plugin-root "$PLUGIN_ROOT" --home "$TMP/home" >/dev/null
SECOND="$(readlink "$PLUGIN_ROOT/current")"
[[ "$SECOND" != "$FIRST" ]]
[[ "$(readlink "$PLUGIN_ROOT/previous")" == "$FIRST" ]]
[[ "$(HOME="$TMP/home" bash "$PLUGIN_ROOT/stable/karpathy-hook-dispatch.sh")" == "karpathy-hook-dispatch-b" ]]
grep -q 'fixture-b' "$TMP/home/.codex/skills/example/SKILL.md"
grep -q 'fixture-b' "$TMP/home/.minimax/skills/example/SKILL.md"
for script in detect-project-context memory-outbox-flush pk-health; do
  cmp "$SOURCE/shared/scripts/$script.sh" "$PLUGIN_ROOT/stable/$script.sh"
done
echo '[PASS] activation atomically advances current and retains a previous pointer'

node "$INSTALLER" --plugin-root "$PLUGIN_ROOT" --home "$TMP/home" --rollback >/dev/null
[[ "$(readlink "$PLUGIN_ROOT/current")" == "$FIRST" ]]
[[ "$(HOME="$TMP/home" bash "$PLUGIN_ROOT/stable/karpathy-hook-dispatch.sh")" == "karpathy-hook-dispatch-a" ]]
grep -q 'description: fixture$' "$TMP/home/.codex/skills/example/SKILL.md"
grep -q 'description: fixture$' "$TMP/home/.minimax/skills/example/SKILL.md"
echo '[PASS] pointer rollback restores the previous verified generation'

chmod -x "$SOURCE/shared/scripts/pk-health.sh"
if node "$INSTALLER" --source-root "$SOURCE" --plugin-root "$PLUGIN_ROOT" --home "$TMP/home" >/dev/null 2>&1; then
  echo '[FAIL] invalid generation unexpectedly activated' >&2
  exit 1
fi
[[ "$(readlink "$PLUGIN_ROOT/current")" == "$FIRST" ]]
echo '[PASS] invalid script modes cannot replace the active generation'

for target in \
  .claude/skills .opencode/skills .kimi-code/skills .minimax/skills .cursor/skills \
  .codex/skills .gemini/skills .roo/skills .windsurf/skills .codeium/windsurf/skills \
  .agents/skills .config/zed/skills .zed/skills .cline/skills; do
  [[ -f "$TMP/home/$target/example/SKILL.md" ]]
done
[[ "$(jq -r '.platform' "$TMP/home/.minimax/skills/example/_meta.json")" == minimax ]]
echo '[PASS] all 14 installed target payloads resolve through the certified generation'

node "$INSTALLER" --plugin-root "$PLUGIN_ROOT" --home "$TMP/home" --uninstall >/dev/null
[[ ! -e "$PLUGIN_ROOT" ]]
[[ "$(cat "$TMP/home/.codex/skills/user-owned/marker.txt")" == keep ]]
for target in .claude/skills .codex/skills .minimax/skills .cline/skills; do
  [[ ! -e "$TMP/home/$target/example" ]]
done
echo '[PASS] uninstall removes only generation-managed projections and plugin state'
