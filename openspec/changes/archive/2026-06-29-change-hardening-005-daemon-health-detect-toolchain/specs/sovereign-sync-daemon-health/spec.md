## ADDED Requirements

### Requirement: Daemon status command
The sovereign-sync binary SHALL provide a local status command for daemon health checks without starting a server.

#### Scenario: Status command reports healthy daemon
- **WHEN** `sovereign-sync --mode status --format json` connects to a healthy sovereign-sync `/health` endpoint
- **THEN** it returns a JSON report with status `healthy`
- **AND** it exits with code 0.

#### Scenario: Status command reports missing daemon
- **WHEN** no process is listening on the configured sovereign-sync port
- **THEN** the status command returns a report with status `missing`
- **AND** it exits with code 1.

#### Scenario: Status command reports occupied port
- **WHEN** a non-sovereign-sync process is listening on the configured sovereign-sync port
- **THEN** the status command returns a report with status `occupied`
- **AND** it exits with code 2
- **AND** it does not kill or modify the process using the port.

### Requirement: Toolchain diagnostic integration
The shared toolchain detector SHALL include sovereign-sync daemon health in its diagnostics.

#### Scenario: JSON diagnostics include daemon health
- **WHEN** `shared/scripts/detect-toolchain.sh --json` is run
- **THEN** the output includes `sovereign-sync-daemon`
- **AND** the status distinguishes `ok`, `missing`, and `occupied`.

#### Scenario: Text diagnostics identify occupied port
- **WHEN** the daemon port is occupied by a different service
- **THEN** text diagnostics display the sovereign-sync daemon entry as occupied rather than missing.

### Requirement: Health-state regression coverage
Daemon health detection SHALL have regression tests for healthy, missing, and occupied-port states.

#### Scenario: Rust health checker tests all states
- **WHEN** the sovereign-sync health-check tests are run
- **THEN** they cover healthy sovereign-sync response, missing listener, and non-sovereign occupied listener.

#### Scenario: Shell diagnostic fixture tests all states
- **WHEN** `shared/scripts/tests/test-detect-toolchain-sovereign-sync.sh` is run
- **THEN** it verifies healthy, missing, and occupied diagnostic mapping.
