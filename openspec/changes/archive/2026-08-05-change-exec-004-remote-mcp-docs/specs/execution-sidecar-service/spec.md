## MODIFIED Requirements

### Requirement: Health-first UDS service
The daemon SHALL atomically bind a mode-`0600` UDS and serve static `/health` before subsystem initialization. `/ready` SHALL report bounded per-subsystem state, local connections SHALL be accepted only from the daemon UID, and optional MCP or remote subsystem failures SHALL NOT delay health or prevent otherwise-ready local execution.

#### Scenario: Slow state initialization
- **WHEN** durable state initialization is delayed
- **THEN** `/health` remains responsive while `/ready` reports the delayed subsystem as not ready

#### Scenario: Slow remote initialization
- **WHEN** the estate remote adapter is enabled but transport initialization is slow or unavailable
- **THEN** `/health` remains responsive and `/ready` reports remote degradation separately from local execution readiness

### Requirement: Non-mutating doctor
`prometheus-exec doctor` SHALL inspect binary identity, socket permissions, peer credentials, key availability, Tier P and Tier W backend availability, component trust state, state reconciliation, CAS, optional MCP schema parity, and configured remote queue state without installing, starting, stopping, compiling components, contacting excluded services, consuming queues, or rewriting service state.

#### Scenario: Missing Seatbelt binary
- **WHEN** doctor runs on macOS without the configured sandbox executable
- **THEN** it reports Tier P unavailable and exits non-zero without attempting repair

#### Scenario: Pulley unavailable in mobile profile
- **WHEN** doctor inspects a mobile build without the required Pulley backend
- **THEN** it reports Tier W unavailable and exits non-zero without attempting repair or engine initialization

#### Scenario: Remote service excluded
- **WHEN** remote diagnosis is configured but `service:sovereign-sync` is excluded
- **THEN** doctor reports the remote check as excluded without constructing a client or contacting the service
