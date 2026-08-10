#!/usr/bin/env bash
# workflow-dispatch.sh — Dispatch workflow triggers for a ZeeSpec lifecycle event
# Usage: workflow-dispatch.sh <subject_name> <event_type> [phase]
# Triggers are fire-and-forget — failures never halt interrogation.
# Exit 0 = always

set -euo pipefail

SUBJECT_NAME="${1:?Usage: workflow-dispatch.sh <subject_name> <event_type> [phase]}"
EVENT_TYPE="${2:?Usage: workflow-dispatch.sh <subject_name> <event_type> [phase]}"
PHASE="${3:-}"
STATE_DIR="${ZEESPEC_STATE_DIR:-.zeespec}"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

SUBJECT_DIR="${STATE_DIR}/subjects/${SUBJECT_NAME}"
STATE_FILE="${SUBJECT_DIR}/state.json"

if [ ! -f "$STATE_FILE" ]; then
  exit 0
fi

export SUBJECT_NAME EVENT_TYPE PHASE STATE_FILE TIMESTAMP

python3 << 'DISPATCH_SCRIPT'
import json, subprocess, sys, os

subject_name = os.environ.get("SUBJECT_NAME", "")
event_type = os.environ.get("EVENT_TYPE", "")
phase = os.environ.get("PHASE", "")
state_file = os.environ.get("STATE_FILE", "")
timestamp = os.environ.get("TIMESTAMP", "")

try:
    state = json.load(open(state_file))
except:
    sys.exit(0)

triggers = list(state.get("workflow_triggers", []))

if not triggers:
    sys.exit(0)

context = {
    "subject_name": subject_name,
    "interrogation_id": state.get("interrogation_id", ""),
    "caller": state.get("caller", "standalone"),
    "phase": phase,
    "status": state.get("status", "running"),
    "timestamp": timestamp,
}

def substitute_vars(text, ctx):
    if not isinstance(text, str):
        return text
    result = text
    for key, val in ctx.items():
        result = result.replace(f"${{{key}}}", str(val))
    return result

dispatched = 0
for trigger in triggers:
    if trigger.get("event") != f"on_{event_type}" and trigger.get("event") != event_type:
        continue

    action = trigger.get("action", {})
    action_type = action.get("type", "")
    target = substitute_vars(action.get("target", ""), context)
    timeout = action.get("timeout", 30)
    trigger_id = trigger.get("id", "unnamed")

    try:
        if action_type == "command":
            result = subprocess.run(
                target, shell=True, capture_output=True, text=True, timeout=timeout
            )
            status = "✅" if result.returncode == 0 else f"❌ exit={result.returncode}"
            print(f"  {status} trigger '{trigger_id}': {target}")
        elif action_type == "mcp_tool":
            print(f"  📋 trigger '{trigger_id}': MCP tool '{target}' queued (agent dispatch)")
        dispatched += 1
    except subprocess.TimeoutExpired:
        print(f"  ⏱️ trigger '{trigger_id}': timed out after {timeout}s")
    except Exception as e:
        print(f"  ❌ trigger '{trigger_id}': {e}")

if dispatched > 0:
    print(f"\n📨 Dispatched {dispatched} trigger(s) for event '{event_type}'")

DISPATCH_SCRIPT

exit 0
