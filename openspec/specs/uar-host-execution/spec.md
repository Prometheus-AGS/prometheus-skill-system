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

