# mobile-skill-portability Specification

## Purpose
TBD - created by archiving change change-msp-001-classify-script-skills. Update Purpose after archive.
## Requirements
### Requirement: Every script-bearing skill has a mobile execution verdict

All 60 script-bearing skills SHALL carry an execution verdict (E0/E1/E2/R), and a check SHALL fail when any is unclassified or the inventory drifts.

#### Scenario: An unclassified skill fails the check

- **GIVEN** a script-bearing skill with no verdict
- **WHEN** the classification check runs
- **THEN** it exits non-zero naming that skill

### Requirement: zed delivery is either verified Tier 1 or recorded Tier 0

The zed outcome SHALL be exactly one of verified-Tier-1 (round trip executed) or recorded-Tier-0 with a diagnostic. No Tier 1 claim SHALL be made without an executed round trip.

#### Scenario: Tier 1 is claimed only when the round trip ran

- **GIVEN** zed is claimed as Tier 1
- **WHEN** the evidence is checked
- **THEN** an executed file-pair round trip is present

### Requirement: Harness delivery claims rest on executed round trips

A harness SHALL NOT be claimed as Tier 1 without an executed file-pair round trip; a failure SHALL be recorded rather than the claim narrowed.

#### Scenario: An unrun harness is not claimed

- **GIVEN** opencode or kimi has no executed round trip
- **WHEN** delivery is reported
- **THEN** that harness is not listed as verified Tier 1

### Requirement: Version invariants are enforced, with violations quarantined not suppressed

The three holding invariants (Loro minor, wasmtime major, iroh floor) SHALL fail CI on drift. The already-violated WIT invariant SHALL be reported and allowlisted, and the check SHALL fail both on un-allowlisted violations and on allowlisted entries that have been fixed.

#### Scenario: A fixed allowlist entry fails until removed

- **GIVEN** an allowlisted violation no longer occurs
- **WHEN** the check runs
- **THEN** it exits non-zero demanding the entry be removed

### Requirement: One WIT family supersedes the divergent worlds

The prometheus:component@0.1.0 family SHALL parse under wasm-tools, its skill world SHALL be a superset of UAR's run contract, and a mapping SHALL record how each existing target relates to it, including any that cannot be expressed. The change SHALL abort if UAR's discovery path no longer reads the submodule skills dir.

#### Scenario: A changed discovery path aborts the change

- **GIVEN** UAR no longer discovers components in the submodule skills dir
- **WHEN** the precondition check runs
- **THEN** the change aborts rather than proceeding

