---
id: change-slli-005-pk-semantic-focus
title: Semantic fallback for pk-focus-on-prompt.sh
phase: self-learning-loop-integration
gaps: [PK-FOCUS-1]
priority: 6 of 10
agent: claude-code
status: done
scope:
  - shared/scripts/pk-focus-on-prompt.sh
---

# change-slli-005-pk-semantic-focus — Semantic fallback for pk-focus-on-prompt.sh

## Summary

Upgrade `shared/scripts/pk-focus-on-prompt.sh` to call `surreal-memory`'s `hybrid_search_memories` REST endpoint as a semantic fallback alongside the current lexical (longest-word) approach.

## Files Modified

### `shared/scripts/pk-focus-on-prompt.sh`

Extended flow:
1. **Lexical path (always runs, fast):** Extract top-5 longest words from prompt → build focus list
2. **Semantic path (runs when surreal-memory is reachable):** POST to `http://localhost:23001/api/v1/memory/search` with prompt text → extract top-3 topic keys from response
3. **Merge:** Deduplicate lexical + semantic results, pass merged list to `pk focus`
4. **Opt-out:** `PROMETHEUS_FOCUS_SEMANTIC=0` env var skips the semantic path entirely

```bash
# Semantic path addition (pseudocode):
if [[ "${PROMETHEUS_FOCUS_SEMANTIC:-1}" == "1" ]]; then
  SEMANTIC_RESPONSE=$(curl -sf --max-time 3 \
    -X POST http://localhost:23001/api/v1/memory/search \
    -H "Content-Type: application/json" \
    -d "{\"query\": \"${PROMPT_TEXT}\", \"user_id\": \"prometheus-skill-pack\", \"limit\": 3}" \
    2>/dev/null) || true
  if [[ -n "$SEMANTIC_RESPONSE" ]]; then
    SEMANTIC_TOPICS=$(echo "$SEMANTIC_RESPONSE" | jq -r '.[].memory | split(" ") | .[0:2][]' 2>/dev/null | head -6)
    # merge with lexical list
  fi
fi
```

## Acceptance Criteria

- When surreal-memory is running, `pk-focus-on-prompt.sh` calls the REST endpoint
- When surreal-memory is down, falls back to lexical-only without error output
- Total script runtime stays under 3s regardless of path
- `PROMETHEUS_FOCUS_SEMANTIC=0` disables semantic call
- No regression in current lexical behavior

## Tasks

- [x] 1. When surreal-memory is running, `pk-focus-on-prompt.sh` calls the REST endpoint
- [x] 2. When surreal-memory is down, falls back to lexical-only without error output
- [x] 3. Total script runtime stays under 3s regardless of path
- [x] 4. `PROMETHEUS_FOCUS_SEMANTIC=0` disables semantic call
- [x] 5. No regression in current lexical behavior
