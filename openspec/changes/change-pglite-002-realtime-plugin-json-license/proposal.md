# Change: pglite-002 — Add License to entity-graph-realtime plugin.json

**Phase**: pglite-certification-2026-05-25  
**Gap closed**: G3  
**Priority**: Low  
**Effort**: 5 minutes

## Problem

`skills/react/prometheus-entity-skills/entity-graph-realtime/.claude-plugin/plugin.json` is missing a `"license"` field. The top-level plugin.json declares `"license": "MIT"` but the sub-plugin does not propagate this, which is inconsistent for marketplace submission.

## Proposed Change

### File: `skills/react/prometheus-entity-skills/entity-graph-realtime/.claude-plugin/plugin.json`

Add `"license": "MIT"` after `"author"`.

## Acceptance Criteria

- [ ] `license` field present with value `"MIT"`
- [ ] JSON remains valid (pass `python3 -m json.tool`)
- [ ] No other fields modified
