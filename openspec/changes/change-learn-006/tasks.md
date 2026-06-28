# Tasks — change-learn-006

- [ ] Write `skills/learn/ui-surface/SKILL.md` documenting the three-tier rendering model and invocation contract
- [ ] Implement Tier 0: markdown/checklist rendering (works on all harnesses, no tool dependency)
- [ ] Implement Tier 1: `AskUserQuestion` for Claude Code and `.ui-question` / `.ui-answer` file-pair convention for other harnesses
- [ ] Add Tier 2 stub with a clear comment that it requires the surface-bridge Axum server (not yet shipped)
- [ ] Document the degradation rule: never block on `preferred_tier`; always fall back to the next available tier
