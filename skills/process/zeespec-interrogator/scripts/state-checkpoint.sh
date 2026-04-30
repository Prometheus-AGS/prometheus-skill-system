#!/usr/bin/env bash
# state-checkpoint.sh — Filesystem provider: snapshot mid-session interrogation state
# Usage: state-checkpoint.sh <subject_name> [phase] [event_type]
# Output: checkpoint_id to stdout
# Exit 0 = OK, Exit 1 = error

set -euo pipefail

SUBJECT_NAME="${1:?Usage: state-checkpoint.sh <subject_name> [phase] [event_type]}"
PHASE="${2:-unknown}"
EVENT_TYPE="${3:-checkpoint}"
STATE_DIR="${ZEESPEC_STATE_DIR:-.zeespec}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
CHECKPOINT_ID="${PHASE}-$(date -u +"%Y%m%dT%H%M%S")"

SUBJECT_DIR="${STATE_DIR}/subjects/${SUBJECT_NAME}"
STATE_FILE="${SUBJECT_DIR}/state.json"
CHECKPOINT_DIR="${SUBJECT_DIR}/checkpoints"
CHECKPOINT_FILE="${CHECKPOINT_DIR}/${CHECKPOINT_ID}.json"

if [ ! -f "$STATE_FILE" ]; then
  echo "❌ No state found for subject: ${SUBJECT_NAME}" >&2
  exit 1
fi

mkdir -p "$CHECKPOINT_DIR"

python3 -c "
import json

state = json.load(open('$STATE_FILE'))

checkpoint = {
    'checkpoint_id': '$CHECKPOINT_ID',
    'event_type': '$EVENT_TYPE',
    'phase': '$PHASE',
    'timestamp': '$TIMESTAMP',
    'state_snapshot': state
}

json.dump(checkpoint, open('$CHECKPOINT_FILE', 'w'), indent=2)

checkpoints = state.get('checkpoints', [])
checkpoints.append({
    'checkpoint_id': '$CHECKPOINT_ID',
    'phase': '$PHASE',
    'event_type': '$EVENT_TYPE',
    'timestamp': '$TIMESTAMP'
})
state['checkpoints'] = checkpoints
state['updated_at'] = '$TIMESTAMP'
json.dump(state, open('$STATE_FILE', 'w'), indent=2)

print('$CHECKPOINT_ID')
"

exit 0
