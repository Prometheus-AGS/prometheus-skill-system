---
id: change-slli-006-forge-independent-reflect
title: Forge-independent reflect fallback (pk ingest without forge)
phase: self-learning-loop-integration
gaps: [REFLECT-1, REFLECT-2]
priority: 7 of 10
agent: claude-code
status: done
scope:
  - shared/scripts/forge-reflect-on-stop.sh
  - hooks/hooks.json
  - shared/scripts/write-session-summary.sh
---

# change-slli-006-forge-independent-reflect — Forge-independent reflect fallback (pk ingest without forge)

## Summary

Fix the silent no-op in `forge-reflect-on-stop.sh` when forge is absent. On most installs, this script does nothing — the `pk ingest` call after `forge reflect` is never reached. This change adds a direct `pk ingest` fallback path using a session summary written by a new pre-stop helper.

## Files Modified

### `shared/scripts/forge-reflect-on-stop.sh`

Revised flow:
1. **With forge:** `forge reflect` → `pk ingest` (unchanged behavior)
2. **Without forge:** read `~/.prometheus/last-session-summary.txt` → `pk ingest --stdin`
3. Both paths exit 0 on failure (non-blocking hook)

### `hooks/hooks.json`

Stop array: add `write-session-summary.sh` as first Stop hook entry (before forge-reflect-on-stop.sh).

## Files Created

### `shared/scripts/write-session-summary.sh`

```bash
#!/usr/bin/env bash
# Called first in Stop hook chain
# Writes a summary of the session for use by forge-reflect and pk ingest fallback

set -euo pipefail

SUMMARY_DIR="${HOME}/.prometheus"
mkdir -p "$SUMMARY_DIR"

WAYPOINT="$(git rev-parse --show-toplevel 2>/dev/null)/.kbd-orchestrator/current-waypoint.json"
PHASE=$(jq -r '.phase // "unknown"' "$WAYPOINT" 2>/dev/null) || PHASE="unknown"
STAGE=$(jq -r '.stage // "unknown"' "$WAYPOINT" 2>/dev/null) || STAGE="unknown"
LAST=$(jq -r '.last_completed // "none"' "$WAYPOINT" 2>/dev/null) || LAST="none"

cat > "$SUMMARY_DIR/last-session-summary.txt" <<EOF
Session ended: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Phase: $PHASE
Stage: $STAGE
Last completed: $LAST
EOF
```

## Acceptance Criteria

- On a machine without forge: Stop hook calls `pk ingest` with session summary content
- On a machine with forge: `forge reflect` runs as before, followed by `pk ingest`
- `~/.prometheus/last-session-summary.txt` always exists after a session Stop
- Neither script blocks the Stop chain on failure

## Tasks

- [x] 1. On a machine without forge: Stop hook calls `pk ingest` with session summary content
- [x] 2. On a machine with forge: `forge reflect` runs as before, followed by `pk ingest`
- [x] 3. `~/.prometheus/last-session-summary.txt` always exists after a session Stop
- [x] 4. Neither script blocks the Stop chain on failure
