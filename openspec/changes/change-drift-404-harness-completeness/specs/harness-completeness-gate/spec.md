## ADDED Requirements

### Requirement: A harness losing its tree fails a gate
A gate SHALL assert, per declared harness, that its skills tree is present and populated,
deriving the harness set from the manifest rather than a hardcoded list.

#### Scenario: A harness tree is emptied
- **WHEN** every skill directory is removed from one declared harness
- **THEN** the gate SHALL exit non-zero and name that harness

#### Scenario: A harness is added to the manifest
- **WHEN** a new harness is declared
- **THEN** the gate SHALL cover it without editing the gate, because the set is derived

#### Scenario: The gate runs twice
- **WHEN** the gate or any normalizer it ships runs a second consecutive time
- **THEN** the working tree SHALL be unchanged, satisfying the idempotence constraint
