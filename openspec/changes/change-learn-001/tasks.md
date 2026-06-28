# Tasks — change-learn-001

- [ ] Define `learner_model_seed` JSON schema covering all fields (concept_states, fsrs_cards, gap_records, mastery_priors, metadata)
- [ ] Document CRDT field-level merge strategy for every field (LWW, max, set-union, or custom)
- [ ] Write 3 worked conflict resolution examples showing before/after merge for realistic divergent states
- [ ] Define PFA update rule triggered at ≥ 5 observations (formula, inputs, output fields)
- [ ] Review schema doc against conflict semantics doc for consistency and flag any gaps
