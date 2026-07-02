---
id: change-evolver-004
title: "Competitor tracking: registry + parity matrix + changelog ingestion"
phase: pmpo-evolver
gaps: [G-03, G-04, G-05]
priority: "HIGH — whitespace #1 in competitive landscape; no current harness has this"
goals: G1
agent: claude-code
status: done
scope:
  - skills/process/pmpo-evolver/references/competitive-analysis.md
  - scripts/competitor-registry-init.sh
  - scripts/changelog-fetch.sh
---

# change-evolver-004 — Competitor tracking: registry + parity matrix + changelog ingestion

## Problem

No competitor tracking infrastructure exists. The competitive perspective (G-03) has no registry format, no parity matrix format, and no tooling to ingest competitor changelogs (G-04, G-05). Assessing competitive gaps requires comparing our feature set against competitors, which is impossible without structured data.

## Solution

Create `competitive-analysis.md` as the authoritative reference for the competitive perspective protocol. Create two executable scripts: `competitor-registry-init.sh` for one-time registry initialization, and `changelog-fetch.sh` for recurring changelog ingestion. Define the competitor registry and parity matrix JSON formats.

## New file: references/competitive-analysis.md

**Contents:**

### Competitor Registry Format
`.evolver/<name>/competitor-registry.json`:
```json
{
  "evolution_name": "string",
  "last_updated": "ISO8601",
  "competitors": [{
    "id": "string (kebab-case)",
    "name": "string (display name)",
    "url": "string",
    "github_repo": "string (owner/repo, optional)",
    "category": "string (direct | adjacent | aspirational)",
    "last_scanned": "ISO8601",
    "last_changelog_tag": "string",
    "feature_claims": ["string"],
    "notes": "string"
  }]
}
```

### Parity Matrix Format
`.evolver/<name>/parity-matrix.json`:
```json
{
  "evolution_name": "string",
  "last_updated": "ISO8601",
  "features": [{
    "id": "string",
    "name": "string",
    "category": "string",
    "our_status": "has | missing | partial | better | n/a",
    "competitors": {
      "<competitor-id>": "has | missing | partial | better | n/a"
    },
    "priority": "high | medium | low",
    "effort_estimate": "xs | s | m | l | xl",
    "last_updated": "ISO8601",
    "source_signal": "string (which changelog or scan identified this)"
  }]
}
```

### Changelog Ingestion Protocol
1. `changelog-fetch.sh <owner/repo> [--since-tag <tag>]` fetches releases via `gh api`
2. Passes release notes through liter-llm `complete(model=medium)` with extraction prompt
3. Extraction prompt outputs: `{repo, from_tag, to_tag, features_added[], breaking_changes[], deprecations[]}`
4. Script stores result in `.evolver/<name>/changelogs/<competitor-id>-<timestamp>.json`
5. The parity matrix update step reads these changelogs and compares against our feature set via liter-llm `complete(model=frontier)` (judgment required for equivalence)

### Competitive scan cadence
- `staleness_ttl_minutes: 1440` (once per day) for most competitors
- `category: direct` competitors: manual trigger on major releases

### Model routing
- Changelog feature extraction → `[MODEL_ROUTING] phase=evolver-competitive-extract class=medium`
- Parity matrix update → `[MODEL_ROUTING] phase=evolver-competitive-parity class=frontier`

## New script: scripts/competitor-registry-init.sh

```bash
#!/usr/bin/env bash
set -euo pipefail
EVOLUTION_NAME="${1:?Usage: competitor-registry-init.sh <evolution-name>}"
REGISTRY_DIR=".evolver/${EVOLUTION_NAME}"
mkdir -p "${REGISTRY_DIR}"
REGISTRY_PATH="${REGISTRY_DIR}/competitor-registry.json"

if [ -f "${REGISTRY_PATH}" ]; then
  echo "Registry already exists at ${REGISTRY_PATH}"
  exit 0
fi

echo "Initializing competitor registry for: ${EVOLUTION_NAME}"
echo "Enter competitor details (Ctrl+D when done):"
# Uses pmpo-elicit pattern: writes stub and prompts operator to fill in manually
python3 -c "
import json, sys
stub = {
  'evolution_name': '${EVOLUTION_NAME}',
  'last_updated': __import__('datetime').datetime.utcnow().isoformat() + 'Z',
  'competitors': []
}
with open('${REGISTRY_PATH}', 'w') as f:
    json.dump(stub, f, indent=2)
print('Stub written to ${REGISTRY_PATH}')
print('Edit this file to add competitor entries, then run competitor-scan.')
"
```

