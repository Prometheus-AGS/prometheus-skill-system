#!/usr/bin/env bash
# feedback-digest.sh — Collect all loop.json feedback sources and normalize to LearningSignal[]
# Usage: feedback-digest.sh <evolution-name> [--loop-json <path>]
# Run as an isolated subprocess — never inline in the evolver session.
set -euo pipefail

EVOLUTION_NAME="${1:?Usage: feedback-digest.sh <evolution-name> [--loop-json <path>]}"
LOOP_JSON_OVERRIDE=""

shift
while [[ $# -gt 0 ]]; do
  case "$1" in
    --loop-json) LOOP_JSON_OVERRIDE="${2:-}"; shift 2 ;;
    *) shift ;;
  esac
done

# Locate loop.json
if [ -n "${LOOP_JSON_OVERRIDE}" ]; then
  LOOP_JSON="${LOOP_JSON_OVERRIDE}"
else
  LOOP_JSON=".evolver/${EVOLUTION_NAME}/loop.json"
  if [ ! -f "${LOOP_JSON}" ]; then
    LOOP_JSON=".kbd-orchestrator/loops/${EVOLUTION_NAME}/loop.json"
  fi
fi

EVOLVER_DIR=".evolver/${EVOLUTION_NAME}"
mkdir -p "${EVOLVER_DIR}"

TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ 2>/dev/null || \
  python3 -c "from datetime import datetime,timezone; print(datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ'))")
OUTPUT_FILE="${EVOLVER_DIR}/learning-signals-${TIMESTAMP}.json"

if [ ! -f "${LOOP_JSON}" ]; then
  echo "[feedback-digest] No loop.json found — returning empty digest" >&2
  printf '{"collected": 0, "high_severity_count": 0, "new_signals": [], "note": "No loop.json at %s"}\n' "${LOOP_JSON}"
  exit 0
fi

echo "[feedback-digest] Reading feedback sources from: ${LOOP_JSON}" >&2

SOURCE_COUNT=$(python3 -c "
import json
with open('${LOOP_JSON}') as f:
    d = json.load(f)
sources = d.get('feedback_sources', [])
print(len(sources))
")

echo "[feedback-digest] Found ${SOURCE_COUNT} feedback sources" >&2

python3 -c "
import json, sys, subprocess, os
from datetime import datetime, timezone

with open('${LOOP_JSON}') as f:
    loop = json.load(f)

sources = loop.get('feedback_sources', [])
signals = []
script_dir = os.path.join(os.path.dirname(os.path.abspath('${LOOP_JSON}')), '..', '..', 'skills', 'process', 'pmpo-evolver', 'scripts')

for src in sources:
    src_type = src.get('type', 'unknown')
    collected_at = datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z')
    signal_base = {
        'source_type': src_type,
        'source_ref': src.get('repo', src.get('url', src.get('repo_path', src.get('registry_path', '')))),
        'collected_at': collected_at,
        'examples': [],
        'model_used': 'none'
    }

    if src_type == 'commit-history':
        repo_path = src.get('repo_path', '.')
        since = src.get('since', '30 days ago')
        analyze_script = os.path.join(script_dir, 'commit-history-analyze.sh')
        try:
            result = subprocess.run(
                ['bash', analyze_script, repo_path, '--since', since],
                capture_output=True, text=True, timeout=60
            )
            data = json.loads(result.stdout)
            fix_ratio = data.get('fix_ratio', 0)
            total = data.get('total_commits', 0)
            if fix_ratio > 0.4: severity = 'high'
            elif fix_ratio > 0.25: severity = 'medium'
            else: severity = 'low'
            signal_base.update({
                'signal': f'Fix ratio {fix_ratio:.0%} over {total} commits since {since}. ' + (
                    'High fix ratio indicates quality debt.' if fix_ratio > 0.25 else 'Healthy commit distribution.'
                ),
                'severity': severity,
                'count': data.get('breakdown', {}).get('fix', 0),
                'model_used': 'small'
            })
        except Exception as e:
            signal_base.update({'signal': f'commit-history collection failed: {e}', 'severity': 'low', 'count': 0})

    elif src_type == 'gh-issues':
        repo = src.get('repo', '')
        labels = src.get('labels', [])
        label_filter = ' '.join([f'--label {l}' for l in labels]) if labels else ''
        try:
            label_args = []
            for l in labels:
                label_args += ['--label', l]
            result = subprocess.run(
                ['gh', 'api', f'repos/{repo}/issues', '--jq', '[.[] | select(.state==\"open\")] | length'],
                capture_output=True, text=True, timeout=30
            )
            count = int(result.stdout.strip() or '0')
            severity = 'high' if count > 50 else 'medium' if count > 20 else 'low'
            signal_base.update({
                'signal': f'{count} open issues in {repo}' + (f' with labels {labels}' if labels else '') + '.',
                'severity': severity,
                'count': count,
                'model_used': 'small'
            })
        except Exception as e:
            signal_base.update({'signal': f'gh-issues collection failed: {e}', 'severity': 'low', 'count': 0})

    else:
        signal_base.update({
            'signal': f'{src_type} collection not yet implemented in digest script; add handler in feedback-digest.sh',
            'severity': 'low',
            'count': 0
        })

    signals.append(signal_base)

high_count = sum(1 for s in signals if s.get('severity') == 'high')
result = {
    'collected': len(signals),
    'high_severity_count': high_count,
    'new_signals': signals
}

with open('${OUTPUT_FILE}', 'w') as f:
    json.dump(result, f, indent=2)

print(json.dumps(result, indent=2))
" 2>&1

echo "[feedback-digest] Output: ${OUTPUT_FILE}" >&2
