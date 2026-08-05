# execution-sidecar-service Specification

## Purpose
TBD - created by archiving change change-exec-002-tier-p-sidecar. Update Purpose after archive.
## Requirements
### Requirement: Restart-safe idempotent runs
The service SHALL atomically persist request identity before spawn and terminal receipt before response. Same-ID/same-hash replay SHALL return the existing run/receipt; same-ID/different-hash SHALL return conflict, including after restart.

#### Scenario: Response-loss reconciliation
- **WHEN** a run finishes but the caller loses the response and resubmits the same signed request
- **THEN** the service returns the original run and receipt without a second execution

### Requirement: Health-first UDS service
The daemon SHALL atomically bind a mode-`0600` UDS and serve static `/health` before subsystem initialization. `/ready` SHALL report bounded per-subsystem state, local connections SHALL be accepted only from the daemon UID, and optional MCP or remote subsystem failures SHALL NOT delay health or prevent otherwise-ready local execution.

#### Scenario: Slow state initialization
- **WHEN** durable state initialization is delayed
- **THEN** `/health` remains responsive while `/ready` reports the delayed subsystem as not ready

#### Scenario: Slow remote initialization
- **WHEN** the estate remote adapter is enabled but transport initialization is slow or unavailable
- **THEN** `/health` remains responsive and `/ready` reports remote degradation separately from local execution readiness

### Requirement: REST and event retrieval
The sidecar SHALL expose Tier P and Tier W run creation, run status, resumable ordered events, terminal receipt retrieval, artifact retrieval, verified-replay results, `/health`, and `/ready` through versioned APIs backed by the same service layer.

#### Scenario: Event resume
- **WHEN** a client reconnects with `after=<sequence>`
- **THEN** it receives only later events in strictly increasing sequence order

#### Scenario: Tier W event resume
- **WHEN** a Tier W client reconnects with `after=<sequence>`
- **THEN** it receives only later events in strictly increasing sequence order from the same durable lifecycle used by Tier P

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

### Requirement: Offline verified replay
The verifier SHALL optionally re-execute a Tier W receipt from its authorized component and inputs, compare the deterministic receipt projection and all output hashes, and remain transport- and service-independent.

#### Scenario: Replay output mismatch
- **WHEN** re-execution produces any different deterministic field or output byte
- **THEN** verification fails with the exact mismatched subject and never downgrades the receipt to attested success
