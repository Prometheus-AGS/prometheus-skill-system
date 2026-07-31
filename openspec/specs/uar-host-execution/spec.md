# uar-host-execution Specification

## Purpose
TBD - created by archiving change change-uhe-001-cursor-tier1. Update Purpose after archive.
## Requirements
### Requirement: cursor delivery is either verified Tier 1 or recorded Tier 0

The cursor outcome SHALL be exactly one of verified-Tier-1 (round trip executed) or recorded-Tier-0 with a diagnostic. No Tier 1 claim SHALL be made without an executed round trip.

#### Scenario: Tier 1 is claimed only when the round trip ran

- **GIVEN** cursor is claimed as Tier 1
- **WHEN** the evidence is checked
- **THEN** an executed file-pair round trip is present

### Requirement: CI verifies all four invariants or states the limit

The fabric-invariants job SHALL report 4 of 4 verified with 0 SKIP, or SHALL record a stated limit naming the missing prerequisite. Silent partial coverage SHALL NOT be reported as coverage.

#### Scenario: Partial coverage is never silent

- **GIVEN** a sibling repo cannot be checked out
- **WHEN** the job runs
- **THEN** the SKIP is recorded as a stated limit rather than reported as verified

