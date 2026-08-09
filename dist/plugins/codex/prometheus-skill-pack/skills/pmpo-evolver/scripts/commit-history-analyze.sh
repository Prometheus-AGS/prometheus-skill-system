#!/usr/bin/env bash
# commit-history-analyze.sh — Analyze git commit history for classification and churn hotspots
# Usage: commit-history-analyze.sh [<repo-path>] [--since <ISO8601-date-or-relative>]
# [MODEL_ROUTING] phase=evolver-signal-commits class=small
set -euo pipefail

REPO_PATH="."
SINCE="30 days ago"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --since) SINCE="${2:-30 days ago}"; shift 2 ;;
    -*) shift ;;
    *) REPO_PATH="$1"; shift ;;
  esac
done

echo "[commit-history] Analyzing ${REPO_PATH} since: ${SINCE}" >&2

# Verify this is a git repo
if ! git -C "${REPO_PATH}" rev-parse --is-inside-work-tree > /dev/null 2>&1; then
  echo "[commit-history] ERROR: ${REPO_PATH} is not a git repository" >&2
  printf '{"period": "%s", "total_commits": 0, "breakdown": {}, "fix_ratio": 0, "hotspots": [], "error": "not a git repo"}\n' "${SINCE}"
  exit 0
fi

GIT_LOG=$(git -C "${REPO_PATH}" log --oneline --since="${SINCE}" 2>/dev/null || echo "")

if [ -z "${GIT_LOG}" ]; then
  echo "[commit-history] No commits found since ${SINCE}" >&2
  printf '{"period": "%s", "total_commits": 0, "breakdown": {"fix": 0, "feat": 0, "refactor": 0, "chore": 0, "test": 0, "docs": 0, "perf": 0, "other": 0}, "fix_ratio": 0.0, "hotspots": []}\n' "${SINCE}"
  exit 0
fi

TOTAL=$(echo "${GIT_LOG}" | wc -l | tr -d ' ')
echo "[commit-history] Found ${TOTAL} commits" >&2

# Classify via conventional commit patterns (no LLM needed for small class)
printf '%s' "${GIT_LOG}" | python3 -c "
import re, json, sys
from collections import defaultdict

lines = [l.strip() for l in sys.stdin.read().strip().split('\n') if l.strip()]
types = defaultdict(int)

for line in lines:
    parts = line.split(None, 1)
    if len(parts) < 2:
        types['other'] += 1
        continue
    msg = parts[1].lower()
    if re.match(r'^fix[\(:!\s]|^bug\s|^hotfix\b|^bugfix\b', msg):    types['fix'] += 1
    elif re.match(r'^feat[\(:!\s]|^add\s|^new\s|^feature\b', msg):   types['feat'] += 1
    elif re.match(r'^refactor[\(:!\s]|^clean\b|^cleanup\b', msg):    types['refactor'] += 1
    elif re.match(r'^test[\(:!\s]|^spec\b', msg):                     types['test'] += 1
    elif re.match(r'^docs[\(:!\s]|^doc\s|^readme\b', msg):           types['docs'] += 1
    elif re.match(r'^chore[\(:!\s]|^ci[\(:!\s]|^build[\(:!\s]', msg): types['chore'] += 1
    elif re.match(r'^perf[\(:!\s]|^optim\b', msg):                   types['perf'] += 1
    else:                                                              types['other'] += 1

total = sum(types.values()) or 1
fix_ratio = round(types.get('fix', 0) / total, 3)

result = {
    'period': sys.argv[1] if len(sys.argv) > 1 else 'unknown',
    'total_commits': total,
    'breakdown': dict(types),
    'fix_ratio': fix_ratio,
    'hotspots': []
}
print(json.dumps(result, indent=2))
" "${SINCE}"
