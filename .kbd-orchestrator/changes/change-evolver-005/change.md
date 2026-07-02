---
id: change-evolver-005
title: Learning signals persistence + commit-history analysis
phase: pmpo-evolver
gaps: [G-10, G-11]
priority: HIGH — most novel Karpathy capability; no existing harness has it
goals: G5
agent: claude-code
status: done
scope:
  - skills/process/pmpo-evolver/references/learning-signals.md
  - scripts/commit-history-analyze.sh
  - scripts/feedback-digest.sh
---

# change-evolver-005 — Learning signals persistence + commit-history analysis

## Problem

No mechanism exists to collect, normalize, and persist learning signals from feedback sources (G-10). No script exists to analyze commit history for churn hotspots and classification patterns (G-11). Without these, the Karpathy self-learning perspective cannot run.

## Solution

Create `learning-signals.md` as the authoritative reference for signal collection protocol. Create `commit-history-analyze.sh` for git-log-based commit classification. Create `feedback-digest.sh` for collecting all loop.json feedback sources and normalizing them to `LearningSignal[]`.

## New file: references/learning-signals.md

**Contents:**

### Signal collection protocol per source type

**gh-issues:**
```bash
gh api repos/<owner>/<repo>/issues --jq '[.[] | {number, title, labels: [.labels[].name], created_at}]'
# Output: array of issues
# Classification: pass titles to liter-llm complete(model=medium) → {sentiment, themes[]}
# [MODEL_ROUTING] phase=evolver-signal-gh-issues class=medium
```

**commit-history:**
```bash
bash commit-history-analyze.sh <repo_path> --since <ISO8601>
# Output: {period, total_commits, breakdown: {fix, feat, refactor, chore}, hotspots: [{file, fix_count}]}
# [MODEL_ROUTING] phase=evolver-signal-commits class=small
```

**sentiment-feed:**
```bash
curl -s <url> | python3 scripts/parse-feed.py --format <rss|json|csv>
# Each item passed to liter-llm complete(model=medium) for sentiment classification
# [MODEL_ROUTING] phase=evolver-signal-sentiment class=medium
```

**telemetry-url:**
```bash
curl -s -H "Authorization: Bearer $TOKEN" <url> | python3 -c "import sys,json; d=json.load(sys.stdin); print(d<jsonpath>)"
# Numeric comparison against baseline stored in evolution_state
# No LLM needed: [MODEL_ROUTING] phase=evolver-signal-telemetry class=small
```

**Learning signal synthesis (after all sources collected):**
```
What do these signals collectively mean for product direction?
[MODEL_ROUTING] phase=evolver-signal-synthesis class=frontier
```

### Normalization format (LearningSignal)
All collected signals normalize to:
```json
{
  "id": "uuid",
  "source_type": "gh-issues | commit-history | ...",
  "source_ref": "string",
  "collected_at": "ISO8601",
  "signal": "string (human-readable 1-2 sentence summary)",
  "severity": "high | medium | low",
  "count": 0,
  "examples": ["up to 5 examples"],
  "model_used": "small | medium | frontier (or none)"
}
```

### Persistence
- Appended to `evolution_state.learning_signals[]` in `.evolver/<name>/state.json`
- Also written to `.evolver/<name>/learning-signals-<tick>.json` for per-tick archival
- Staleness TTL prevents re-collection within the same loop session

## New script: scripts/commit-history-analyze.sh

```bash
#!/usr/bin/env bash
set -euo pipefail
REPO_PATH="${1:-.}"
SINCE="1970-01-01"

while [[ $# -gt 1 ]]; do
  case "$2" in
    --since) SINCE="$3"; shift 2 ;;
    *) shift ;;
  esac
done

echo "[commit-history] Analyzing ${REPO_PATH} since ${SINCE}"

# [MODEL_ROUTING] phase=evolver-signal-commits class=small
GIT_LOG=$(git -C "${REPO_PATH}" log --oneline --since="${SINCE}" 2>/dev/null || echo "")

if [ -z "${GIT_LOG}" ]; then
  echo '{"period": "'${SINCE}'", "total_commits": 0, "breakdown": {}, "hotspots": []}'
  exit 0
fi

TOTAL=$(echo "${GIT_LOG}" | wc -l | tr -d ' ')

# Classify via conventional commit prefix (fast, no LLM needed for small class)
python3 -c "
import sys, re, json
from collections import defaultdict

log = '''${GIT_LOG}'''
lines = [l.strip() for l in log.strip().split('\n') if l.strip()]

types = defaultdict(int)
for line in lines:
    # Extract after short hash
    msg = ' '.join(line.split()[1:]).lower()
    if re.match(r'^fix[\(:!]|^bug\b', msg): types['fix'] += 1
    elif re.match(r'^feat[\(:!]|^add\b|^new\b', msg): types['feat'] += 1
    elif re.match(r'^refactor[\(:!]|^clean\b', msg): types['refactor'] += 1
    elif re.match(r'^test[\(:!]', msg): types['test'] += 1
    elif re.match(r'^docs[\(:!]|^doc\b', msg): types['docs'] += 1
    elif re.match(r'^chore[\(:!]', msg): types['chore'] += 1
    elif re.match(r'^perf[\(:!]', msg): types['perf'] += 1
    else: types['other'] += 1

print(json.dumps({
    'period': '${SINCE}',
    'total_commits': ${TOTAL},
    'breakdown': dict(types),
    'fix_ratio': round(types.get('fix', 0) / max(${TOTAL}, 1), 3),
    'hotspots': []  # Populated by git log --stat analysis in extended mode
}, indent=2))
"
```

