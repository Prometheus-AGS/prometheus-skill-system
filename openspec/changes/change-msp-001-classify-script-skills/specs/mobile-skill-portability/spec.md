## ADDED Requirements

### Requirement: Every script-bearing skill has a mobile execution verdict

All 60 script-bearing skills SHALL carry an execution verdict (E0/E1/E2/R), and a check SHALL fail when any is unclassified or the inventory drifts.

#### Scenario: An unclassified skill fails the check

- **GIVEN** a script-bearing skill with no verdict
- **WHEN** the classification check runs
- **THEN** it exits non-zero naming that skill
