## ADDED Requirements

### Requirement: CI verifies all four invariants or states the limit

The fabric-invariants job SHALL report 4 of 4 verified with 0 SKIP, or SHALL record a stated limit naming the missing prerequisite. Silent partial coverage SHALL NOT be reported as coverage.

#### Scenario: Partial coverage is never silent

- **GIVEN** a sibling repo cannot be checked out
- **WHEN** the job runs
- **THEN** the SKIP is recorded as a stated limit rather than reported as verified
