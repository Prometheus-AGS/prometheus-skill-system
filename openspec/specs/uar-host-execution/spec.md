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

### Requirement: Dynamic registration is off by default

A skill generated by this pack SHALL register in the UAR database only when an explicit opt-in is set. Without the opt-in nothing SHALL be written, proven by a test.

#### Scenario: Silence writes nothing

- **GIVEN** no opt-in flag is set
- **WHEN** a skill is generated
- **THEN** no row is written to the UAR database

### Requirement: Builtin skills are visibly undeletable

The admin UI SHALL make builtin skills visually distinguishable and their delete affordance absent or disabled. A delete control that returns 409 SHALL NOT satisfy this requirement.

#### Scenario: No dead delete button

- **GIVEN** a builtin skill is shown in the admin UI
- **WHEN** the user looks for a delete control
- **THEN** it is absent or disabled rather than present and failing

### Requirement: An unreachable network never reports up-to-date

UAR SHALL report up-to-date, behind-by-N, or unknown when comparing the loaded manifest against the GitHub repository, and SHALL report unknown on network failure. A desktop or server update SHALL be initiable. Tests SHALL use a fixture manifest, not live GitHub.

#### Scenario: Network failure is unknown, not current

- **GIVEN** the GitHub check cannot reach the network
- **WHEN** the update status is reported
- **THEN** it is unknown, never up-to-date

### Requirement: Mobile updates without git, or PARTIAL

The transport SHALL be chosen in a reviewed decision record, then a mobile-reachable path SHALL fetch a versioned bundle and the provenance surface SHALL reflect the new version. If the transport cannot be exercised, the change SHALL be archived BLOCKED naming the prerequisite and R5 SHALL be reported PARTIAL, never MET on the decision alone.

#### Scenario: A decision alone is not MET

- **GIVEN** the transport is decided but never exercised
- **WHEN** R5 is reported
- **THEN** it is PARTIAL, not MET

### Requirement: Waypoint defects are detected in this repo

A check SHALL exit non-zero when current-waypoint.json names a phase disagreeing with the active phase directory, or when next is self-referential. The fix itself SHALL NOT be applied to installed skills from this repository, because such edits are destroyed by the next install.

#### Scenario: The next occurrence is caught

- **GIVEN** the waypoint next field is self-referential
- **WHEN** the check runs
- **THEN** it exits non-zero naming the defect

### Requirement: A skill component returns its own output

The Wasm runtime SHALL instantiate and invoke a component so the reference skill returns its own output rather than the placeholder string. Nothing SHALL be described as end-to-end parity until this passes.

#### Scenario: Placeholder output is not execution

- **GIVEN** the runtime returns the placeholder string
- **WHEN** the result is evaluated
- **THEN** the change is not complete and parity is not claimed

