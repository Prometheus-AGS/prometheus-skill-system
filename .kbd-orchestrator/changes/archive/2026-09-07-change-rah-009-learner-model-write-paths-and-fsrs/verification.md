# Verification — change-rah-009-learner-model-write-paths-and-fsrs

Repository: `prometheus-skill-pack`
Depends on: `change-rah-001-compile-baseline-and-timestamps`, `change-rah-008-learn-artifact-and-corpus-coherence`

## Acceptance criteria

- `cargo test -p learner-model --test rpc_roundtrip` passes against the built binary over stdin and stdout.
- The `cargo tree` result for the chosen FSRS path is recorded in Evidence.
- The three learn skills reference the new methods and pass `npm run validate:strict`.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run (for example, another Cargo build is active) is recorded BLOCKED with the reason, never skipped silently.

```verify
test -z "$(pgrep -x cargo)"
cd substrate/learner-model && cargo test -p learner-model --test rpc_roundtrip
for s in learn-grade learn-practice learn-certify; do npm run validate:strict skills/learn/$s || exit 1; done
```

## Evidence

Run locally on 2026-09-07 (UTC) on branch `feat/cpc-001-002-integration-contract` at base `cfbc262` (working tree, uncommitted). No hosted CI.

| Gate | Command | Result |
|---|---|---|
| Cargo slot | `pgrep -x cargo` polled before every cargo command | Idle at each run (one 10-min wait loop needed once; another session's build held the slot at the first attempt) |
| FSRS dependency weight | `cargo tree` on scratch manifests (task 1) | `rs-fsrs` 1.2.1: 7 unique crates, all already in the tree; `fsrs` 6.6.2: 41 crates (burn stack). Decision: adopt rs-fsrs (README "FSRS dependency decision") |
| Compiler diagnostic | `cargo check -p learner-model --tests` | `Finished`, 0 errors, 0 warnings (3 runs: initial, after round 1, after round 2) |
| Integration gate | `cargo test -p learner-model --test rpc_roundtrip` | `test result: ok. 1 passed; 0 failed` (3 runs; final run after the round-2 fix). Asserts over the built binary: mastery unchanged through 4 observations and 0.58 after the 5th; `add_gap`/`add_session`/`set_certified` round-trip through `get_concept` and a fresh `load` with their ids; session update replaces (no duplicate) and preserves `started_at`; `certified_at` set on c1, null on c2; Good→Hard reviews change `difficulty` (Hard higher); error replies for unknown concept, bad/non-string timestamps, bad/non-string label and severity, unknown gap; rejected calls write nothing |
| Task verify strings | `kbd-apply` per-task `verify` (tasks 1, 2, 4; 3 has none; 5 = the integration gate) | All OK (task 1 required rewording the decision line so `rs-fsrs` precedes `adopted`) |
| Skill validation | `npm run validate:strict skills/learn/{learn-grade,learn-practice,learn-certify}` | rc=0 each (advisory trigger/exclusion-clause notes are pre-existing; description lines untouched) |
| OpenSpec | `openspec validate --specs` | 31 passed, 0 failed (new `learn-model-coherence` included) |
| Formatting | `cargo fmt -p learner-model -- --check` after `cargo fmt` | clean |
| Adversarial review | k3 judge, 2 rounds | Round 1 BLOCK (1C/4W) all fixed; Round 2 BLOCK (1C) fixed after the cap with a test probe; sycophancy gate PASS 0.0 both rounds. See spec.md "Unresolved review findings" |

Unit-test modules in `src/fsrs.rs` were rewritten to property assertions but are not cited as evidence (policy: integration-only).

## Verdict

**PASS WITH NOTES.**

- Note 1: the declared dependency on rah-008 was taken out of order (see spec.md "Dependency note"); rah-008 must merge with these SKILL.md sections.
- Note 2: rs-fsrs 1.2.1 implements FSRS-5 (19 weights), not FSRS-6; recorded in README. The spec's optional requirement asked for FSRS-6 formulas only on the port path, which was not taken.
- Note 3: round-2 CRITICAL was fixed without a third judge round (2-round cap); the refuting probe is in the integration test.
- Note 4: learn-practice's `learner_id` key was changed from `$GOAL_ID` to `$LEARNER_ID` in the pre-existing `add_observation` snippet as well, so all three skills key one model. The learner-DID vs goal-id convention across the rest of the learn skills is rah-008's coherence scope.
