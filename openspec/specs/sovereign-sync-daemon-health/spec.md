# sovereign-sync-daemon-health Specification

## Purpose

Define deterministic local health classification and diagnostic requirements for the sovereign-sync daemon.

## Requirements

### Requirement: Daemon status command

The sovereign-sync binary SHALL provide a local status command for daemon health checks without starting a server, and managed KBD clients SHALL use the platform's private Unix control socket by default while honoring an explicit configured TCP endpoint.

#### Scenario: Status command reports healthy daemon

- **WHEN** `sovereign-sync --mode status --format json` connects to a healthy sovereign-sync `/health` endpoint
- **THEN** it returns a JSON report with status `healthy`
- **AND** it exits with code 0.

#### Scenario: Status command reports missing daemon

- **WHEN** neither the managed Unix socket nor an explicitly configured sovereign-sync endpoint is reachable
- **THEN** the status command returns a report with status `missing`
- **AND** it exits with code 1.

#### Scenario: Status command reports occupied port

- **WHEN** an explicit TCP endpoint is configured and a non-sovereign-sync process is listening there
- **THEN** the status command returns a report with status `occupied`
- **AND** it exits with code 2
- **AND** it does not kill or modify the process using the port.

#### Scenario: Managed KBD uses the private control socket

- **WHEN** the platform-managed sovereign-sync Unix control socket exists and no explicit control endpoint is configured
- **THEN** KBD status, diagnostics, audit, event, and mutation requests use that Unix socket.

#### Scenario: Explicit endpoint overrides managed discovery

- **WHEN** an operator configures an explicit sovereign-sync control endpoint
- **THEN** KBD control requests use that endpoint instead of the discovered managed Unix socket.

### Requirement: Toolchain diagnostic integration

The shared toolchain detector SHALL include sovereign-sync daemon health in its diagnostics, and KBD control diagnostics SHALL identify `kbd-runtime` as authority hosted by sovereign-sync rather than as a standalone daemon.

#### Scenario: JSON diagnostics include daemon health

- **WHEN** `shared/scripts/detect-toolchain.sh --json` is run
- **THEN** the output includes `sovereign-sync-daemon`
- **AND** the status distinguishes `ok`, `missing`, and `occupied`.

#### Scenario: Text diagnostics identify occupied port

- **WHEN** the daemon port is occupied by a different service
- **THEN** text diagnostics display the sovereign-sync daemon entry as occupied rather than missing.

#### Scenario: KBD authority diagnostics use the managed host

- **WHEN** doctor checks `control.kbd-runtime` and the managed Unix control socket is healthy
- **THEN** it reports KBD runtime authority health obtained through sovereign-sync.

#### Scenario: Unreachable diagnostics identify the hosting boundary

- **WHEN** doctor cannot reach KBD authority diagnostics
- **THEN** it reports that the authority is temporarily unreachable through sovereign-sync
- **AND** it does not instruct the operator to restart a standalone `kbd-runtime` daemon.

### Requirement: Health-state regression coverage

Daemon health detection SHALL have regression tests for healthy, missing, and occupied-port states.

#### Scenario: Rust health checker tests all states

- **WHEN** the sovereign-sync health-check tests are run
- **THEN** they cover healthy sovereign-sync response, missing listener, and non-sovereign occupied listener.

#### Scenario: Shell diagnostic fixture tests all states

- **WHEN** `shared/scripts/tests/test-detect-toolchain-sovereign-sync.sh` is run
- **THEN** it verifies healthy, missing, and occupied diagnostic mapping.

### Requirement: Managed KBD authority availability

The sovereign-sync daemon SHALL keep healthy registered KBD project authorities available when one or more other registered projects cannot be opened, and it SHALL identify each failed registration without exposing key material.

#### Scenario: A stale registration does not disable healthy projects

- **WHEN** daemon startup opens at least one registered KBD project and one or more other registrations fail
- **THEN** the daemon installs routes for every successfully opened project
- **AND** requests for those healthy projects remain readable and writable.

#### Scenario: Failed registration is diagnosable

- **WHEN** a registered project cannot be opened during daemon startup
- **THEN** local diagnostics identify the failed project and its concrete open error
- **AND** no signer secret or private key material is logged.

#### Scenario: Managed mutations use an enrolled signer

- **WHEN** a local interactive KBD command targets the default canonical platform data root
- **THEN** it signs with the managed device identity enrolled by sovereign-sync
- **AND** the daemon accepts the mutation when that identity remains authorized.

#### Scenario: Custom data roots remain hermetic

- **WHEN** a KBD runtime uses a non-default data root
- **THEN** it does not implicitly load the platform-managed signer.

### Requirement: Supervised restart continuity

The managed sovereign-sync service SHALL recover its private control endpoint after a supervised restart without requiring KBD callers to switch to a legacy TCP default.

#### Scenario: Control availability returns after restart

- **WHEN** the platform supervisor restarts sovereign-sync
- **THEN** the managed Unix health endpoint becomes healthy again
- **AND** a signed KBD mutation succeeds after the restart.

#### Scenario: Partial registry failure persists across restart

- **WHEN** stale and healthy registrations coexist during consecutive supervised starts
- **THEN** the same healthy project authorities remain available on each successful start
- **AND** the stale registrations remain isolated as project-specific failures.
