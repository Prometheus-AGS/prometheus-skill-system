---
id: change-slli-009-periodic-nudge
title: Periodic nudge script (KB enrichment + stall detection)
phase: self-learning-loop-integration
gaps: [NUDGE-1]
priority: 9 of 10
agent: claude-code
depends_on: [change-slli-002, change-slli-004]
status: done
scope:
  - scripts/scheduled/periodic-nudge.sh
  - launchd/dev.prometheusags.prometheus-nudge.plist
---

# change-slli-009-periodic-nudge — Periodic nudge script (KB enrichment + stall detection)

## Summary

Create a scheduled periodic nudge script that enriches the knowledge base between sessions, surfaces stall conditions, and aligns with the Hermes-style "periodic nudge" pattern (the background thread that wakes every N iterations to ingest new context).

## Files Created

### `scripts/scheduled/periodic-nudge.sh`

```bash
#!/usr/bin/env bash
# Runs every 4 hours via launchd
# Enriches pk KB from recent changes, checks for loop stalls

set -euo pipefail

NUDGE_LOG="${HOME}/.prometheus/nudge-log"
mkdir -p "$NUDGE_LOG"

LOG_FILE="$NUDGE_LOG/$(date +%Y-%m-%d).txt"

echo "=== Nudge run: $(date -u +%Y-%m-%dT%H:%M:%SZ) ===" >> "$LOG_FILE"

# 1. Ingest recent git changes into pk
if command -v pk >/dev/null 2>&1 && command -v git >/dev/null 2>&1; then
  RECENT_FILES=$(git log --since='24 hours ago' --name-only --pretty=format: 2>/dev/null | sort -u | head -20)
  if [[ -n "$RECENT_FILES" ]]; then
    echo "Ingesting recent changes: $(echo "$RECENT_FILES" | wc -l | tr -d ' ') files" >> "$LOG_FILE"
    echo "$RECENT_FILES" | xargs -I{} cat {} 2>/dev/null | pk ingest 2>/dev/null || true
  fi
fi

# 2. Semantic memory search for active loop/evolution context
ACTIVE_EVOLUTION=$(cat .evolver/active-evolution 2>/dev/null || echo "")
if [[ -n "$ACTIVE_EVOLUTION" ]]; then
  curl -sf --max-time 5 \
    -X POST http://localhost:23001/api/v1/memory/search \
    -H "Content-Type: application/json" \
    -d "{\"query\": \"$ACTIVE_EVOLUTION progress\", \"user_id\": \"prometheus-skill-pack\", \"limit\": 5}" \
    >> "$LOG_FILE" 2>/dev/null || true
fi

# 3. Stall detection — check loop.json for max_no_progress_ticks breach
for LOOP_JSON in .kbd-orchestrator/loops/*/loop.json; do
  [[ -f "$LOOP_JSON" ]] || continue
  STALL=$(jq -r 'if .no_progress_ticks >= .termination.max_no_progress_ticks then "STALL" else "ok" end' "$LOOP_JSON" 2>/dev/null)
  if [[ "$STALL" == "STALL" ]]; then
    LOOP_NAME=$(jq -r '.name' "$LOOP_JSON")
    echo "STALL DETECTED: loop $LOOP_NAME" >> "$LOG_FILE"
    # Write to watched file for notification
    echo "Loop $LOOP_NAME stalled at $(date)" > "${HOME}/.prometheus/stall-alert.txt"
  fi
done

echo "Nudge complete" >> "$LOG_FILE"
```

### `launchd/dev.prometheusags.prometheus-nudge.plist`

Runs `scripts/scheduled/periodic-nudge.sh` every 4 hours via `StartInterval: 14400`.

## Acceptance Criteria

- `launchctl list | grep prometheus-nudge` shows the agent (non-zero PID)
- `~/.prometheus/nudge-log/` gains an entry within 4 hours of first install
- `pk search <topic>` returns richer results after nudge has run against recent commits
- Stall detection writes to `~/.prometheus/stall-alert.txt` when `no_progress_ticks >= max_no_progress_ticks`
- Script exits 0 even when no recent git commits exist

## Tasks

- [x] 1. `launchctl list | grep prometheus-nudge` shows the agent (non-zero PID)
- [x] 2. `~/.prometheus/nudge-log/` gains an entry within 4 hours of first install
- [x] 3. `pk search <topic>` returns richer results after nudge has run against recent commits
- [x] 4. Stall detection writes to `~/.prometheus/stall-alert.txt` when `no_progress_ticks >= max_no_progress_ticks`
- [x] 5. Script exits 0 even when no recent git commits exist
