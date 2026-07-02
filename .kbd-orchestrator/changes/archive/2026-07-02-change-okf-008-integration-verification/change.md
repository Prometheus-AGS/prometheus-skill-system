---
id: change-okf-008-integration-verification
title: Hooks/MCP/e2e verification of the OKF wiki stack
phase: phase-okf-llm-wiki-adoption
gaps: [Goal1, Goal2, Goal3, Goal4, Goal5]
priority: P1
effort: M
agent: claude-code
evolver_item_id: null
status: pending
model_class: medium
depends_on: [change-okf-005, change-okf-006, change-okf-007]
scope:
  - shared/scripts/pk-focus-on-prompt.sh
  - shared/scripts/pk-health.sh
  - shared/scripts/pk-lint.sh
  - tests/features/drafts/
---

# change-okf-008 — Integration verification

## Context

Hook scripts and pk-mcp knowledge_* tools consume pk output; format changes
could degrade context injection silently. The phase's mastery check is an
end-to-end round-trip on the new format.

## Tasks

- [ ] Verify pk-focus/pk-health/pk-lint hooks against OKF-format KB output
- [ ] Verify knowledge_search/knowledge_get/knowledge_ingest MCP tools (:8942)
- [ ] E2E: pk ingest a real source → index.md + log.md updated → pk focus/query
      returns cited content (falls back to static format verification if the
      002 ingest bug was deferred)
- [ ] Draft BDD feature under tests/features/drafts/ covering ingest round-trip
- [ ] Verify: all 5 phase goals re-checked against goals.md; results recorded
