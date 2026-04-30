#!/usr/bin/env bash
# state-init.sh — Filesystem provider: initialize or resume interrogation state
# Usage: state-init.sh <subject_name> [caller]
# Output: JSON state to stdout
# Exit 0 = OK, Exit 1 = error

set -euo pipefail

SUBJECT_NAME="${1:?Usage: state-init.sh <subject_name> [caller]}"
CALLER="${2:-standalone}"
STATE_DIR="${ZEESPEC_STATE_DIR:-.zeespec}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

SUBJECT_DIR="${STATE_DIR}/subjects/${SUBJECT_NAME}"
mkdir -p "${SUBJECT_DIR}/checkpoints"
mkdir -p "${SUBJECT_DIR}/history"

REGISTRY_FILE="${STATE_DIR}/registry.json"
STATE_FILE="${SUBJECT_DIR}/state.json"

if [ ! -f "$REGISTRY_FILE" ]; then
  echo '{"subjects": {}}' > "$REGISTRY_FILE"
fi

if [ -f "$STATE_FILE" ]; then
  EXISTING_STATUS=$(python3 -c "
import json
state = json.load(open('$STATE_FILE'))
print(state.get('status', 'running'))
")

  if [ "$EXISTING_STATUS" = "complete" ] || [ "$EXISTING_STATUS" = "abandoned" ]; then
    ITERATION=$(python3 -c "import json; print(json.load(open('$STATE_FILE')).get('interrogation_id', 'unknown'))")
    cp "$STATE_FILE" "${SUBJECT_DIR}/history/interrogation-${ITERATION}.json"
    python3 -c "
import json, uuid
state = json.load(open('$STATE_FILE'))
state['interrogation_id'] = str(uuid.uuid4())
state['status'] = 'running'
state['phases_completed'] = []
state['updated_at'] = '$TIMESTAMP'
state['prior_interrogation_id'] = state.get('interrogation_id', '')
state.pop('interrogation_record', None)
state.pop('coverage_score', None)
state.pop('manifest_path', None)
json.dump(state, open('$STATE_FILE', 'w'), indent=2)
print(json.dumps(state, indent=2))
"
  else
    python3 -c "
import json
state = json.load(open('$STATE_FILE'))
state['updated_at'] = '$TIMESTAMP'
json.dump(state, open('$STATE_FILE', 'w'), indent=2)
print(json.dumps(state, indent=2))
"
  fi
else
  python3 -c "
import json, uuid
state = {
    'interrogation_id': str(uuid.uuid4()),
    'subject_name': '$SUBJECT_NAME',
    'caller': '$CALLER',
    'started_at': '$TIMESTAMP',
    'updated_at': '$TIMESTAMP',
    'status': 'running',
    'phases_completed': [],
    'dimensions_requested': ['why', 'who', 'when', 'what', 'where', 'how'],
    'coverage_threshold': 0.70,
    'checkpoints': [],
    'workflow_triggers': []
}
json.dump(state, open('$STATE_FILE', 'w'), indent=2)
print(json.dumps(state, indent=2))
"
fi

python3 -c "
import json
registry = json.load(open('$REGISTRY_FILE'))
registry['subjects']['$SUBJECT_NAME'] = {
    'path': '$SUBJECT_DIR',
    'caller': '$CALLER',
    'updated_at': '$TIMESTAMP'
}
json.dump(registry, open('$REGISTRY_FILE', 'w'), indent=2)
" 2>/dev/null

exit 0
