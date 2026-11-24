## ADDED Requirements

### Requirement: Record the audited scope of a readiness claim

The bounded production-readiness claim SHALL retain a sycophancy audit score, tool version, and timestamp. A score of at least 0.15 SHALL trigger revision and another audit.

#### Scenario: Audit a claim above the threshold

- **WHEN** the recorded claim scores 0.15 or higher
- **THEN** identified patterns are corrected and a later score below 0.15 is required before recording certification
