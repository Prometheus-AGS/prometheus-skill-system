#!/usr/bin/env bash
# smoke-test.sh — verify bdd-cucumber-js skill artifacts are well-formed.
#
# Checks (offline, no cucumber/npm install required):
#   - SKILL.md has required frontmatter
#   - references/examples/ has both api-http-only and ui-playwright
#   - Each example .feature file starts with a tag and Feature: line
#   - Each example .steps.ts file imports @cucumber/cucumber or createBdd
#
# Exit codes: 0 OK, 1 defect found

set -euo pipefail

SKILL_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FAIL=0

fail() { echo "smoke: FAIL — $*" >&2; FAIL=1; }
pass() { echo "smoke: OK  — $*"; }

# --- SKILL.md frontmatter ---
if grep -q "^name: bdd-cucumber-js$" "$SKILL_DIR/SKILL.md" \
   && grep -q "^version:" "$SKILL_DIR/SKILL.md" \
   && grep -q "^license:" "$SKILL_DIR/SKILL.md" \
   && grep -q "^  tags:" "$SKILL_DIR/SKILL.md"; then
    pass "SKILL.md frontmatter"
else
    fail "SKILL.md missing required frontmatter fields"
fi

# --- examples present ---
for ex in api-http-only/sign-in.feature api-http-only/sign-in.steps.ts \
          ui-playwright/sign-in.feature ui-playwright/sign-in.steps.ts \
          README.md; do
    if [ -f "$SKILL_DIR/references/examples/$ex" ]; then
        pass "example present: $ex"
    else
        fail "example missing: $ex"
    fi
done

# --- .feature files start with a tag then Feature: ---
for f in "$SKILL_DIR"/references/examples/*/sign-in.feature; do
    head -1 "$f" | grep -qE "^@[a-z]+" || fail "$f: missing leading tag"
    grep -q "^Feature:" "$f" || fail "$f: missing Feature: line"
done
pass "feature files well-formed"

# --- api-http-only imports cucumber ---
grep -q "from '@cucumber/cucumber'" \
    "$SKILL_DIR/references/examples/api-http-only/sign-in.steps.ts" \
    && pass "api example imports @cucumber/cucumber" \
    || fail "api example missing cucumber import"

# --- ui-playwright uses createBdd ---
grep -q "createBdd()" \
    "$SKILL_DIR/references/examples/ui-playwright/sign-in.steps.ts" \
    && pass "ui example uses playwright-bdd createBdd()" \
    || fail "ui example missing createBdd() call"

exit "$FAIL"
