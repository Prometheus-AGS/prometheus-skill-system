# Tasks — change-learn-022

- [ ] Write `tests/learn/integration-full-loop.sh`: reuse the fixture KB from change-learn-021, run `feynman-loop` twice for two distinct concepts, then invoke `learn-retain` with both `feynman-artifact.json` outputs and capture the FSRS card store path
- [ ] Assert FSRS card update: after `learn-retain` completes, read the FSRS card store JSON and verify that both concepts have a card entry with non-null `due`, `stability`, and `difficulty` fields
- [ ] Invoke `learn-practice` with the updated card store and assert that `practice-result.json` is produced with at least one question-answer pair and a `score` field
- [ ] Invoke `learn-certify --checkpoint` and assert that a checkpoint VC is emitted as valid JSON-LD: check `@context`, `type`, `credentialSubject.concepts`, and `proof` top-level keys
- [ ] Add anomalous-trajectory branch: artificially set an implausibly high score in `practice-result.json`, re-run `learn-certify --checkpoint`, and assert that the VC contains `integrity_warning: true`
