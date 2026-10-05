#!/usr/bin/env bash
# test-codex-memories-config.sh — integration tests for codex-memories-config.sh and
# the two installer entry points that call it. Everything runs against a scratch
# HOME / CODEX_HOME (no auth.json: the operator's credentials are unreachable).
# Deterministic cases run first; the `codex debug prompt-input` probe runs last and
# prints SKIP when codex is absent or refuses unauthenticated (REQUIRE_CODEX_PROBE=1
# turns that SKIP into exit 2).
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
TARGET="$REPO_ROOT/shared/scripts/codex-memories-config.sh"
HELPER="$REPO_ROOT/scripts/lib/install-codex-memories.sh"
INSTALL_SYSTEM="$REPO_ROOT/scripts/install-system.js"

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); }
bad() { FAIL=$((FAIL+1)); printf 'FAIL: %s — %s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
n=0
fresh_home() { n=$((n+1)); H="$TMP/home$n"; mkdir -p "$H/.codex"; CH="$H/.codex"; }
run_target() { HOME="$H" CODEX_HOME="$CH" /bin/bash "$TARGET" "$@"; }

SEED='# my codex config
model = "gpt-5"   # keep me

[tui]
theme = "dark"

[projects."/tmp/x"]
trust_level = "trusted"
'

# 1. comments + other tables preserved, table appended, idempotent
fresh_home
printf '%s' "$SEED" > "$CH/config.toml"
run_target || bad "append" "exit $?"
grep -q '^\[memories\]$' "$CH/config.toml" && grep -q '^generate_memories = false$' "$CH/config.toml" \
  && ok || bad "append" "table missing"
head -c "${#SEED}" "$CH/config.toml" | cmp -s - <(printf '%s' "$SEED") && ok || bad "append" "existing bytes changed"
cp "$CH/config.toml" "$TMP/after1"
bk1="$(ls "$CH" | grep -c '^config.toml.bak-')"
run_target || bad "idempotent" "exit $?"
cmp -s "$CH/config.toml" "$TMP/after1" && ok || bad "idempotent" "second run changed config"
[ "$(ls "$CH" | grep -c '^config.toml.bak-')" = "$bk1" ] && ok || bad "idempotent" "second run wrote a backup"
[ "$bk1" = 1 ] && ok || bad "backup" "expected one timestamped backup, got $bk1"

# 2. true -> false in place, no duplicates, comment kept
fresh_home
printf 'a = 1\n[memories]\ngenerate_memories = true  # was on\nother = 2\n[x]\ny = 3\n' > "$CH/config.toml"
run_target || bad "flip" "exit $?"
[ "$(grep -c 'generate_memories' "$CH/config.toml")" = 1 ] && grep -q '^generate_memories = false  # was on$' "$CH/config.toml" \
  && ok || bad "flip" "not changed in place: $(cat "$CH/config.toml")"
grep -q '^other = 2$' "$CH/config.toml" && ok || bad "flip" "neighbour line lost"

# 2b. [memories] without the key gets it inserted inside the table
fresh_home
printf '[memories]\nother = 2\n\n[x]\ny = 3\n' > "$CH/config.toml"
run_target || bad "insert" "exit $?"
awk '/^\[memories\]/{m=1;next} /^\[/{m=0} m&&/generate_memories = false/{f=1} END{exit !f}' "$CH/config.toml" \
  && ok || bad "insert" "key not inside [memories]"

# 3. missing config.toml in an existing CODEX_HOME is created
fresh_home
run_target || bad "create" "exit $?"
grep -q '^generate_memories = false$' "$CH/config.toml" && ok || bad "create" "not created"

# 4. unparseable result -> original restored, non-zero exit
fresh_home
printf '[memories]\ngenerate_memories = true\ngenerate_memories = true\n' > "$CH/config.toml"
cp "$CH/config.toml" "$TMP/corrupt"
run_target 2>/dev/null; rc=$?
[ "$rc" -ne 0 ] && ok || bad "restore" "expected non-zero exit"
cmp -s "$CH/config.toml" "$TMP/corrupt" && ok || bad "restore" "original not restored"

# 5. summary archived, MEMORY.md / raw_memories.md untouched
fresh_home
mkdir -p "$CH/memories"
printf 'SUMMARY\n' > "$CH/memories/memory_summary.md"
printf 'MEM\n' > "$CH/memories/MEMORY.md"
printf 'RAW\n' > "$CH/memories/raw_memories.md"
run_target || bad "archive" "exit $?"
[ ! -e "$CH/memories/memory_summary.md" ] && ok || bad "archive" "summary still present"
ls "$CH/memories-archive"/memory_summary-*.md >/dev/null 2>&1 && ok || bad "archive" "no archived copy"
[ "$(cat "$CH/memories/MEMORY.md")" = MEM ] && [ "$(cat "$CH/memories/raw_memories.md")" = RAW ] && ok || bad "archive" "other memory files touched"

# 6. --check reports state and writes nothing; absent Codex is silent
fresh_home
printf '[memories]\ngenerate_memories = true\n' > "$CH/config.toml"
mkdir -p "$CH/memories"; printf 's\n' > "$CH/memories/memory_summary.md"
out="$(run_target --check)"
case "$out" in *'"generate_memories":"true"'*'"summary_present":true'*'"ok":false'*) ok ;; *) bad "check" "$out" ;; esac
grep -q 'true' "$CH/config.toml" && [ -f "$CH/memories/memory_summary.md" ] && ok || bad "check" "wrote something"
run_target; out="$(run_target --check)"
case "$out" in *'"generate_memories":"false"'*'"summary_present":false'*'"ok":true'*) ok ;; *) bad "check-after" "$out" ;; esac
H="$TMP/nohome"; mkdir -p "$H"; CH="$H/.codex"
out="$(run_target 2>&1)"; rc=$?
[ "$rc" = 0 ] && [ -z "$out" ] && [ ! -e "$CH" ] && ok || bad "absent" "rc=$rc out=$out"

