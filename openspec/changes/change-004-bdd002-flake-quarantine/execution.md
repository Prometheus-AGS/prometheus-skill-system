# Execution — change-004-bdd002-flake-quarantine

**Executed:** 2026-05-09  
**Backend:** OpenSpec  
**Agent role:** bdd-engineer  
**Executor:** claude-sonnet-4-6

## Dispatch

Feature implementation in ssr-frontend. Three files modified, one documented.

## Files Modified (ssr-frontend)

1. `scripts/run-video-proof.ts`
   - Added `QUARANTINE_STATE_PATH`, `QUARANTINE_MAX_RETRIES` (3), `QUARANTINE_PROMOTE_THRESHOLD` (5), `QUARANTINE_ESCALATE_THRESHOLD` (10) constants.
   - Added `QuarantineScenarioState`, `QuarantineState` interfaces.
   - Added `loadQuarantineState()`, `saveQuarantineState()`, `updateQuarantineScenario()`, `quarantineAdvisory()` helpers.
   - Main loop: quarantined scenarios get up to 3 retries before pipeline failure; non-quarantined scenarios keep fail-fast behavior.
   - Quarantine state persisted to `tests/reports/quarantine-state.json` after every scenario that carries the `@quarantine` tag.

2. `scripts/generate-video-run-report.ts`
   - Added `QUARANTINE_STATE_PATH`, `QUARANTINE_PROMOTE_THRESHOLD`, `QUARANTINE_ESCALATE_THRESHOLD` constants.
   - Added `QuarantineScenarioState`, `QuarantineState` interfaces.
   - Report now loads `quarantine-state.json` and emits a "Quarantined Scenarios" section with per-scenario retry counts, promote candidates, and escalation alerts.

3. `tests/README.md`
   - New "Quarantine Tag — Flaky Scenario Handling" section documenting: tag syntax, behavior, thresholds, lifecycle, and state file.

## QA Gate

Applied: 3 files modified (≥3 threshold met). Type-check passes (only pre-existing deprecation warning). No runtime quarantine-state.json created (no quarantined scenarios exist yet — that is correct; the file is created on first use).

**Acceptance criteria check:**
- [x] `@quarantine` tag recognized in run-video-proof.ts
- [x] Retry loop: up to 3 retries for quarantined scenarios
- [x] Non-quarantined scenarios keep fail-fast behavior
- [x] `tests/reports/quarantine-state.json` created on first quarantined run (file not committed; runtime artifact)
- [x] Report section: "Quarantined Scenarios" with retry counts, promote candidates, escalations
- [x] `tests/README.md` documents the convention

## Status

DONE
