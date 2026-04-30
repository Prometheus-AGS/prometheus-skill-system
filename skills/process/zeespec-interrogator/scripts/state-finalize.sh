#!/usr/bin/env bash
# state-finalize.sh — Archive completed interrogation state to history
# Usage: state-finalize.sh <subject_name>
# Exit 0 = OK, Exit 1 = error

set -euo pipefail

SUBJECT_NAME="${1:?Usage: state-finalize.sh <subject_name>}"
STATE_DIR="${ZEESPEC_STATE_DIR:-.zeespec}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

SUBJECT_DIR="${STATE_DIR}/subjects/${SUBJECT_NAME}"
STATE_FILE="${SUBJECT_DIR}/state.json"

if [ ! -f "$STATE_FILE" ]; then
  echo "❌ No state found for subject: ${SUBJECT_NAME}" >&2
  exit 1
fi

python3 -c "
import json, shutil, os

state = json.load(open('$STATE_FILE'))
interrogation_id = state.get('interrogation_id', 'unknown')

history_dir = '$SUBJECT_DIR/history'
os.makedirs(history_dir, exist_ok=True)

archive_path = f'{history_dir}/{interrogation_id}.json'
shutil.copy2('$STATE_FILE', archive_path)

state['status'] = 'complete'
state['finalized_at'] = '$TIMESTAMP'
state['updated_at'] = '$TIMESTAMP'
json.dump(state, open('$STATE_FILE', 'w'), indent=2)

print(f'✅ Finalized interrogation {interrogation_id} → {archive_path}')
"

exit 0
