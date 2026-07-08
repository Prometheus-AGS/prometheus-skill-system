# Tasks — change-prui-002-htmx-ui-real-sse

- [ ] task-001: Read `docs/deep-research/deep-research-ui.html` — identify CDN script tags and `startResearch()` call sites
- [ ] task-002: Replace CDN HTMX and Alpine.js `<script>` tags with `/static/` paths
- [ ] task-003: Rewrite `startResearch()` to POST `/api/v1/jobs` and capture `job_id`
- [ ] task-004: Wire `EventSource` to `/api/v1/jobs/{job_id}/events`; map `agent.status`, `agent.message`, `agent.error`, `a2ui.component` to Alpine state
- [ ] task-005: Gate original `simulateProgress()` / `simulateSongResearch()` behind `?demo=1` check
- [ ] task-006: Update cancel flow to call `DELETE /api/v1/jobs/{job_id}`
- [ ] task-007: Manual browser test: open UI against running binary, start job, verify SSE events drive UI updates
- [ ] task-008: Commit with message `feat(deep-research-ui): replace simulation with real SSE/HTTP calls to prometheus-research`
