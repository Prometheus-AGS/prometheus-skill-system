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

### Requirement: Skill origin is expressible in a database constraint

This change SHALL first probe whether a constraint can target definition->>'origin'. It SHALL complete in either branch — delivering that expression, or adding real columns — so that dependent changes never dangle.

#### Scenario: Either branch completes the change

- **GIVEN** the probe finds a constraint can target the JSONB field
- **WHEN** the change finishes
- **THEN** it delivers that expression and adds no columns, and dependent ordering is unchanged

### Requirement: Builtin deletion is refused at the database

A DELETE of a Builtin skill SHALL fail at the database layer. A guard present only in SkillService SHALL NOT satisfy this requirement, because a caller reaching the storage provider directly bypasses it.

#### Scenario: The bypass route is closed

- **GIVEN** a caller invokes the storage provider directly, bypassing SkillService
- **WHEN** it deletes a Builtin skill
- **THEN** the delete is refused

### Requirement: Builtin registration holds on all three persistence providers

After startup, the count of builtin skills in the database SHALL equal the loader's discovered count, on postgres, surreal, and memory. The memory provider is the embedded case and SHALL NOT be skipped. A provider that cannot be exercised SHALL be recorded BLOCKED and R1 reported PARTIAL.

#### Scenario: One provider is not enough

- **GIVEN** only one persistence provider has been verified
- **WHEN** R1 is reported
- **THEN** it is PARTIAL, not MET

### Requirement: An embedder uses a public API, not internals

UAR SHALL expose a public skill facade (list, get, install, toggle, query) consumable from an external crate, proven by an integration test in tests/ that uses only the public API. Runtime internals SHALL remain private.

#### Scenario: The facade is usable without reaching into internals

- **GIVEN** an external crate consumes the SDK
- **WHEN** it lists and toggles a skill
- **THEN** it does so without importing uar::runtime::skills internals

### Requirement: Every REST verb R4 names has a passing test

Skill installation and query endpoints SHALL each have a passing request/response test covering install, list, get, search, and toggle. The existence of a mounted route SHALL NOT by itself satisfy this requirement.

#### Scenario: Existence is not acceptance

- **GIVEN** an endpoint is mounted but has no passing test
- **WHEN** R4 coverage is reported
- **THEN** that verb is recorded as a gap rather than counted as covered

