#!/usr/bin/env bash
# flake-budget.sh — enforce a project's BDD flake budget.
#
# Reads .bdd-flake-budget.json (or the file passed as $2) and counts
# scenarios tagged @flaky in the given features tree. Fails when:
#   - count > max_flaky_scenarios, OR
#   - any @flaky scenario is older than max_flaky_age_days (git blame),
#     unless it appears in grace_scenarios[]
#
# Usage:
#   flake-budget.sh <features-dir> [budget-file]
#
# Exit codes:
#   0  — under budget
#   1  — over budget or malformed input
#   2  — jq or git not available

set -euo pipefail

FEATURES_DIR="${1:-tests/features}"
BUDGET_FILE="${2:-.bdd-flake-budget.json}"

if ! command -v jq >/dev/null 2>&1; then
    echo "flake-budget: jq is required" >&2
    exit 2
fi

if ! command -v git >/dev/null 2>&1; then
    echo "flake-budget: git is required" >&2
    exit 2
fi

if [ ! -d "$FEATURES_DIR" ]; then
    echo "flake-budget: features dir not found: $FEATURES_DIR" >&2
    exit 1
fi

if [ ! -f "$BUDGET_FILE" ]; then
    echo "flake-budget: budget file not found: $BUDGET_FILE" >&2
    exit 1
fi

MAX_COUNT="$(jq -r '.max_flaky_scenarios // 5' "$BUDGET_FILE")"
MAX_AGE_DAYS="$(jq -r '.max_flaky_age_days // 14' "$BUDGET_FILE")"
GRACE_LIST="$(jq -r '.grace_scenarios // [] | .[]' "$BUDGET_FILE")"

TODAY_EPOCH="$(date +%s)"
MAX_AGE_SEC=$((MAX_AGE_DAYS * 86400))

# Find every @flaky scenario. Cucumber tags are `@name` on a line above
# `Scenario:` or above the `Feature:` block. Report file:line:title.
FLAKY_LINES=$(grep -rn -B0 -A1 "^\s*@flaky" "$FEATURES_DIR" \
    --include="*.feature" 2>/dev/null \
    | grep -E "^\S+:\s*(Scenario|Scenario Outline|Feature):" \
    | awk -F: '{print $1":"$2}' \
    || true)

COUNT=0
OVER_AGE_COUNT=0
FAILURES=()

if [ -n "$FLAKY_LINES" ]; then
    while IFS=: read -r FILE LINE; do
        [ -z "$FILE" ] && continue
        COUNT=$((COUNT + 1))
        KEY="${FILE}:${LINE}"

        # Skip grace-listed entries
        if echo "$GRACE_LIST" | grep -qxF "$KEY"; then
            continue
        fi

        # Look up when the @flaky tag was first added on the preceding line
        TAG_LINE=$((LINE - 1))
        BLAME_TS=$(git log --follow --diff-filter=A --format=%ct -1 \
            -L "${TAG_LINE},${TAG_LINE}:${FILE}" 2>/dev/null \
            | head -1 || true)

        if [ -z "$BLAME_TS" ]; then
            # Fall back to file creation date
            BLAME_TS=$(git log --diff-filter=A --format=%ct -- "$FILE" \
                | tail -1 || true)
        fi

        if [ -n "$BLAME_TS" ]; then
            AGE=$((TODAY_EPOCH - BLAME_TS))
            if [ "$AGE" -gt "$MAX_AGE_SEC" ]; then
                OVER_AGE_COUNT=$((OVER_AGE_COUNT + 1))
                FAILURES+=("$KEY — flaky > ${MAX_AGE_DAYS} days")
            fi
        fi
    done <<< "$FLAKY_LINES"
fi

echo "flake-budget: found $COUNT @flaky scenarios (max $MAX_COUNT)"
echo "flake-budget: $OVER_AGE_COUNT scenario(s) past max age $MAX_AGE_DAYS days"

if [ "$COUNT" -gt "$MAX_COUNT" ]; then
    echo "flake-budget: FAIL — over count budget ($COUNT > $MAX_COUNT)" >&2
    exit 1
fi

if [ "${#FAILURES[@]}" -gt 0 ]; then
    printf 'flake-budget: FAIL — %s\n' "${FAILURES[@]}" >&2
    exit 1
fi

echo "flake-budget: OK"
exit 0
