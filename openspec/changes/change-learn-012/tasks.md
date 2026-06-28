# Tasks — change-learn-012

- [ ] Write `skills/learn/learn-retain/SKILL.md` with invocation contract, FSRS queue read protocol, and session exit criteria
- [ ] Read FSRS due queue from learner-model via JSON RPC CLI (`learner-model-cli due-queue --today`)
- [ ] Surface review prompts for each due card via ui-surface (Tier 1 preferred, Tier 0 fallback)
- [ ] Grade each review response via `learn-grade` with retention threshold ≥ 0.6 as passing bar
- [ ] Update `FSRSCard` in learner-model via `fsrs-rs next_states()` for correct and incorrect outcomes
