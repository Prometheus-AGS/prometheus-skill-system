#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TMP_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/prometheus-install-policy.XXXXXX")"
trap 'rm -rf "$TMP_ROOT"' EXIT

prepare_home() {
    local test_home="$1"
    mkdir -p \
        "$test_home/.claude" \
        "$test_home/.opencode" \
        "$test_home/.kimi-code" \
        "$test_home/.minimax" \
        "$test_home/.cursor" \
        "$test_home/.codex" \
        "$test_home/.gemini" \
        "$test_home/.roo" \
        "$test_home/.codeium/windsurf" \
        "$test_home/.agents" \
        "$test_home/.config/zed" \
        "$test_home/.zed" \
        "$test_home/.cline"
}

strict_home="$TMP_ROOT/strict-home"
prepare_home "$strict_home"
if HOME="$strict_home" \
    PROMETHEUS_INSTALL_TEST_MODE=1 \
    PROMETHEUS_INSTALL_TEST_FAIL_COMPONENT=strict-proof \
    bash "$REPO_ROOT/scripts/install-skills-flat.sh" --skills-only \
    >"$TMP_ROOT/strict.out" 2>"$TMP_ROOT/strict.err"; then
    echo "FAIL: strict installation accepted an injected component failure" >&2
    exit 1
fi
grep -Fq 'fixture:strict-proof failed' "$TMP_ROOT/strict.err"
if grep -Fq 'installed and verified' "$TMP_ROOT/strict.out"; then
    echo "FAIL: strict failure printed a false success message" >&2
    exit 1
fi

best_effort_home="$TMP_ROOT/best-effort-home"
prepare_home "$best_effort_home"
HOME="$best_effort_home" \
    PROMETHEUS_INSTALL_TEST_MODE=1 \
    PROMETHEUS_INSTALL_TEST_FAIL_COMPONENT=best-effort-proof \
    bash "$REPO_ROOT/scripts/install-skills-flat.sh" --skills-only --best-effort \
    >"$TMP_ROOT/best-effort.out" 2>"$TMP_ROOT/best-effort.err"
grep -Fq 'continuing only because --best-effort is active' "$TMP_ROOT/best-effort.err"
grep -Eq 'best-effort run completed with [1-9][0-9]* failed component' "$TMP_ROOT/best-effort.out"
if grep -Fq 'installed and verified' "$TMP_ROOT/best-effort.out"; then
    echo "FAIL: best-effort failure printed a false all-green message" >&2
    exit 1
fi

skills_home="$TMP_ROOT/skills-home"
prepare_home "$skills_home"
HOME="$skills_home" bash "$REPO_ROOT/scripts/install-skills-flat.sh" --skills-only \
    >"$TMP_ROOT/skills.out" 2>"$TMP_ROOT/skills.err"
grep -Fq 'installed and verified' "$TMP_ROOT/skills.out"
generation="$(HOME="$skills_home" node "$REPO_ROOT/scripts/install-plugin-generation.js" \
    --home "$skills_home" --verify)"
[[ -n "$generation" ]]

echo "PASS: installer strict, best-effort, skills-only, and false-green policies"
