#!/usr/bin/env bash
# smoke-test.sh — verify bdd-lifecycle-loop skill artifacts are well-formed
# and its bundled scripts behave sanely on obvious inputs.
#
# Exit codes: 0 OK, 1 defect found

set -euo pipefail

SKILL_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FAIL=0

fail() { echo "smoke: FAIL — $*" >&2; FAIL=1; }
pass() { echo "smoke: OK  — $*"; }

# --- SKILL.md frontmatter ---
if grep -q "^name: bdd-lifecycle-loop$" "$SKILL_DIR/SKILL.md" \
   && grep -q "^version:" "$SKILL_DIR/SKILL.md" \
   && grep -q "^license:" "$SKILL_DIR/SKILL.md" \
   && grep -q "^  tags:" "$SKILL_DIR/SKILL.md"; then
    pass "SKILL.md frontmatter"
else
    fail "SKILL.md missing required frontmatter fields"
fi

# --- references present ---
for ref in immutable-tests.md visual-baseline-refresh.md; do
    if [ -f "$SKILL_DIR/references/$ref" ]; then
        pass "reference present: $ref"
    else
        fail "reference missing: $ref"
    fi
done

# --- scripts parse ---
for s in flake-budget.sh test-file-diff-guard.sh; do
    if bash -n "$SKILL_DIR/scripts/$s"; then
        pass "$s parses"
    else
        fail "$s: bash syntax error"
    fi
    if [ -x "$SKILL_DIR/scripts/$s" ]; then
        pass "$s is executable"
    else
        fail "$s not executable"
    fi
done

# --- test-file-diff-guard: override path exits 0 ---
if BDD_ALLOW_TEST_EDITS=1 bash "$SKILL_DIR/scripts/test-file-diff-guard.sh" \
        HEAD HEAD >/dev/null 2>&1; then
    pass "test-file-diff-guard: override path exits 0"
else
    fail "test-file-diff-guard: override path did not exit 0"
fi

# --- flake-budget: bad-args path exits non-zero ---
TMP=$(mktemp -d)
if bash "$SKILL_DIR/scripts/flake-budget.sh" /nonexistent /nonexistent \
        >/dev/null 2>&1; then
    fail "flake-budget: expected non-zero exit on missing paths"
else
    pass "flake-budget: rejects missing paths"
fi
rm -rf "$TMP"

exit "$FAIL"
