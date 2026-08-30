# kbd-control-plane-reconciliation Specification

## Purpose

Defines evidence-preserving reconciliation and local certification after KBD control-plane and skill-package repairs are combined.

## Requirements

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

### Requirement: Local KBD certification does not require a control plane
The reconciliation MUST validate memory write/recall and signed KBD runtime health locally after refreshing binaries and skills. sovereign-sync SHALL remain stopped and disabled unless the operator explicitly selects sharing setup.

#### Scenario: Post-refresh daemon-free certification
- **WHEN** refreshed components are installed without a sharing profile
- **THEN** memory recall writes a non-unreachable digest, a hook event is retrievable, ordinary KBD status and typed mutations use the signed local runtime, and no unavailable-authority warning is emitted

#### Scenario: Sharing is explicitly enabled
- **WHEN** an operator invokes setup with both `--full` and `--sharing`
- **THEN** sovereign-sync is installed and started as an optional passive replication service

#### Scenario: Ordinary full setup runs
- **WHEN** an operator invokes setup with `--full` but without `--sharing`
- **THEN** every canonical or legacy sovereign-sync service identity is stopped and disabled
- **AND** KBD remains fully operational through its local signed runtime

#### Scenario: Any live gate fails
- **WHEN** a required local certification command fails
- **THEN** the change remains incomplete and records the exact failure without using hosted CI as substitute evidence