# 7. failing CODEX_MEMORIES_SCRIPT: both installer entry points return 0 with a warning
printf '#!/bin/sh\nexit 1\n' > "$TMP/fail.sh"; chmod +x "$TMP/fail.sh"
fresh_home
out="$(HOME="$H" CODEX_HOME="$CH" CODEX_MEMORIES_SCRIPT="$TMP/fail.sh" REPO_ROOT="$REPO_ROOT" \
  /bin/bash -c 'set -euo pipefail; source "'"$HELPER"'"; install_codex_memories; echo "rc=$?"' 2>&1)"; rc=$?
[ "$rc" = 0 ] && case "$out" in *rc=0*) true ;; *) false ;; esac && case "$out" in *warning*|*⚠*) true ;; *) false ;; esac \
  && ok || bad "helper-fault" "rc=$rc out=$out"
out="$(cd "$REPO_ROOT" && HOME="$H" CODEX_MEMORIES_SCRIPT="$TMP/fail.sh" node --input-type=module -e \
  "import { applyCodexMemories } from '$INSTALL_SYSTEM'; const r = applyCodexMemories({ home: '$H' }); console.log('ret=' + r);" 2>&1)"; rc=$?
[ "$rc" = 0 ] && case "$out" in *WARNING*) true ;; *) false ;; esac && ok || bad "node-fault" "rc=$rc out=$out"

# 7b. node entry point succeeds on the real script under a scratch HOME
fresh_home
printf '%s' "$SEED" > "$CH/config.toml"
(cd "$REPO_ROOT" && HOME="$H" node --input-type=module -e \
  "import { applyCodexMemories } from '$INSTALL_SYSTEM'; process.exit(applyCodexMemories({ home: '$H' }) ? 0 : 1);") \
  && grep -q '^generate_memories = false$' "$CH/config.toml" && ok || bad "node-ok" "config not updated"

# 8. optional probe (last): codex prompt-input carries no seeded MEMORY.md text
probe_skip() { echo "SKIP: codex prompt-input probe ($1)"; PROBE_RC=0; [ "${REQUIRE_CODEX_PROBE:-0}" = 1 ] && PROBE_RC=2; }
PROBE_RC=0
if ! command -v codex >/dev/null 2>&1; then
  probe_skip "codex not on PATH"
else
  fresh_home
  printf '%s' "$SEED" > "$CH/config.toml"
  mkdir -p "$CH/memories"
  MARK="TLH02-MEMORY-MARKER-$$"
  printf '%s\n' "$MARK" > "$CH/memories/MEMORY.md"
  printf '%s\n' "$MARK" > "$CH/memories/memory_summary.md"
  run_target
  ( HOME="$H" CODEX_HOME="$CH" codex debug prompt-input >"$TMP/probe.out" 2>"$TMP/probe.err" ) &
  pid=$!; t=0
  while kill -0 "$pid" 2>/dev/null && [ "$t" -lt 60 ]; do sleep 1; t=$((t+1)); done
  if kill -0 "$pid" 2>/dev/null; then kill "$pid" 2>/dev/null; wait "$pid" 2>/dev/null; probe_skip "timed out"
  elif ! wait "$pid" || [ ! -s "$TMP/probe.out" ]; then probe_skip "refused to run unauthenticated"
  elif grep -q "$MARK" "$TMP/probe.out"; then bad "probe" "memories text present in prompt-input"
  else ok; fi
fi

# Regression (phase review): the install path must reach install_codex_memories.
# The top-level `if $UNINSTALL; then ... else ... fi` block is the install dispatch;
# the call must sit in its else-arm, not inside install_to_codex (uninstall-only).
if python3 - "$REPO_ROOT/scripts/install-skills-flat.sh" <<'PY'
import re, sys
lines = open(sys.argv[1]).read().split("\n")
start = next(i for i, l in enumerate(lines) if l == "if $UNINSTALL; then" and i > 200)
els = next(i for i in range(start, len(lines)) if lines[i] == "else")
end = next(i for i in range(els, len(lines)) if lines[i] == "fi")
sys.exit(0 if any(l.strip() == "install_codex_memories" for l in lines[els:end]) else 1)
PY
then ok; else bad "installer" "install-skills-flat.sh install path never calls install_codex_memories"; fi

echo "codex-memories-config: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ] || exit 1
exit "$PROBE_RC"
