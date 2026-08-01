## ADDED Requirements

### Requirement: Recorded findings are re-verified, not merely written

Each recorded finding SHALL carry a reproduction command re-run at write time. The existence of a decision record SHALL NOT by itself satisfy this requirement, because a record of false findings would pass such a check.

#### Scenario: A stale finding fails the check

- **GIVEN** a recorded finding no longer reproduces
- **WHEN** its command is re-run at write time
- **THEN** the finding is corrected rather than recorded as-is
