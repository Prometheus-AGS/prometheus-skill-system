## Why

`run-video-proof.ts` (412 lines) has zero quarantine, retry, or flake-handling logic. At 250+ Playwright scenarios, even a 0.5% per-scenario flake rate produces 1–2 spurious failures per full run. The current behavior stops the entire pipeline on the first failure.

Developers respond by tagging scenarios `@no-guide-video` to escape the gate, silently eroding coverage. Assessment confirmed: no `@quarantine` tag handling, no retry logic, no `tests/reports/quarantine-state.json`.

Two competing requirements must be satisfied simultaneously:
- **Release gate**: fail fast on any real failure.
- **Development gate**: absorb known transient flake without killing the run.

## What Changes

- Add `@quarantine` tag recognition to `run-video-proof.ts`:
  - Quarantined scenarios get up to 3 retries before declaring failure.
  - Non-quarantined scenarios keep current fail-fast behavior.
- Add `tests/reports/quarantine-state.json` state machine:
  - Tracks per-scenario retry history across runs.
  - After 5 consecutive clean runs (no retry needed): advisory suggestion to remove `@quarantine`.
  - After 10 consecutive runs requiring retry: escalation alert ("this scenario is broken, investigate").
- Update `generate-video-run-report.ts` to add a "Quarantined scenarios" section with retry counts and re-promotion candidates.
- Document the convention in `tests/README.md`.

## Capabilities

### New Capabilities
- `quarantine-tag`: `@quarantine` tag support in the video proof runner with up to 3 retries.
- `quarantine-state-machine`: Per-scenario retry history tracking with promotion suggestion and escalation alert thresholds.
- `quarantine-reporting`: Quarantine section in the video run report.

### Modified Capabilities
- `video-proof-runner`: Extended with retry logic for quarantined scenarios.
- `video-run-report`: Extended with quarantine section.

## Impact

- `ssr-frontend/scripts/run-video-proof.ts` — add quarantine tag reading + retry loop
- `ssr-frontend/scripts/generate-video-run-report.ts` — add quarantine section
- `ssr-frontend/tests/reports/quarantine-state.json` — new state file (gitignored or committed, TBD)
- `ssr-frontend/tests/README.md` — document `@quarantine` convention
- No changes to existing `.feature` files or step definitions
