#!/usr/bin/env bash
# smoke-test.sh — verify bdd-cucumber-rs skill artifacts are well-formed.
#
# Offline checks:
#   - SKILL.md has required frontmatter
#   - references/examples/ has both api-http-only and ui-thirtyfour crates
#   - Each example Cargo.toml declares cucumber 0.23
#   - Each example .feature file starts with a tag and Feature: line
#   - Each example features.rs uses #[derive(World)] and tokio::main
#
# Exit codes: 0 OK, 1 defect found

set -euo pipefail

SKILL_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FAIL=0

fail() { echo "smoke: FAIL — $*" >&2; FAIL=1; }
pass() { echo "smoke: OK  — $*"; }

# --- SKILL.md frontmatter ---
if grep -q "^name: bdd-cucumber-rs$" "$SKILL_DIR/SKILL.md" \
   && grep -q "^version:" "$SKILL_DIR/SKILL.md" \
   && grep -q "^license:" "$SKILL_DIR/SKILL.md" \
   && grep -q "^  tags:" "$SKILL_DIR/SKILL.md"; then
    pass "SKILL.md frontmatter"
else
    fail "SKILL.md missing required frontmatter fields"
fi

# --- references present ---
for ref in browser-drivers.md migration-from-0.20.md; do
    if [ -f "$SKILL_DIR/references/$ref" ]; then
        pass "reference present: $ref"
    else
        fail "reference missing: $ref"
    fi
done

# --- example crates present ---
for ex in api-http-only/Cargo.toml api-http-only/tests/features.rs \
          api-http-only/tests/features/sign-in.feature \
          ui-thirtyfour/Cargo.toml ui-thirtyfour/tests/features.rs \
          ui-thirtyfour/tests/features/sign-in.feature \
          README.md; do
    if [ -f "$SKILL_DIR/references/examples/$ex" ]; then
        pass "example present: $ex"
    else
        fail "example missing: $ex"
    fi
done

# --- both Cargo.toml files pin cucumber 0.23 ---
for cargo in "$SKILL_DIR/references/examples/api-http-only/Cargo.toml" \
             "$SKILL_DIR/references/examples/ui-thirtyfour/Cargo.toml"; do
    grep -qE '^cucumber = "0\.23"' "$cargo" \
        && pass "$(basename "$(dirname "$cargo")")/Cargo.toml pins cucumber 0.23" \
        || fail "$cargo: cucumber not pinned to 0.23"
done

# --- feature files well-formed ---
for f in "$SKILL_DIR"/references/examples/*/tests/features/sign-in.feature; do
    head -1 "$f" | grep -qE "^@[a-z]+" || fail "$f: missing leading tag"
    grep -q "^Feature:" "$f" || fail "$f: missing Feature: line"
done
pass "feature files well-formed"

# --- features.rs uses derive(World) + tokio::main ---
for f in "$SKILL_DIR"/references/examples/*/tests/features.rs; do
    grep -q "#\[derive(.*World)\]" "$f" \
        || fail "$f: missing #[derive(World)]"
    grep -q "^#\[tokio::main\]" "$f" \
        || fail "$f: missing #[tokio::main]"
done
pass "features.rs uses derive(World) and tokio::main"

exit "$FAIL"
