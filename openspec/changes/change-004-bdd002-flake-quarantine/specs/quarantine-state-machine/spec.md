## ADDED Requirements

### Requirement: Track and report quarantine history

The runner SHALL persist per-scenario retry history in `tests/reports/quarantine-state.json` and the report SHALL show retry counts, re-promotion candidates, and persistent failure alerts.

#### Scenario: History crosses a reporting threshold

- **GIVEN** a quarantined scenario reaches five consecutive clean runs or ten consecutive runs requiring retry
- **WHEN** the runner records the latest run and builds its report
- **THEN** the report recommends removing the tag after five clean runs and alerts after ten runs requiring retry
