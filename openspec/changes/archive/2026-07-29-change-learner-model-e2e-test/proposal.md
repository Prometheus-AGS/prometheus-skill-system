# Add a learner-model end-to-end replication test

## Why

`tests/domain_sync.rs::skill_index_replicates_end_to_end_between_two_nodes`
proves the full push → CRDT merge → receive → adapter-import pipeline for
the `skill-index` domain. `LearnerModelAdapter` runs through the exact same
generic pipeline (`build_push_envelope` / `handle_incoming_message`) but has
no equivalent test — a regression in the learner-model bridge (e.g. a
serde-shape mismatch between `learner_model::LearnerModel` and what
`LoroAdapter::apply_json`/`to_json` round-trips) would only surface at
runtime, not in CI.

## What Changes

Add a test to `substrate/sovereign-sync/tests/domain_sync.rs` (or a new
file) mirroring the skill-index one: two `AppState`s, one with a real
learner model seeded via `learner_model::seed_from_survey` or a hand-built
`LearnerModel`, push `learner-model` from node A, hand the envelope to node
B's `handle_incoming_message`, and assert node B's on-disk learner-model
store (via `LearnerModelStore::load`) now contains node A's content.

## Impact

- `substrate/sovereign-sync/tests/domain_sync.rs` (new test function)
- No production code change expected unless the test surfaces a real bug
  in `LearnerModelAdapter`
