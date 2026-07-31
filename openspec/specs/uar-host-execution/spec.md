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

### Requirement: The FFI pattern is confirmed or reversed by measurement

Adding a second function SHALL have its hand-written glue counted. Exceeding the decision's threshold SHALL reverse the pattern choice and record that reversal.

#### Scenario: Exceeding the threshold reverses the decision

- **GIVEN** adding one function needs more than the threshold of hand-written glue
- **WHEN** the measurement is recorded
- **THEN** the pattern decision is reversed rather than retained

### Requirement: The librefang ABI fork is decided under review

The choice among port, keep-both, and retire SHALL be recorded via decision-log.sh with alternatives, a stated falsifier, and outcome_status pending, and SHALL pass decision-mode review with cross_model_check verified-distinct. No code SHALL be written.

#### Scenario: The decision carries a falsifier

- **GIVEN** the decision record is reviewed
- **WHEN** the falsifier field is read
- **THEN** it names a measurable condition that would reverse the choice

### Requirement: The loaded pack version is knowable at runtime

The pack SHALL emit a version manifest and UAR SHALL expose the loaded version, commit, and skill count without shelling out to git (impossible on mobile). Drift of the kind that went 359 commits undetected SHALL be visible through this surface.

#### Scenario: Drift becomes visible

- **GIVEN** the loaded pack is behind the manifest it was built from
- **WHEN** the provenance surface is queried
- **THEN** it reports a version distinguishable from the current one

