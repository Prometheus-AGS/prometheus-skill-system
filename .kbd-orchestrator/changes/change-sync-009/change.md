# change-sync-009: AG-UI + A2UI streaming endpoint

**Phase:** phase-learn-sovereign-sync
**Tier:** 2 (parallelize with 008, 010, 011 after Tier 1)
**Status:** pending
**Library:** cand-014 (AG-UI from flint-gate), cand-015 (A2UI from flint-gate)
**Gap:** G-NEW-4

## Summary

Port AG-UI and A2UI types from flint-gate. Implement SSE streaming route
for agent task execution. Implement task schema endpoint that dynamically
enumerates SkillIndex entries.

## Files to change

- `substrate/sovereign-sync/src/ag_ui.rs` — port from `flint-gate/src/stream/ag_ui.rs`
- `substrate/sovereign-sync/src/a2ui.rs` — port from `flint-gate/src/stream/a2ui.rs`
- `substrate/sovereign-sync/src/routes/agent.rs` — SSE route + task schema

## Source files to port from

- `/Users/gqadonis/Projects/know-me/flint-gate/src/stream/ag_ui.rs`
- `/Users/gqadonis/Projects/know-me/flint-gate/src/stream/a2ui.rs`

## Endpoints

```
POST /sovereign/agent/run         → SSE stream of AG-UI events
GET  /sovereign/tasks/schema      → JSON schema of all skill tasks
POST /sovereign/tasks/{id}/run    → AG-UI SSE stream for named task
GET  /sovereign/tasks/{id}/status
```

## A2UI intents used

- `render_component`: sync dashboard widget
- `update_state`: sync progress updates
- `stream_content`: sync log lines
- `request_input`: conflict resolution prompts

## Tasks

- [ ] Read and port ag_ui.rs (15 event types)
- [ ] Read and port a2ui.rs (7 intent types)
- [ ] Implement task schema generation from SkillIndex
- [ ] Implement SSE route with proper Axum streaming
- [ ] Test: POST /sovereign/agent/run returns RUN_STARTED, RUN_FINISHED events
