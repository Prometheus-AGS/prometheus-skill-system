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
  "$SOURCE/scripts" \
  "$SOURCE/shared/harnesses" \
  "$SOURCE/shared/scripts/lib" \
  "$SOURCE/shared/scripts/tests/fixtures" \
  "$SOURCE/.claude-plugin" \
  "$SOURCE/.codex-plugin" \
  "$SOURCE/.agents/plugins"
printf '%s\n' '---' 'name: example' 'description: fixture' '---' > "$SOURCE/skills/example/SKILL.md"
printf '#!/usr/bin/env bash\nprintf "example\\n"\n' > "$SOURCE/skills/example/scripts/example.sh"
printf 'fixture\n' > "$SOURCE/shared/scripts/tests/fixtures/example.txt"
printf '#!/usr/bin/env bash\n' > "$SOURCE/shared/scripts/lib/hook-log.sh"
printf '#!/usr/bin/env bash\n' > "$SOURCE/shared/scripts/lib/memory-bridge.sh"
cp "$ROOT/scripts/install-plugin-generation.js" "$SOURCE/scripts/install-plugin-generation.js"
cp "$ROOT/scripts/generate-harness-adapters.js" "$SOURCE/scripts/generate-harness-adapters.js"
cp "$ROOT/shared/scripts/hook-runtime-v1.sh" "$SOURCE/shared/scripts/hook-runtime-v1.sh"
cp "$ROOT/shared/scripts/bootstrap-hook-runtime.sh" "$SOURCE/shared/scripts/bootstrap-hook-runtime.sh"
cp "$ROOT/shared/harnesses/capabilities.json" "$SOURCE/shared/harnesses/capabilities.json"
printf '%s\n' '{"schemaVersion":"hook-contract-v1","dispatcherAbi":"hook-runtime-v1","harnesses":["claude-code","codex"],"events":[{"event":"UserPromptSubmit","hooks":[{"id":"fixture-hook","target":"shared/scripts/karpathy-hook-dispatch.sh"}]}]}' > "$SOURCE/shared/harnesses/hook-contract.json"
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
node "$SOURCE/scripts/generate-harness-adapters.js" >/dev/null

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
FIRST_BUNDLE="$(jq -r '.bundleId' "$PLUGIN_ROOT/generations/$FIRST_HASH/manifest.json")"
[[ "$(readlink "$PLUGIN_ROOT/bundles/$FIRST_BUNDLE")" == "../generations/$FIRST_HASH" ]]
[[ -x "$PLUGIN_ROOT/runtime/v1/run-hook" ]]
PROMETHEUS_PLUGIN_ROOT="$PLUGIN_ROOT" "$PLUGIN_ROOT/runtime/v1/run-hook" \
  --bundle "$FIRST_BUNDLE" --resolve-only >/dev/null
[[ "$(jq '.targetPayloads | length' "$PLUGIN_ROOT/generations/$FIRST_HASH/manifest.json")" == 14 ]]
[[ -f "$PLUGIN_ROOT/generations/$FIRST_HASH/manifest.sig.json" ]]
[[ "$(stat -f '%Lp' "$PLUGIN_ROOT/generations/$FIRST_HASH/manifest.sig.json" 2>/dev/null || stat -c '%a' "$PLUGIN_ROOT/generations/$FIRST_HASH/manifest.sig.json")" == 600 ]]
[[ "$(find "$PLUGIN_ROOT/receipts/$FIRST_HASH" -type f -name '*.json' | wc -l | tr -d ' ')" == 14 ]]
[[ -f "$PLUGIN_ROOT/trust/allowed-signers.json" ]]
[[ "$(jq '.signers | length' "$PLUGIN_ROOT/trust/allowed-signers.json")" == 1 ]]
[[ "$(stat -f '%Lp' "$TMP/home/.prometheus/plugin-signing/ed25519-private.pem" 2>/dev/null || stat -c '%a' "$TMP/home/.prometheus/plugin-signing/ed25519-private.pem")" == 600 ]]
cmp "$PLUGIN_ROOT/generations/$FIRST_HASH/indexes/skills.json" \
  "$PLUGIN_ROOT/generations/$FIRST_HASH/mobile/skill-index.json"
cmp "$PLUGIN_ROOT/generations/$FIRST_HASH/indexes/skills.json" \
  "$PLUGIN_ROOT/generations/$FIRST_HASH/agents/skill-index.json"
cmp "$PLUGIN_ROOT/generations/$FIRST_HASH/indexes/skills.json" \
  "$PLUGIN_ROOT/stable/skill-index.json"
