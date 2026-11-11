## ADDED Requirements

### Requirement: Operators can enroll teammates' devices
The CLI SHALL let an active operator enroll another machine's signing key for the project.

#### Scenario: Teammate enrolled as a device
- **WHEN** the teammate runs `prometheus kbd device identity` and the operator runs
  `prometheus kbd device enroll --public-key <b64> --device-id <name> --command-id <id>`
- **THEN** the teammate's harness writes (decisions, phases, tasks) are accepted

### Requirement: Operators can grant co-operator authority
The runtime SHALL record `OperatorGranted` only when signed by an active operator, for an
enrolled active device that is not already an operator.

#### Scenario: Co-operator controls the run lifecycle
- **WHEN** the operator runs `prometheus kbd device enroll ... --operator`
- **THEN** the teammate can run `prometheus kbd pause` and `prometheus kbd resume`

#### Scenario: Non-operator cannot grant
- **WHEN** a non-operator device attempts `device grant-operator`
- **THEN** the command is refused and no event is appended

#### Scenario: Revocation removes authority
- **WHEN** the operator runs `prometheus kbd device revoke --key-id <id> --reason <text>`
- **THEN** later writes signed by that key are refused

#### Scenario: Enrolled device revises the plan
- **WHEN** a device enrolled without `--operator` runs `prometheus kbd revise --reason <text>`
- **THEN** the revision is accepted (plan revision is a harness command), but `pause`, `resume`,
  `run start`, `cancel` and `device grant-operator` are refused