## New script: scripts/changelog-fetch.sh

Fetches and processes a competitor's changelog using GitHub Releases API or CHANGELOG.md file.

```bash
#!/usr/bin/env bash
set -euo pipefail
REPO="${1:?Usage: changelog-fetch.sh <owner/repo> [--since-tag <tag>] [--evolution-name <name>]}"
SINCE_TAG=""
EVOLUTION_NAME="default"

while [[ $# -gt 1 ]]; do
  case "$2" in
    --since-tag) SINCE_TAG="$3"; shift 2 ;;
    --evolution-name) EVOLUTION_NAME="$3"; shift 2 ;;
    *) shift ;;
  esac
done

CHANGELOG_DIR=".evolver/${EVOLUTION_NAME}/changelogs"
mkdir -p "${CHANGELOG_DIR}"

COMPETITOR_ID=$(echo "${REPO}" | tr '/' '-')
TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ)
OUTPUT_FILE="${CHANGELOG_DIR}/${COMPETITOR_ID}-${TIMESTAMP}.json"

echo "[changelog-fetch] Fetching releases for ${REPO} since tag: ${SINCE_TAG:-<all>}"

# Fetch via gh API
if [ -n "${SINCE_TAG}" ]; then
  RELEASES=$(gh api "repos/${REPO}/releases" --paginate \
    --jq "[.[] | select(.tag_name > \"${SINCE_TAG}\")]")
else
  RELEASES=$(gh api "repos/${REPO}/releases" --paginate \
    --jq ".[0:10]")  # Last 10 releases
fi

RELEASE_COUNT=$(echo "${RELEASES}" | python3 -c "import json,sys; print(len(json.load(sys.stdin)))")
echo "[changelog-fetch] Found ${RELEASE_COUNT} releases to process"

# Pass to liter-llm for structured extraction (model=medium)
# [MODEL_ROUTING] phase=evolver-changelog-extract class=medium
echo "[changelog-fetch] Extracting features via model=medium"
EXTRACTED=$(echo "${RELEASES}" | python3 -c "
import json, sys
releases = json.load(sys.stdin)
notes = '\n\n---\n\n'.join([
    f\"## {r['tag_name']} ({r['published_at'][:10]})\n{r.get('body','')[:2000]}\"
    for r in releases
])
print(notes)
" | liter-llm complete --model medium \
    --system "Extract features, breaking changes, and deprecations from these release notes. Output JSON: {from_tag, to_tag, features_added: [string], breaking_changes: [string], deprecations: [string]}" \
    2>/dev/null || echo '{"error": "liter-llm not available", "features_added": [], "breaking_changes": [], "deprecations": []}')

python3 -c "
import json, sys
data = json.loads('${EXTRACTED}'.replace(\"'\", '\"') if '${EXTRACTED}'.startswith(\"'\") else '${EXTRACTED}')
result = {
    'repo': '${REPO}',
    'fetched_at': '${TIMESTAMP}',
    'since_tag': '${SINCE_TAG}',
    'release_count': ${RELEASE_COUNT},
    **data
}
with open('${OUTPUT_FILE}', 'w') as f:
    json.dump(result, f, indent=2)
print(json.dumps(result, indent=2))
"
echo "[changelog-fetch] Output: ${OUTPUT_FILE}"
```

## Acceptance criteria

- `skills/process/pmpo-evolver/references/competitive-analysis.md` exists with registry and parity matrix formats
- `scripts/competitor-registry-init.sh` is executable and creates a valid stub JSON
- `scripts/changelog-fetch.sh` is executable
- `bash scripts/changelog-fetch.sh anthropics/anthropic-sdk-python --evolution-name test` exits 0 and produces valid JSON (or graceful error when liter-llm absent)
- Model routing directives are present in the scripts as comments

## Tasks

- [x] 1. `skills/process/pmpo-evolver/references/competitive-analysis.md` exists with registry and parity matrix formats
- [x] 2. `scripts/competitor-registry-init.sh` is executable and creates a valid stub JSON
- [x] 3. `scripts/changelog-fetch.sh` is executable
- [x] 4. `bash scripts/changelog-fetch.sh anthropics/anthropic-sdk-python --evolution-name test` exits 0 and produces valid JSON (or graceful error when liter-llm absent)
- [x] 5. Model routing directives are present in the scripts as comments