[[ "$(jq -r '.skillIndexSha256' "$PLUGIN_ROOT/generations/$FIRST_HASH/mobile/parity.json")" == \
    "$(jq -r '.skillIndex.sha256' "$PLUGIN_ROOT/generations/$FIRST_HASH/manifest.json")" ]]
[[ "$(jq -r '.files[] | select(.path == "shared/scripts/karpathy-hook-dispatch.sh") | .mode' "$PLUGIN_ROOT/generations/$FIRST_HASH/manifest.json")" == "0755" ]]
for script in karpathy-hook-dispatch detect-project-context memory-outbox-flush pk-health; do
  [[ "$(readlink "$PLUGIN_ROOT/stable/$script.sh")" == "../current/shared/scripts/$script.sh" ]]
  cmp "$SOURCE/shared/scripts/$script.sh" "$PLUGIN_ROOT/stable/$script.sh"
done
for script in enqueue-learning-job enqueue-memory-operation; do
  [[ "$(readlink "$PLUGIN_ROOT/stable/$script.py")" == "../current/shared/scripts/$script.py" ]]
  [[ "$(HOME="$TMP/home" "$PLUGIN_ROOT/stable/$script.py")" == "$script-a" ]]
done
[[ "$(readlink "$PLUGIN_ROOT/stable/lib")" == "../current/shared/scripts/lib" ]]
cmp "$SOURCE/shared/scripts/lib/memory-bridge.sh" "$PLUGIN_ROOT/stable/lib/memory-bridge.sh"
[[ "$(HOME="$TMP/home" bash "$PLUGIN_ROOT/stable/karpathy-hook-dispatch.sh")" == "karpathy-hook-dispatch-a" ]]
[[ ! -e "$TMP/home/.codex/skills/prometheus-example" ]]
[[ ! -e "$TMP/home/.minimax/skills/prometheus-example" ]]
[[ ! -e "$TMP/home/.claude/skills/prometheus-example" ]]
node "$INSTALLER" --plugin-root "$PLUGIN_ROOT" --verify >/dev/null
cp "$PLUGIN_ROOT/generations/$FIRST_HASH/manifest.sig.json" "$TMP/manifest.sig.json"
jq '.signature = "AAAA"' "$TMP/manifest.sig.json" > \
  "$PLUGIN_ROOT/generations/$FIRST_HASH/manifest.sig.json"
if node "$INSTALLER" --plugin-root "$PLUGIN_ROOT" --verify >/dev/null 2>&1; then
  echo '[FAIL] tampered generation signature unexpectedly verified' >&2
  exit 1
fi
cp "$TMP/manifest.sig.json" "$PLUGIN_ROOT/generations/$FIRST_HASH/manifest.sig.json"
node "$INSTALLER" --plugin-root "$PLUGIN_ROOT" --verify >/dev/null
echo '[PASS] signed generation, canonical index parity, and 14 signed target receipts verify'
echo '[PASS] manifest-signature tampering fails closed before activation'

node - "$TMP/untrusted-private.pem" <<'JS'
const crypto = require('crypto');
const fs = require('fs');
const pair = crypto.generateKeyPairSync('ed25519');
fs.writeFileSync(process.argv[2], pair.privateKey.export({ type: 'pkcs8', format: 'pem' }), {
  mode: 0o600,
});
JS
if node "$INSTALLER" --source-root "$SOURCE" --plugin-root "$PLUGIN_ROOT" \
  --home "$TMP/home" --signing-key "$TMP/untrusted-private.pem" >/dev/null 2>&1; then
  echo '[FAIL] untrusted signing key was silently enrolled' >&2
  exit 1
fi
[[ "$(readlink "$PLUGIN_ROOT/current")" == "$FIRST" ]]
echo '[PASS] an existing trust store rejects unapproved signing identities'

