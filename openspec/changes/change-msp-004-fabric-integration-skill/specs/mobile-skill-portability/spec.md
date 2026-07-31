## ADDED Requirements

### Requirement: Version invariants are enforced, with violations quarantined not suppressed

The three holding invariants (Loro minor, wasmtime major, iroh floor) SHALL fail CI on drift. The already-violated WIT invariant SHALL be reported and allowlisted, and the check SHALL fail both on un-allowlisted violations and on allowlisted entries that have been fixed.

#### Scenario: A fixed allowlist entry fails until removed

- **GIVEN** an allowlisted violation no longer occurs
- **WHEN** the check runs
- **THEN** it exits non-zero demanding the entry be removed
