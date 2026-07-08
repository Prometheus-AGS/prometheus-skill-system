# Tasks — change-prui-002-htmx-ui-real-sse

- [ ] task-001: Read `docs/deep-research/deep-research-ui.html` — map Alpine state shape, locate CDN script tags and all `simulateProgress()` / `simulateSongResearch()` call sites
- [ ] task-002: Move all CSS to `:root` custom properties (OKLCH tokens per plan.md design spec); replace every hardcoded hex/hsl value
- [ ] task-003: Replace CDN `<script>` tags with `/static/htmx.min.js` and `/static/alpine.min.js`
- [ ] task-004: Rewrite `startResearch()`: POST `/api/v1/jobs` → capture `job_id`, gate simulation behind `?demo=1`
- [ ] task-005: Wire `EventSource` to `/api/v1/jobs/{job_id}/events`; map all 4 AG-UI event types to Alpine state updates
- [ ] task-006: Apply `animate` skill spec — progress ring `stroke-dashoffset` transition, stage slide-in, entrance choreography; add `prefers-reduced-motion` gate
- [ ] task-007: Apply `delight` skill moments — start pulse, stage-done flash, job-done checkmark SVG draw, error shake, SSE-connect dot
- [ ] task-008: Apply `polish` skill — all 5 interaction states per interactive element; focus rings; spacing grid; typography hierarchy
- [ ] task-009: Update cancel button → `DELETE /api/v1/jobs/{job_id}`
- [ ] task-010: Manual browser test with running binary: start job, verify SSE events drive all panels; verify demo mode still works with `?demo=1`
- [ ] task-011: Commit: `feat(deep-research-ui): real SSE AG-UI integration with Anthropic design system`
