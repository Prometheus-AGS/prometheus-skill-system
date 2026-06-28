# Tasks — change-learn-008

- [ ] Write `skills/learn/learn-survey/SKILL.md` with invocation contract, input (corpus from learn-goal), and output spec
- [ ] Generate diagnostic items from corpus: conceptual questions, procedural tasks, and misconception probe prompts
- [ ] Render items via ui-surface at Tier 1 (preferred); degrade to Tier 0 if unavailable
- [ ] Produce `survey-result.json` with `recursion_floor` (minimum Feynman depth), `mastery_priors` per concept, and confidence intervals
- [ ] Write `learner_model_seed` to the learner-model crate via the JSON RPC shell interface from change-learn-005
