#!/usr/bin/env bash
# smoke-test.sh — verify bdd-video-proof skill artifacts are well-formed
# and mint-certification-bundle.sh dry-run works end-to-end.
#
# Exit codes: 0 OK, 1 defect found

set -euo pipefail

SKILL_DIR="$(cd "$(dirname "$0")/.." && pwd)"
FAIL=0

fail() { echo "smoke: FAIL — $*" >&2; FAIL=1; }
pass() { echo "smoke: OK  — $*"; }

# --- SKILL.md frontmatter (v2.0) ---
if grep -q "^name: bdd-video-proof$" "$SKILL_DIR/SKILL.md" \
   && grep -q "^version: '2\." "$SKILL_DIR/SKILL.md" \
   && grep -q "^license:" "$SKILL_DIR/SKILL.md" \
   && grep -q "^  tags:" "$SKILL_DIR/SKILL.md"; then
    pass "SKILL.md frontmatter (v2.x)"
else
    fail "SKILL.md missing required frontmatter or not v2.x"
fi

# --- references present ---
for ref in SETUP.md IPFS.md; do
    if [ -f "$SKILL_DIR/references/$ref" ]; then
        pass "reference present: $ref"
    else
        fail "reference missing: $ref"
    fi
done

# --- mint-certification-bundle.sh parses + is executable ---
MINT="$SKILL_DIR/scripts/mint-certification-bundle.sh"
if bash -n "$MINT"; then
    pass "mint-certification-bundle.sh parses"
else
    fail "mint-certification-bundle.sh: bash syntax error"
fi
if [ -x "$MINT" ]; then
    pass "mint-certification-bundle.sh is executable"
else
    fail "mint-certification-bundle.sh not executable"
fi

# --- dry-run end-to-end (needs git + jq available) ---
if ! command -v jq >/dev/null 2>&1; then
    echo "smoke: SKIP dry-run (jq not on PATH)"
elif ! command -v git >/dev/null 2>&1; then
    echo "smoke: SKIP dry-run (git not on PATH)"
else
    TMP_JSON=$(mktemp)
    echo '{"features":[]}' > "$TMP_JSON"
    if bash "$MINT" \
        --module bdd-video-proof \
        --cucumber-json "$TMP_JSON" \
        --dry-run >/dev/null 2>&1; then
        pass "mint dry-run succeeds on minimal input"
    else
        fail "mint dry-run failed on minimal input"
    fi
    rm -f "$TMP_JSON"
fi

# --- required args enforced ---
if bash "$MINT" >/dev/null 2>&1; then
    fail "mint should reject invocation without --module"
else
    pass "mint rejects invocation without required args"
fi

exit "$FAIL"
