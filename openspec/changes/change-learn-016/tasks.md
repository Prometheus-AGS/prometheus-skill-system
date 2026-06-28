# Tasks — change-learn-016

- [ ] Write `docs/learn/meta-corpus/kbd-lifecycle-corpus.json` with concept entries for: assess, analyze, plan, execute, reflect, evolve, OpenSpec, hooks (PreToolUse/PostToolUse/Stop), waypoints (position-reminder.txt, current-waypoint.json, progress.json), and progress signaling format
- [ ] Add misconception entries to `kbd-lifecycle-corpus.json` for common wrong mental models (e.g. treating reflect as a success summary, skipping progress signals, confusing phase vs. change)
- [ ] Write `docs/learn/meta-corpus/skill-pack-corpus.json` with concept entries for: skill domains, SKILL.md frontmatter schema, dual-format (agentskills.io + Claude Code plugin), imported submodules, validate:strict vs. validate:skill, and install-skills-flat.sh platform targets
- [ ] Add misconception entries to `skill-pack-corpus.json` for common wrong mental models (e.g. editing `.claude-plugin/` directly, confusing `name` field with directory name, using backslashes in paths)
- [ ] Validate both corpora against `grounding-corpus.schema.json` (run `npm run validate:strict` or equivalent schema check; fix any schema violations)
