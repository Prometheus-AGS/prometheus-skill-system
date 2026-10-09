#!/usr/bin/env bash
# test-codex-catalog-config.sh — integration test for codex-catalog-config.sh and its
# installer entry point scripts/lib/install-codex-catalog.sh. Everything runs against
# a scratch HOME / CODEX_HOME / fixture repo (no auth.json: the operator's credentials
# and real catalog are unreachable). What Codex actually lists is read back with the
# real `codex debug prompt-input`; the test prints SKIP when codex is absent
# (REQUIRE_CODEX_PROBE=1 turns that SKIP into exit 2).
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REAL_REPO="$(cd "$SCRIPT_DIR/../../.." && pwd)"

if ! command -v codex >/dev/null 2>&1; then
  echo "SKIP: codex is not installed"
  [ "${REQUIRE_CODEX_PROBE:-0}" = "1" ] && exit 2
  exit 0
fi

PASS=0; FAIL=0
ok()  { PASS=$((PASS+1)); }
bad() { FAIL=$((FAIL+1)); printf 'FAIL: %s — %s\n' "$1" "${2:-}" >&2; }

TMP="$(mktemp -d)"; trap 'rm -rf "$TMP"' EXIT
H="$TMP/home"; CH="$H/.codex"; REPO="$TMP/repo"
mkdir -p "$CH/skills" "$H/.agents/skills/ecc/ecc-one" "$REPO/skills/process" "$REPO/config" \
         "$REPO/scripts/lib" "$REPO/shared/scripts"

skill() { # dir name body
  mkdir -p "$1"
  printf -- '---\nname: %s\ndescription: %s skill long enough to appear in the Codex catalog\n---\n%s\n' "$2" "$2" "${3:-body}" > "$1/SKILL.md"
}

