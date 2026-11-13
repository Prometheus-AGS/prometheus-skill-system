# Tasks

- [x] 1. CLI: `close_input` passes evidence phase, projectId, sourceDigests and per-scenario attempted/observed/evidence
- [x] 2. Runtime: versioned evidence admission (pure fact checks in the reducer for `admissionVersion >= 2`; source-hash freshness at command admission)
- [x] 3. Corpus: cases 18-23 in `shared/references/conformance/kbd-phase-closure/v1` plus adapter ops; regenerate dist
- [x] 4. Replay: a closure journal written by prometheus 1.13.2 replays to the recorded state
- [x] 5. Gate: conformance corpus and replay integration targets, once, after implementation
