# Tasks — change-learn-005

- [ ] Define `LearnerModel`, `ConceptState`, `FSRSCard`, and `GapRecord` types matching the schema from change-learn-001
- [ ] Implement automerge document backend using the `CrdtEngine` trait from change-learn-004b
- [ ] Implement `seed_from_survey()` — cold-start from LLM-derived mastery priors when no prior data exists
- [ ] Integrate `fsrs-rs` for card scheduling (due-date calculation, `next_states()`, stability/difficulty tracking)
- [ ] Expose MCP-callable JSON RPC shell interface (`learner-model-cli`) for read/write/query from skill scripts