# Pack source: alpha and the bundle are selected, beta is not; the bundle has a nested step.
skill "$REPO/skills/process/alpha" alpha
skill "$REPO/skills/process/beta" beta
skill "$REPO/skills/process/bundle" bundle
skill "$REPO/skills/process/bundle/skills/child" bundle-child
cat > "$REPO/config/codex-catalog.txt" <<'CAT'
exclude skills/**
include skills/process/alpha
include skills/process/bundle
CAT
cp "$REAL_REPO/scripts/register-slash-commands.sh" "$REPO/scripts/"
cp "$REAL_REPO/scripts/lib/install-codex-catalog.sh" "$REPO/scripts/lib/"
cp "$REAL_REPO/shared/scripts/codex-catalog-config.sh" "$REPO/shared/scripts/"

# What the immutable-generation installer leaves behind: real copies carrying the
# generation marker, a prometheus-* alias, a .foreign backup, a symlink in ~/.agents.
for n in alpha beta bundle; do
  skill "$CH/skills/$n" "$n"; : > "$CH/skills/$n/.prometheus-generation"
done
skill "$CH/skills/prometheus-alpha" alpha; : > "$CH/skills/prometheus-alpha/.prometheus-generation"
skill "$CH/skills/alpha.foreign-20260101" alpha
skill "$CH/skills/bundle/skills/child" bundle-child
ln -s "$CH/skills/alpha" "$H/.agents/skills/alpha"
# Third party: an identical duplicate collapses; different skills sharing a name do not.
skill "$CH/skills/ecc-one" ecc-one "same body"
skill "$H/.agents/skills/ecc/ecc-one" ecc-one "same body"
skill "$H/.agents/skills/keepme" keepme
skill "$H/.agents/skills/dropme" dropme
skill "$CH/skills/shared-name" shared-name "first"
skill "$H/.agents/skills/shared-name" shared-name "second"

SEED='# my codex config
model = "gpt-5"   # keep me

[tui]
theme = "dark"

[[skills.config]]
name = "something-user-disabled"
enabled = false
'
printf '%s' "$SEED" > "$CH/config.toml"

helper() { HOME="$H" CODEX_HOME="$CH" bash "$REPO/shared/scripts/codex-catalog-config.sh" --repo-root "$REPO" "$@"; }
entry()  { HOME="$H" CODEX_HOME="$CH" bash "$REPO/scripts/lib/install-codex-catalog.sh" "$REPO"; }
listing() { # prints "name<TAB>file" per entry as Codex sees them from the fixture repo
  (cd "$REPO" && HOME="$H" CODEX_HOME="$CH" codex debug prompt-input x 2>/dev/null) | python3 -c '
import json,re,sys
text=""
for item in json.load(sys.stdin):
    for chunk in item.get("content",[]):
        if "### Available skills" in chunk.get("text",""): text=chunk["text"]
roots=dict(re.findall(r"- `(r\d+)` = `([^`]+)`",text))
for line in text[text.index("### Available skills"):].splitlines():
    m=re.match(r"- (\S+?):\s+(.*?)\s*\(file: ([^)]+)\)\s*$",line)
    if m:
        f=m.group(3); r=re.match(r"(r\d+)/(.*)",f)
        print(m.group(1).rpartition(":")[2]+"\t"+(roots[r.group(1)]+"/"+r.group(2) if r and r.group(1) in roots else f))
'
}
count() { listing | awk -F'\t' -v n="$1" '$1==n' | wc -l | tr -d ' '; }

# 0. baseline: Codex really lists the duplicates (guards against a vacuous test)
[ "$(count alpha)" -ge 3 ] && ok || bad "baseline" "expected duplicate alpha entries, got $(count alpha)"

# 1. --check before applying reports drift
helper --check | python3 -c 'import json,sys; sys.exit(0 if json.load(sys.stdin)["ok"] is False else 1)' && ok || bad "check-before" "should report ok=false"

# 2. refuses while an excluded skill (beta) has no slash-command prompt; config untouched
helper >/dev/null 2>&1; rc=$?
[ "$rc" -eq 1 ] && ok || bad "missing-prompt" "expected exit 1, got $rc"
cmp -s "$CH/config.toml" <(printf '%s' "$SEED") && ok || bad "missing-prompt" "config changed despite refusal"

# 3. installer entry point registers prompts, then applies
entry >/dev/null 2>"$TMP/entry.err" || bad "entry" "exit $? $(cat "$TMP/entry.err")"
[ -f "$CH/prompts/beta.md" ] && ok || bad "prompts" "beta has no slash-command prompt"
grep -q 'prometheus-codex-catalog' "$CH/config.toml" && ok || bad "block" "managed block missing"
head -c "${#SEED}" "$CH/config.toml" | cmp -s - <(printf '%s' "$SEED") && ok || bad "preserve" "existing bytes changed"
grep -q 'something-user-disabled' "$CH/config.toml" && ok || bad "preserve" "user skills.config entry lost"

# 4. the catalog Codex now lists
[ "$(count alpha)" -eq 1 ] && ok || bad "alpha" "expected exactly one alpha, got $(count alpha)"
listing | awk -F'\t' '$1=="alpha"{print $2}' | grep -q "^$(cd "$CH" && pwd -P)/skills/alpha/SKILL.md$" && ok || bad "alpha-canonical" "wrong canonical copy"
[ "$(count beta)" -eq 0 ] && ok || bad "beta" "unselected skill still listed"
[ "$(count bundle)" -eq 1 ] && ok || bad "bundle" "selected bundle should be listed once"
[ "$(count bundle-child)" -eq 0 ] && ok || bad "bundle-child" "nested bundle step still listed"
[ "$(count ecc-one)" -eq 1 ] && ok || bad "ecc-one" "identical third-party duplicate not collapsed"
[ "$(count shared-name)" -eq 2 ] && ok || bad "shared-name" "different skills sharing a name must both stay"

# 5. idempotent: second run changes nothing and makes no new backup
cp "$CH/config.toml" "$TMP/after1"; b1="$(ls "$CH" | grep -c '^config.toml.bak-')"
entry >/dev/null 2>&1 || bad "idempotent" "second run exit $?"
cmp -s "$CH/config.toml" "$TMP/after1" && ok || bad "idempotent" "second run changed config"
[ "$(ls "$CH" | grep -c '^config.toml.bak-')" -eq "$b1" ] && ok || bad "idempotent" "second run wrote a backup"
helper --check | python3 -c 'import json,sys; sys.exit(0 if json.load(sys.stdin)["ok"] is True else 1)' && ok || bad "check-after" "should report ok=true"

# 6. a changed policy re-enables a skill (beta) on the next run
printf 'include skills/process/beta\n' >> "$REPO/config/codex-catalog.txt"
entry >/dev/null 2>&1 || bad "policy-change" "exit $?"
[ "$(count beta)" -eq 1 ] && ok || bad "policy-change" "beta not re-enabled"

# 7. third-party policy: with `third-party exclude`, only keep-name / keep-path survive
printf 'third-party exclude\nkeep-name keepme\n' >> "$REPO/config/codex-catalog.txt"
entry >/dev/null 2>&1 || bad "third-party" "exit $?"
[ "$(count keepme)" -eq 1 ] && ok || bad "keep-name" "kept third-party skill not listed"
[ "$(count dropme)" -eq 0 ] && ok || bad "third-party" "unlisted third-party skill still listed"
[ "$(count ecc-one)" -eq 0 ] && [ "$(count shared-name)" -eq 0 ] && ok || bad "third-party" "excluded third-party copies still listed"
[ "$(count alpha)" -eq 1 ] && [ "$(count beta)" -eq 1 ] && ok || bad "third-party" "pack selection changed by the third-party policy"
cp "$CH/config.toml" "$TMP/after-tp"
entry >/dev/null 2>&1 || bad "third-party-idempotent" "exit $?"
cmp -s "$CH/config.toml" "$TMP/after-tp" && ok || bad "third-party-idempotent" "second run changed config"

# 8. --uninstall restores the seed
helper --uninstall >/dev/null 2>&1 || bad "uninstall" "exit $?"
cmp -s "$CH/config.toml" <(printf '%s' "$SEED") && ok || bad "uninstall" "config not restored byte for byte"
[ "$(count alpha)" -ge 3 ] && ok || bad "uninstall-listing" "duplicates should be listed again"

echo "codex-catalog-config: $PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
