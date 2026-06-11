# Goals

- Add kbd-analyze stage with tiered research pipeline (gh search, Context7, registries, firecrawl) writing analysis.md + library-candidates.json
- Add kbd-spec stage that creates native changes or dispatches /opsx:new, gated by zeespec coverage
- Implement native-kbd spec backend: tasks.json source of truth + nk_* adapter arms in kbd-apply; default for new projects
- Wire kbd_position_sync into kbd-apply end-task and the waypoint write path (Phase 1 carry-forward)
- Add pmpo-elicit ask-or-research primitive in inline-fallback mode