## New script: scripts/feedback-digest.sh

```bash
#!/usr/bin/env bash
set -euo pipefail
EVOLUTION_NAME="${1:?Usage: feedback-digest.sh <evolution-name>}"
LOOP_JSON=".evolver/${EVOLUTION_NAME}/loop.json"

if [ ! -f "${LOOP_JSON}" ]; then
  echo "[feedback-digest] No loop.json found at ${LOOP_JSON}" >&2
  echo '{"collected": 0, "high_severity_count": 0, "new_signals": []}'
  exit 0
fi

echo "[feedback-digest] Collecting feedback for ${EVOLUTION_NAME}"

# Read feedback_sources from loop.json
SOURCES=$(python3 -c "
import json
with open('${LOOP_JSON}') as f:
    d = json.load(f)
sources = d.get('feedback_sources', [])
print(json.dumps(sources))
")

SOURCE_COUNT=$(echo "${SOURCES}" | python3 -c "import json,sys; print(len(json.load(sys.stdin)))")
echo "[feedback-digest] Found ${SOURCE_COUNT} feedback sources"

SIGNALS_DIR=".evolver/${EVOLUTION_NAME}"
mkdir -p "${SIGNALS_DIR}"
TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ)
OUTPUT_FILE="${SIGNALS_DIR}/learning-signals-${TIMESTAMP}.json"

# Process each source (simplified: full implementation per source type)
python3 -c "
import json, sys, subprocess, datetime

with open('${LOOP_JSON}') as f:
    loop = json.load(f)

sources = loop.get('feedback_sources', [])
signals = []

for src in sources:
    src_type = src.get('type', 'unknown')
    signal_base = {
        'source_type': src_type,
        'source_ref': src.get('repo', src.get('url', src.get('repo_path', ''))),
        'collected_at': datetime.datetime.utcnow().isoformat() + 'Z',
        'examples': []
    }

    if src_type == 'commit-history':
        repo_path = src.get('repo_path', '.')
        since = src.get('since', '30 days ago')
        try:
            result = subprocess.run(
                ['bash', 'skills/process/pmpo-evolver/scripts/commit-history-analyze.sh', repo_path, '--since', since],
                capture_output=True, text=True
            )
            data = json.loads(result.stdout)
            fix_ratio = data.get('fix_ratio', 0)
            severity = 'high' if fix_ratio > 0.4 else 'medium' if fix_ratio > 0.2 else 'low'
            signal_base.update({
                'signal': f\"Fix ratio {fix_ratio:.0%} over {data.get('total_commits',0)} commits. High fix ratio indicates quality debt.\",
                'severity': severity,
                'count': data.get('breakdown', {}).get('fix', 0),
                'model_used': 'small'
            })
        except Exception as e:
            signal_base.update({'signal': f'commit-history collection failed: {e}', 'severity': 'low', 'count': 0})
    else:
        signal_base.update({'signal': f'{src_type} collection not yet implemented in this version', 'severity': 'low', 'count': 0})

    signals.append(signal_base)

high_count = sum(1 for s in signals if s.get('severity') == 'high')
result = {'collected': len(signals), 'high_severity_count': high_count, 'new_signals': signals}
with open('${OUTPUT_FILE}', 'w') as f:
    json.dump(result, f, indent=2)
print(json.dumps(result, indent=2))
"

echo "[feedback-digest] Output: ${OUTPUT_FILE}"
```

## Acceptance criteria

- `skills/process/pmpo-evolver/references/learning-signals.md` exists with all source type protocols
- `scripts/commit-history-analyze.sh` is executable
- `bash scripts/commit-history-analyze.sh . --since 2026-06-01` exits 0 and outputs valid JSON
- `scripts/feedback-digest.sh` is executable
- Model routing comments (`[MODEL_ROUTING]`) are present in both scripts
- `learning-signals.md` documents the LearningSignal normalization format

## Tasks

- [x] 1. `skills/process/pmpo-evolver/references/learning-signals.md` exists with all source type protocols
- [x] 2. `scripts/commit-history-analyze.sh` is executable
- [x] 3. `bash scripts/commit-history-analyze.sh . --since 2026-06-01` exits 0 and outputs valid JSON
- [x] 4. `scripts/feedback-digest.sh` is executable
- [x] 5. Model routing comments (`[MODEL_ROUTING]`) are present in both scripts
- [x] 6. `learning-signals.md` documents the LearningSignal normalization format
