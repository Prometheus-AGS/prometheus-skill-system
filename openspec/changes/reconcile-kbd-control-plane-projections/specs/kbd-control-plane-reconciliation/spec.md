## Purpose

Defines evidence-preserving reconciliation and local certification after KBD control-plane and skill-package repairs are combined.

## ADDED Requirements

### Requirement: Compatibility projections leave live discovery without evidence loss
A duplicate compatibility projection SHALL be moved outside the live phase namespace only after its canonical record is confirmed, and a receipt SHALL link the preserved copy to that canonical record.

#### Scenario: Duplicate projection is confirmed
- **WHEN** the compatibility directory contains non-canonical evidence and the canonical nested child record exists
- **THEN** the compatibility directory is preserved under the backup area with a receipt and no canonical file is changed

#### Scenario: Canonical record cannot be confirmed
- **WHEN** the expected nested child record is missing or ambiguous
- **THEN** reconciliation stops without moving the compatibility directory

### Requirement: Generated and installed skill surfaces are consistent
The reconciliation SHALL prove deterministic distribution generation, validate the generated surfaces, and refresh the user installation through a repository-owned installer.

#### Scenario: Distribution refresh succeeds
- **WHEN** the source repairs are complete
- **THEN** two generation runs are byte-identical, validation passes, and installed managed copies match the generated/source payloads

### Requirement: Live control-plane certification covers memory and authority health
The reconciliation MUST validate memory write/recall and KBD authority health locally after refreshing binaries and skills.

#### Scenario: Post-refresh live certification
- **WHEN** refreshed components are installed and sovereign-sync is restarted twice
- **THEN** memory recall writes a non-unreachable digest, a hook event is retrievable, KBD doctor/status pass, and current restart logs contain no unavailable-authority warning

#### Scenario: Any live gate fails
- **WHEN** a required local certification command fails
- **THEN** the change remains incomplete and records the exact failure without using hosted CI as substitute evidence
