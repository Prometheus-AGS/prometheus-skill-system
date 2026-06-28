# Tasks — change-learn-017

- [ ] Write `skills/learn/learn-about-system/SKILL.md` with frontmatter, overview, `--area` flag documentation (`kbd` | `skills` | `harness`), and interactive mode description
- [ ] Implement interactive discovery mode (no args): use `AskUserQuestion` to elicit the operator's interest area, then branch to the appropriate `--area` path
- [ ] Implement `--area kbd` path: load `kbd-lifecycle-corpus.json` as the active KB, invoke `learn-goal` with that corpus, and emit a session start message explaining the self-teaching loop
- [ ] Implement `--area skills` path: survey skill domains by reading `skills/` directory structure, surface the five most relevant skills for the operator's stated goal using a brief elicitation exchange
- [ ] Document the self-teaching loop pattern in `skills/learn/learn-about-system/references/self-teaching-loop.md` (how the skill pack teaches itself using its own learning infrastructure, entry → goal → survey → feynman → grade → retain cycle)