printf '#!/usr/bin/env bash\nprintf "karpathy-hook-dispatch-b\\n"\n' > "$SOURCE/shared/scripts/karpathy-hook-dispatch.sh"
printf '%s\n' '---' 'name: example' 'description: fixture-b' '---' > "$SOURCE/skills/example/SKILL.md"
chmod +x "$SOURCE/shared/scripts/karpathy-hook-dispatch.sh"
node "$SOURCE/scripts/generate-harness-adapters.js" >/dev/null
node "$INSTALLER" --source-root "$SOURCE" --plugin-root "$PLUGIN_ROOT" --home "$TMP/home" >/dev/null
SECOND="$(readlink "$PLUGIN_ROOT/current")"
SECOND_HASH="${SECOND##*/}"
SECOND_BUNDLE="$(jq -r '.bundleId' "$PLUGIN_ROOT/generations/$SECOND_HASH/manifest.json")"
[[ "$SECOND" != "$FIRST" ]]
[[ "$SECOND_BUNDLE" != "$FIRST_BUNDLE" ]]
[[ "$(readlink "$PLUGIN_ROOT/previous")" == "$FIRST" ]]
PROMETHEUS_PLUGIN_ROOT="$PLUGIN_ROOT" "$PLUGIN_ROOT/runtime/v1/run-hook" \
  --bundle "$FIRST_BUNDLE" --resolve-only >/dev/null
PROMETHEUS_PLUGIN_ROOT="$PLUGIN_ROOT" "$PLUGIN_ROOT/runtime/v1/run-hook" \
  --bundle "$SECOND_BUNDLE" --resolve-only >/dev/null
FIRST_DISPATCH="$(PROMETHEUS_PLUGIN_ROOT="$PLUGIN_ROOT" "$PLUGIN_ROOT/runtime/v1/run-hook" \
  --bundle "$FIRST_BUNDLE" --hook fixture-hook --harness codex)"
SECOND_DISPATCH="$(PROMETHEUS_PLUGIN_ROOT="$PLUGIN_ROOT" "$PLUGIN_ROOT/runtime/v1/run-hook" \
  --bundle "$SECOND_BUNDLE" --hook fixture-hook --harness codex)"
[[ "$FIRST_DISPATCH" == karpathy-hook-dispatch-a ]]
[[ "$SECOND_DISPATCH" == karpathy-hook-dispatch-b ]]
UNKNOWN_BUNDLE="$(printf 'f%.0s' {1..64})"
if PROMETHEUS_PLUGIN_ROOT="$PLUGIN_ROOT" "$PLUGIN_ROOT/runtime/v1/run-hook" \
  --bundle "$UNKNOWN_BUNDLE" --resolve-only >"$TMP/unknown.out" 2>"$TMP/unknown.err"; then
  echo '[FAIL] unknown bundle unexpectedly resolved' >&2
  exit 1
fi
[[ ! -s "$TMP/unknown.out" ]]
grep -q '"code":"NOT_ACTIVATED"' "$TMP/unknown.err"
[[ "$(HOME="$TMP/home" bash "$PLUGIN_ROOT/stable/karpathy-hook-dispatch.sh")" == "karpathy-hook-dispatch-b" ]]
grep -q 'fixture-b' "$TMP/home/.codex/skills/example/SKILL.md"
grep -q 'fixture-b' "$TMP/home/.minimax/skills/example/SKILL.md"
for script in detect-project-context memory-outbox-flush pk-health; do
  cmp "$SOURCE/shared/scripts/$script.sh" "$PLUGIN_ROOT/stable/$script.sh"
done
echo '[PASS] pinned old and new bundles dispatch independently; unknown bundles fail closed'
echo '[PASS] activation atomically advances current and retains a previous pointer'

node "$INSTALLER" --plugin-root "$PLUGIN_ROOT" --home "$TMP/home" --rollback >/dev/null
[[ "$(readlink "$PLUGIN_ROOT/current")" == "$FIRST" ]]
cmp "$PLUGIN_ROOT/generations/$FIRST_HASH/indexes/skills.json" \
  "$PLUGIN_ROOT/stable/skill-index.json"
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

chmod +x "$SOURCE/shared/scripts/pk-health.sh"
mkdir -p "$SOURCE/skills/collision" \
  "$TMP/home/.claude/skills/collision" \
  "$TMP/home/.claude/skills/prometheus-collision"
printf '%s\n' '---' 'name: collision' 'description: collision fixture' '---' > \
  "$SOURCE/skills/collision/SKILL.md"
printf 'owned\n' > "$TMP/home/.claude/skills/collision/marker.txt"
printf 'owned\n' > "$TMP/home/.claude/skills/prometheus-collision/marker.txt"
if node "$INSTALLER" --source-root "$SOURCE" --plugin-root "$PLUGIN_ROOT" \
  --home "$TMP/home" >/dev/null 2>&1; then
  echo '[FAIL] dual primary/namespaced target collision unexpectedly installed' >&2
  exit 1
fi
[[ "$(readlink "$PLUGIN_ROOT/current")" == "$FIRST" ]]
echo '[PASS] unresolved target collisions reject the candidate without moving activation'

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
