# harness-completeness-gate Specification

## Purpose

Prevent declared harness skill trees from silently disappearing while allowing
targets whose source trees are intentionally created only during installation.

## Requirements

### Requirement: A harness losing its tree fails a gate
A gate SHALL require every declared target to specify a `required | install-only`
source-tree lifecycle. It SHALL assert that every `required` skills tree is present and
populated, deriving the target set and lifecycle from the manifest rather than a hardcoded
list. An `install-only` target MAY have no source tree.

#### Scenario: A harness tree is emptied
- **WHEN** every skill directory is removed from one declared harness
- **THEN** the gate SHALL exit non-zero and name that harness

#### Scenario: A lifecycle policy is omitted
- **WHEN** a declared target omits its source-tree lifecycle
- **THEN** the gate SHALL exit non-zero and name that target

#### Scenario: An install-only source tree is absent
- **WHEN** a target declares `install-only` and its repository source tree is absent
- **THEN** validation and installation preflight SHALL continue

#### Scenario: A harness is added to the manifest
- **WHEN** a new harness is declared
- **THEN** the gate SHALL cover it without editing the gate, because the set is derived

#### Scenario: The gate runs twice
- **WHEN** the gate or any normalizer it ships runs a second consecutive time
- **THEN** the working tree SHALL be unchanged, satisfying the idempotence constraint
