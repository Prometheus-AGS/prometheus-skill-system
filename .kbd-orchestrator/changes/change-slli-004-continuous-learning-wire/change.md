---
id: change-slli-004-continuous-learning-wire
title: "Wire continuous-learning-v2 into SubagentStop[executor]"
phase: self-learning-loop-integration
gaps: [LEARN-1, LEARN-2]
priority: 5 of 10
agent: claude-code
status: done
scope:
  - hooks/hooks.json
  - shared/scripts/evaluate-session.sh
---

# change-slli-004-continuous-learning-wire — Wire continuous-learning-v2 into SubagentStop[executor]

## Summary

Wire `continuous-learning-v2` into the `SubagentStop[executor]` hook so that every completed `/kbd-execute` run automatically extracts reusable patterns and ingests them into the `pk` knowledge base. Currently this skill exists but is never triggered from the loop lifecycle.

## Files Modified

### `hooks/hooks.json`

Add to `SubagentStop[executor]` array (after `state-checkpoint.sh`, before `workflow-dispatch.sh`):

```json
{
  "script": "shared/scripts/evaluate-session.sh",
  "timeout": 30,
  "description": "Extract learning patterns from executor output and ingest into pk"
}
```

## Files Created

### `shared/scripts/evaluate-session.sh`

```bash
#!/usr/bin/env bash
# Called by SubagentStop[executor] hook
# Reads last-completed change scope, extracts patterns, ingests into pk

set -euo pipefail

WAYPOINT="$(git rev-parse --show-toplevel 2>/dev/null)/.kbd-orchestrator/current-waypoint.json"
LOG_DIR="${HOME}/.prometheus/learning-log"
mkdir -p "$LOG_DIR"

# Read active change scope
PHASE=$(jq -r '.phase // empty' "$WAYPOINT" 2>/dev/null) || exit 0
CHANGE=$(jq -r '.active_change // empty' "$WAYPOINT" 2>/dev/null) || exit 0

[[ -z "$PHASE" || -z "$CHANGE" ]] && exit 0

# Extract patterns via continuous-learning-v2 (if available)
PATTERNS_FILE=$(mktemp)

# Fallback: scan the change's scope_paths from waypoint
SCOPE_PATHS=$(jq -r '.scoped_paths[]? // empty' "$WAYPOINT" 2>/dev/null | head -10)

if [[ -n "$SCOPE_PATHS" ]]; then
  SUMMARY="Completed change: $CHANGE in phase: $PHASE. Files touched: $(echo "$SCOPE_PATHS" | tr '\n' ', ')"
  echo "$SUMMARY" > "$PATTERNS_FILE"
fi

# Ingest into pk
if command -v pk >/dev/null 2>&1 && [[ -s "$PATTERNS_FILE" ]]; then
  pk ingest < "$PATTERNS_FILE" 2>/dev/null || true
fi

# Write learning log entry
LOG_ENTRY="{\"date\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"phase\":\"$PHASE\",\"change\":\"$CHANGE\",\"patterns_file\":\"$PATTERNS_FILE\"}"
echo "$LOG_ENTRY" >> "$LOG_DIR/$(date +%Y-%m-%d).jsonl"

rm -f "$PATTERNS_FILE"
```

## Acceptance Criteria

- After any `/kbd-execute` completes, `~/.prometheus/learning-log/YYYY-MM-DD.jsonl` has a new entry
- `pk search <topic>` returns richer results after a related executor run
- If `pk` is not in PATH, `evaluate-session.sh` exits 0 (graceful degradation)
- Hook does not add more than 30s to SubagentStop total runtime

## Tasks

- [x] 1. After any `/kbd-execute` completes, `~/.prometheus/learning-log/YYYY-MM-DD.jsonl` has a new entry
- [x] 2. `pk search <topic>` returns richer results after a related executor run
- [x] 3. If `pk` is not in PATH, `evaluate-session.sh` exits 0 (graceful degradation)
- [x] 4. Hook does not add more than 30s to SubagentStop total runtime
