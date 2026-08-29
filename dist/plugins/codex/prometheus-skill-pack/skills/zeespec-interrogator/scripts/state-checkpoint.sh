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

SUBJECT_DIR="${STATE_DIR}/subjects/${SUBJECT_NAME}"
STATE_FILE="${SUBJECT_DIR}/state.json"
CHECKPOINT_DIR="${SUBJECT_DIR}/checkpoints"

if [ ! -f "$STATE_FILE" ]; then
  echo "❌ No state found for subject: ${SUBJECT_NAME}" >&2
  exit 1
fi

mkdir -p "$CHECKPOINT_DIR"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
KBD_ROOT="${KBD_ORCHESTRATOR_ROOT:-$(cd "$SCRIPT_DIR/../../kbd-process-orchestrator" 2>/dev/null && pwd -P || true)}"
if [ -n "$KBD_ROOT" ] && [ -f "$KBD_ROOT/shared/lib/bottleneck-guard.sh" ]; then
  # shellcheck source=/dev/null
  . "$KBD_ROOT/shared/lib/bottleneck-guard.sh"
fi

guard_enabled=0
case "$PHASE" in
  interrogate|score|manifest)
    if command -v kbd_bottleneck_active >/dev/null 2>&1 && kbd_bottleneck_active; then
      guard_enabled=1
      kbd_bottleneck_evaluate zeespec before "$PHASE" 1 >/dev/null
      guard_output="$(kbd_bottleneck_evaluate zeespec before "$PHASE" 0)"
      kbd_bottleneck_print_signal "$guard_output" >&2
    fi
    ;;
esac

export SUBJECT_NAME PHASE EVENT_TYPE STATE_FILE CHECKPOINT_DIR
CHECKPOINT_ID="$(python3 <<'PY'
import datetime
import hashlib
import json
import os
import pathlib
import tempfile

state_path = pathlib.Path(os.environ["STATE_FILE"])
checkpoint_dir = pathlib.Path(os.environ["CHECKPOINT_DIR"])
phase = os.environ["PHASE"]
event_type = os.environ["EVENT_TYPE"]

raw = state_path.read_bytes()
state = json.loads(raw)
now = datetime.datetime.now(datetime.timezone.utc)
timestamp = now.isoformat(timespec="microseconds").replace("+00:00", "Z")
digest = hashlib.sha256(raw).hexdigest()
checkpoint_id = f"{phase}-{now:%Y%m%dT%H%M%S%fZ}-{digest[:12]}"
checkpoint_path = checkpoint_dir / f"{checkpoint_id}.json"

checkpoint = {
    "checkpoint_id": checkpoint_id,
    "event_type": event_type,
    "phase": phase,
    "timestamp": timestamp,
    "source_state_sha256": digest,
    "state_snapshot": state,
}
state.setdefault("checkpoints", []).append({
    "checkpoint_id": checkpoint_id,
    "phase": phase,
    "event_type": event_type,
    "timestamp": timestamp,
    "source_state_sha256": digest,
})
state["updated_at"] = timestamp

def write_atomic(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        mode="w", encoding="utf-8", dir=path.parent, prefix=f".{path.name}.", delete=False
    ) as handle:
        json.dump(value, handle, indent=2)
        handle.write("\n")
        handle.flush()
        os.fsync(handle.fileno())
        temporary = pathlib.Path(handle.name)
    os.replace(temporary, path)
    directory_fd = os.open(path.parent, os.O_RDONLY)
    try:
        os.fsync(directory_fd)
    finally:
        os.close(directory_fd)

write_atomic(checkpoint_path, checkpoint)
write_atomic(state_path, state)
print(checkpoint_id)
PY
)"

if [ "$guard_enabled" = "1" ]; then
  kbd_bottleneck_evaluate zeespec after "$PHASE" 1 >/dev/null
  guard_output="$(kbd_bottleneck_evaluate zeespec after "$PHASE" 0)"
  kbd_bottleneck_print_signal "$guard_output" >&2
fi

printf '%s\n' "$CHECKPOINT_ID"

exit 0
