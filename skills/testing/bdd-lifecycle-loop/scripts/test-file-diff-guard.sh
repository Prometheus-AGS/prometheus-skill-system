#!/usr/bin/env bash
# test-file-diff-guard.sh — CI guard for the immutable-tests rule.
#
# Fails when a PR modifies protected test files (steps, features under
# non-draft dirs, or support) unless the PR is labeled
# "test-change-approved" or the environment variable
# BDD_ALLOW_TEST_EDITS=1 is set (for local human-driven refactors).
#
# Usage:
#   test-file-diff-guard.sh <base-ref> [head-ref]
#
# Example (GitHub Actions):
#   test-file-diff-guard.sh origin/main HEAD
#
# Exit codes:
#   0  — no protected files touched, or override present
#   1  — protected file diff present without override
#   2  — git not available

set -euo pipefail

BASE_REF="${1:?base ref required (e.g., origin/main)}"
HEAD_REF="${2:-HEAD}"

if ! command -v git >/dev/null 2>&1; then
    echo "test-file-diff-guard: git is required" >&2
    exit 2
fi

# Overrides
if [ "${BDD_ALLOW_TEST_EDITS:-0}" = "1" ]; then
    echo "test-file-diff-guard: BDD_ALLOW_TEST_EDITS=1 — override active"
    exit 0
fi

if [ -n "${GITHUB_ACTIONS:-}" ] && [ -n "${GITHUB_EVENT_PATH:-}" ]; then
    if command -v jq >/dev/null 2>&1; then
        if jq -e '.pull_request.labels[]?.name | select(. == "test-change-approved")' \
            "$GITHUB_EVENT_PATH" >/dev/null 2>&1; then
            echo "test-file-diff-guard: 'test-change-approved' label present — override active"
            exit 0
        fi
    fi
fi

PROTECTED_GLOBS=(
    "tests/steps/*"
    "tests/support/*"
    "tests/features/*"
)

# Compute the diff. Exclude tests/features/drafts/ — those are allowed to
# change freely per BDD-006.
CHANGED=$(git diff --name-only "$BASE_REF" "$HEAD_REF" 2>/dev/null || true)

VIOLATIONS=()
while IFS= read -r FILE; do
    [ -z "$FILE" ] && continue
    # Draft features are allowed
    case "$FILE" in
        tests/features/drafts/*) continue ;;
    esac
    for PAT in "${PROTECTED_GLOBS[@]}"; do
        # shellcheck disable=SC2053
        if [[ "$FILE" == $PAT ]]; then
            VIOLATIONS+=("$FILE")
            break
        fi
    done
done <<< "$CHANGED"

if [ "${#VIOLATIONS[@]}" -gt 0 ]; then
    echo "test-file-diff-guard: FAIL — protected test files modified without override" >&2
    printf '  %s\n' "${VIOLATIONS[@]}" >&2
    echo "" >&2
    echo "To proceed, either:" >&2
    echo "  1. Add the 'test-change-approved' label to the PR (human review confirmed)" >&2
    echo "  2. Move the change to tests/features/drafts/ (BDD-007 candidate)" >&2
    echo "  3. Set BDD_ALLOW_TEST_EDITS=1 in the environment (local runs only)" >&2
    exit 1
fi

echo "test-file-diff-guard: OK (no protected test files touched)"
exit 0
