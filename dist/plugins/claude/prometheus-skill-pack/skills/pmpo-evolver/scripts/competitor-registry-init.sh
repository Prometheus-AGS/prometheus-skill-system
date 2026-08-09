#!/usr/bin/env bash
# competitor-registry-init.sh — Initialize a competitor registry stub for an evolution run
# Usage: competitor-registry-init.sh <evolution-name>
set -euo pipefail

EVOLUTION_NAME="${1:?Usage: competitor-registry-init.sh <evolution-name>}"
REGISTRY_DIR=".evolver/${EVOLUTION_NAME}"
REGISTRY_PATH="${REGISTRY_DIR}/competitor-registry.json"

mkdir -p "${REGISTRY_DIR}"

if [ -f "${REGISTRY_PATH}" ]; then
  echo "[competitor-registry-init] Registry already exists at ${REGISTRY_PATH}"
  echo "[competitor-registry-init] Edit the file to update competitor entries"
  exit 0
fi

echo "[competitor-registry-init] Creating registry stub for: ${EVOLUTION_NAME}"

python3 -c "
import json
from datetime import datetime, timezone

stub = {
  'evolution_name': '${EVOLUTION_NAME}',
  'last_updated': datetime.now(timezone.utc).isoformat().replace('+00:00', 'Z'),
  'competitors': [
    {
      'id': 'example-competitor',
      'name': 'Example Competitor',
      'url': 'https://example.com',
      'github_repo': 'owner/repo',
      'category': 'direct',
      'last_scanned': None,
      'last_changelog_tag': None,
      'feature_claims': [],
      'notes': 'Replace this with a real competitor entry'
    }
  ]
}
with open('${REGISTRY_PATH}', 'w') as f:
    json.dump(stub, f, indent=2)
print(json.dumps(stub, indent=2))
"

echo ""
echo "[competitor-registry-init] Stub written to: ${REGISTRY_PATH}"
echo "[competitor-registry-init] Edit this file to add real competitor entries, then run:"
echo "  bash scripts/changelog-fetch.sh <owner/repo> --evolution-name ${EVOLUTION_NAME}"
