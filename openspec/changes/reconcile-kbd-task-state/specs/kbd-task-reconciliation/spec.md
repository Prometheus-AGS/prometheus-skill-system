# KBD task reconciliation

## ADDED Requirements

### Requirement: Read-only complete reconciliation
The command SHALL compare backend task state, canonical identities and phase counters without changing the live project, including archived OpenSpec, native KBD and Spec Kit artifacts.

#### Scenario: Archived completion
- **WHEN** archived backend tasks match the selected phase ledger
- **THEN** the command exits 0 and reports clean

#### Scenario: Missing or ambiguous input
- **WHEN** backend artifacts or canonical state cannot be read unambiguously
- **THEN** the command exits 2 with explicit errors and does not report clean

### Requirement: Bounded repair
Repair SHALL operate only on the active phase's unambiguous backend-complete tasks through existing canonical transitions and boundaries.

#### Scenario: Inactive phase
- **WHEN** repair targets another phase
- **THEN** the command exits 2 before any write

#### Scenario: Failed transition
- **WHEN** a repair transition fails
- **THEN** dependent operations stop and actual state is rescanned before reporting failure

#### Scenario: Preserve history
- **WHEN** drift concerns archived artifacts, cancelled tasks or ledger-ahead state
- **THEN** it is reported for explicit resolution without changing history
