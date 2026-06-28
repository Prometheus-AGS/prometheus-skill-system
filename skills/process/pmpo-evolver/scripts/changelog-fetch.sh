#!/usr/bin/env bash
# changelog-fetch.sh — Fetch and extract features from a competitor's GitHub releases
# Usage: changelog-fetch.sh <owner/repo> [--since-tag <tag>] [--evolution-name <name>]
# [MODEL_ROUTING] phase=evolver-changelog-extract class=medium
set -euo pipefail

REPO="${1:?Usage: changelog-fetch.sh <owner/repo> [--since-tag <tag>] [--evolution-name <name>]}"
SINCE_TAG=""
EVOLUTION_NAME="default"

shift
while [[ $# -gt 0 ]]; do
  case "$1" in
    --since-tag) SINCE_TAG="${2:-}"; shift 2 ;;
    --evolution-name) EVOLUTION_NAME="${2:-}"; shift 2 ;;
    *) shift ;;
  esac
done

CHANGELOG_DIR=".evolver/${EVOLUTION_NAME}/changelogs"
mkdir -p "${CHANGELOG_DIR}"

COMPETITOR_ID=$(echo "${REPO}" | tr '/' '-')
TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ 2>/dev/null || python3 -c "from datetime import datetime,timezone; print(datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ'))")
OUTPUT_FILE="${CHANGELOG_DIR}/${COMPETITOR_ID}-${TIMESTAMP}.json"

echo "[changelog-fetch] Fetching releases for ${REPO} since tag: ${SINCE_TAG:-<all recent>}"

# Check gh is available
if ! command -v gh > /dev/null 2>&1; then
  echo "[changelog-fetch] ERROR: gh CLI not available" >&2
  echo "{\"repo\": \"${REPO}\", \"error\": \"gh CLI not available\", \"features_added\": [], \"breaking_changes\": [], \"deprecations\": []}" > "${OUTPUT_FILE}"
  cat "${OUTPUT_FILE}"
  exit 0
fi

# Fetch releases
if [ -n "${SINCE_TAG}" ]; then
  RELEASES_JSON=$(gh api "repos/${REPO}/releases" --paginate 2>/dev/null | \
    python3 -c "
import json, sys
releases = json.load(sys.stdin)
tag = '${SINCE_TAG}'
filtered = [r for r in releases if r.get('tag_name', '') > tag]
print(json.dumps(filtered[:10]))
" 2>/dev/null || echo "[]")
else
  RELEASES_JSON=$(gh api "repos/${REPO}/releases" 2>/dev/null | \
    python3 -c "import json, sys; print(json.dumps(json.load(sys.stdin)[:5]))" 2>/dev/null || echo "[]")
fi

RELEASE_COUNT=$(echo "${RELEASES_JSON}" | python3 -c "import json,sys; print(len(json.load(sys.stdin)))" 2>/dev/null || echo "0")
echo "[changelog-fetch] Found ${RELEASE_COUNT} releases to process"

if [ "${RELEASE_COUNT}" -eq 0 ]; then
  python3 -c "
import json
from datetime import datetime, timezone
result = {
  'repo': '${REPO}',
  'fetched_at': datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z'),
  'since_tag': '${SINCE_TAG}',
  'release_count': 0,
  'features_added': [],
  'breaking_changes': [],
  'deprecations': []
}
with open('${OUTPUT_FILE}', 'w') as f:
    json.dump(result, f, indent=2)
print(json.dumps(result, indent=2))
"
  exit 0
fi

# Build release notes text for LLM extraction
RELEASE_NOTES=$(echo "${RELEASES_JSON}" | python3 -c "
import json, sys
releases = json.load(sys.stdin)
notes = []
for r in releases:
    tag = r.get('tag_name', 'unknown')
    date = r.get('published_at', '')[:10]
    body = (r.get('body') or '').strip()[:3000]
    notes.append(f'## {tag} ({date})\n{body}')
print('\n\n---\n\n'.join(notes))
" 2>/dev/null || echo "No release notes available")

# [MODEL_ROUTING] phase=evolver-changelog-extract class=medium
echo "[changelog-fetch] Extracting features via model=medium"

EXTRACTED="{}"
if command -v liter-llm > /dev/null 2>&1; then
  EXTRACTED=$(printf "%s" "${RELEASE_NOTES}" | liter-llm complete \
    --model medium \
    --system 'Extract features, breaking changes, and deprecations from these release notes. Output ONLY valid JSON in this exact format: {"features_added": ["string"], "breaking_changes": ["string"], "deprecations": ["string"]}. No markdown, no explanation.' \
    2>/dev/null || echo "{}")
else
  echo "[changelog-fetch] liter-llm not available — returning raw release count only"
fi

# Combine into final output
python3 -c "
import json, sys
from datetime import datetime, timezone

try:
    extracted = json.loads('''${EXTRACTED}''')
except:
    extracted = {'features_added': [], 'breaking_changes': [], 'deprecations': []}

releases_raw = json.loads('''${RELEASES_JSON}''')
tags = [r.get('tag_name', '') for r in releases_raw]
from_tag = '${SINCE_TAG}' or (tags[-1] if tags else '')
to_tag = tags[0] if tags else ''

result = {
  'repo': '${REPO}',
  'fetched_at': datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z'),
  'since_tag': from_tag,
  'to_tag': to_tag,
  'release_count': ${RELEASE_COUNT},
  'features_added': extracted.get('features_added', []),
  'breaking_changes': extracted.get('breaking_changes', []),
  'deprecations': extracted.get('deprecations', [])
}

with open('${OUTPUT_FILE}', 'w') as f:
    json.dump(result, f, indent=2)
print(json.dumps(result, indent=2))
" 2>/dev/null

echo "[changelog-fetch] Output: ${OUTPUT_FILE}"
