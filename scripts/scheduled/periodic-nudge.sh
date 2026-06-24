#!/usr/bin/env bash
# periodic-nudge.sh — periodic self-learning nudge for the Prometheus loop framework.
#
# Runs every 4 hours via launchd (ai.prometheus.prometheus-nudge).
# Pushes a lightweight heartbeat to surreal-memory so active KBD sessions
# can detect elapsed time and prompt for cross-session continuity.
#
# Does nothing when surreal-memory is unreachable (graceful degradation).

set -euo pipefail

SKILL_PACK_ROOT="${PROMETHEUS_SKILL_PACK_ROOT:-$(cd "$(dirname "$0")/../.." && pwd)}"
SURREAL_URL="${SURREAL_MEMORY_URL:-http://localhost:23001}"
LOG_FILE="${HOME}/.prometheus/logs/prometheus-nudge.log"
TIMESTAMP="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

log() { echo "[$TIMESTAMP] $*" >> "$LOG_FILE" 2>/dev/null || true; }

# Health check — bail silently if surreal-memory is down
if ! curl -sf "${SURREAL_URL}/health" -o /dev/null --connect-timeout 2 --max-time 4; then
    log "SKIP surreal-memory unreachable at ${SURREAL_URL}"
    exit 0
fi

# Emit nudge heartbeat via surreal-memory REST
PAYLOAD="$(cat <<JSON
{
  "content": "Prometheus periodic nudge — ${TIMESTAMP}. Active KBD phases should check cross-session continuity and self-learning opportunities.",
  "user_id": "prometheus-skill-pack",
  "metadata": {
    "type": "nudge",
    "source": "periodic-nudge",
    "timestamp": "${TIMESTAMP}",
    "skill_pack_root": "${SKILL_PACK_ROOT}"
  }
}
JSON
)"

HTTP_CODE="$(curl -sf -w '%{http_code}' -o /dev/null \
    -X POST "${SURREAL_URL}/api/v1/memory" \
    -H 'Content-Type: application/json' \
    -d "$PAYLOAD" \
    --connect-timeout 4 --max-time 10 2>/dev/null || echo "000")"

if [ "$HTTP_CODE" = "201" ] || [ "$HTTP_CODE" = "200" ]; then
    log "OK nudge written (HTTP $HTTP_CODE)"
else
    log "WARN nudge write got HTTP $HTTP_CODE"
fi

# Check waypoint — if a KBD phase is active, append reminder to position-reminder.txt
WAYPOINT="${SKILL_PACK_ROOT}/.kbd-orchestrator/current-waypoint.json"
REMINDER="${SKILL_PACK_ROOT}/.kbd-orchestrator/position-reminder.txt"
if [ -f "$WAYPOINT" ] && command -v jq >/dev/null 2>&1; then
    PHASE="$(jq -r '.phase // empty' "$WAYPOINT" 2>/dev/null)"
    NEXT_CMD="$(jq -r '.exactNextCommand // .exact_next_command // empty' "$WAYPOINT" 2>/dev/null)"
    if [ -n "$PHASE" ] && [ -n "$NEXT_CMD" ]; then
        cat > "$REMINDER" <<REMINDER
POSITION REMINDER — read this as your FIRST tool call every turn
Phase: ${PHASE}
Next command: ${NEXT_CMD}
Nudge: Periodic nudge at ${TIMESTAMP} — cross-session continuity check recommended.
REMINDER
        log "Updated position-reminder.txt for phase=${PHASE} next=${NEXT_CMD}"
    fi
fi
