# Tasks — change-learn-007

- [ ] Write `skills/learn/learn-goal/SKILL.md` with `/learn-goal` entry command, argument spec, and output contract
- [ ] Implement corpus assembly sub-step: invoke `content-grounding.sh` with subject and budget from user input
- [ ] Implement `--kb` flag routing to `content-grounding-kb.sh` when a custom knowledge base is specified
- [ ] Implement feasibility gate with RED (goal unachievable), YELLOW (partially achievable), GREEN (achievable) thresholds and criteria
- [ ] Route feasibility assessment through `sycophancy-correction` (S-01 check) before presenting result to user
