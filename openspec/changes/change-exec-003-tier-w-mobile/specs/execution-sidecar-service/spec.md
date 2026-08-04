## MODIFIED Requirements

### Requirement: REST and event retrieval
The sidecar SHALL expose Tier P and Tier W run creation, run status, resumable ordered events, terminal receipt retrieval, artifact retrieval, verified-replay results, `/health`, and `/ready` through versioned APIs backed by the same service layer.

#### Scenario: Tier W event resume
- **WHEN** a Tier W client reconnects with `after=<sequence>`
- **THEN** it receives only later events in strictly increasing sequence order from the same durable lifecycle used by Tier P

### Requirement: Non-mutating doctor
`prometheus-exec doctor` SHALL inspect binary identity, socket permissions, peer credentials, key availability, Tier P and Tier W backend availability, component trust state, state reconciliation, CAS, and readiness without installing, starting, stopping, compiling components, or rewriting service state.

#### Scenario: Pulley unavailable in mobile profile
- **WHEN** doctor inspects a mobile build without the required Pulley backend
- **THEN** it reports Tier W unavailable and exits non-zero without attempting repair or engine initialization

## ADDED Requirements

### Requirement: Offline verified replay
The verifier SHALL optionally re-execute a Tier W receipt from its authorized component and inputs, compare the deterministic receipt projection and all output hashes, and remain transport- and service-independent.

#### Scenario: Replay output mismatch
- **WHEN** re-execution produces any different deterministic field or output byte
- **THEN** verification fails with the exact mismatched subject and never downgrades the receipt to attested success
