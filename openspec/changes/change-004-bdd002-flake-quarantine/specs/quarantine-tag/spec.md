## ADDED Requirements

### Requirement: Bound retries to tagged video scenarios

The runner SHALL allow up to three retries for scenarios tagged @quarantine and retain fail-fast behavior for untagged scenarios.

#### Scenario: Transient and untagged failures

- **GIVEN** a quarantined scenario fails once and an untagged scenario fails
- **WHEN** the proof runner encounters each failure
- **THEN** the quarantined scenario is retried within its limit and the untagged failure ends the run
