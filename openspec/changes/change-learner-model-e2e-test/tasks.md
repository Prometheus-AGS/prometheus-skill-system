# Tasks

- [ ] Add `learner_model_replicates_end_to_end_between_two_nodes` to
      `tests/domain_sync.rs`, mirroring the skill-index test's structure
- [ ] Seed a real `LearnerModel` on node A (via `seed_from_survey` or a
      hand-built fixture) before pushing
- [ ] Assert node B's `LearnerModelStore::load` reflects node A's content
      after `handle_incoming_message`
- [ ] `cargo test -p sovereign-sync` green
