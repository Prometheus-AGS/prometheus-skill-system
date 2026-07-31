# Tasks

- [x] Emit UiIntent from the ideation flow; never render directly
- [x] Claude Code: detect-surface-tier.sh reports tier1 and the prompt appears
- [x] One named non-Claude harness: writes __ui_intent__.json, consumes __ui_response__.json within 30s, flow continues — RUN IT
- [x] If the harness does not poll, record a stated limit and fall back to Tier 0 rather than claiming delivery
- [x] Exercise Tier 0 degradation by forcing tier0_text
