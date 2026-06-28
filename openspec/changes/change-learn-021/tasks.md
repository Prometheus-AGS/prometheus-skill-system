# Tasks — change-learn-021

- [ ] Write `tests/learn/integration-basic-flow.sh`: set up a temp working directory, invoke `learn-goal` with a test concept, then `learn-survey`, then `feynman-loop` (Tier 0), then `learn-grade` in sequence; capture exit codes and artifact paths
- [ ] Assert happy path: verify all four artifact files are produced — `learn-goal.json`, `survey-result.json`, `feynman-artifact.json`, `grade-result.json` — and that each parses as valid JSON with expected top-level keys
- [ ] Create `tests/learn/fixtures/sample-kb/` with at least three markdown files covering a narrow technical concept (e.g. FSRS spacing algorithm basics) for use as a fixture KB in KB-path tests
- [ ] Add KB-path test branch to `integration-basic-flow.sh`: run `learn-goal` with `--kb tests/learn/fixtures/sample-kb/` and assert that `learn-goal.json` reflects KB-sourced concepts
- [ ] Add sycophancy-correction assertion: grep `grade-result.json` for a `sycophancy_check` key (or equivalent) and assert its value is not `null`; fail with a clear message if the field is absent
