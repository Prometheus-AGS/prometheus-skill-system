# execution-remote-dispatch Specification

## Purpose
TBD - created by archiving change change-exec-004-remote-mcp-docs. Update Purpose after archive.
## Requirements
### Requirement: Estate-only remote dependency boundary
R-class dispatch SHALL live in an `exec-remote` crate enabled only by the estate feature. Contracts, core, Tier P, Tier W, standalone, and bundled-mobile profiles SHALL NOT depend on Sovereign Sync, KBD, or any remote transport crate.

#### Scenario: Standalone dependency graph
- **WHEN** the standalone execution profile is compiled with estate disabled
- **THEN** no remote, Sovereign Sync, or KBD dependency is selected and local execution remains functional

### Requirement: Enrolled-peer signed dispatch
Every remote dispatch SHALL bind the canonical execution request hash, dispatch ID, origin endpoint, target endpoint, issue time, validity window, and signer identity in a signed envelope. The adapter SHALL authorize the endpoint/signing-key pair against an injected enrollment snapshot before queueing or execution.

#### Scenario: Unknown endpoint
- **WHEN** a validly signed dispatch names an endpoint absent from the enrollment snapshot
- **THEN** it is rejected before queue insertion or execution and no receipt claims a run

#### Scenario: Signer mismatch
- **WHEN** the endpoint is enrolled but the envelope signature does not match its bound signing key
- **THEN** the dispatch is rejected with an authorization error and no mutable remote state is advanced

### Requirement: Durable store-and-forward and replay defense
The origin SHALL persist an immutable dispatch record before delivery, deduplicate dispatch IDs and request IDs across restart, enforce request validity at acceptance and execution time, and reconcile response loss from durable peer receipts without re-execution.

#### Scenario: Offline target resumes
- **WHEN** a target is unavailable and later reconnects within the request validity window
- **THEN** the queued dispatch is delivered once and its returned terminal receipt closes the durable origin record

#### Scenario: Expired queued request
- **WHEN** a queued request reaches a target after its validity window
- **THEN** the target records a terminal rejection and never starts a local run

#### Scenario: Duplicate delivery after response loss
- **WHEN** the origin retries a delivered dispatch after losing the response
- **THEN** the target returns the existing local run or terminal receipt without executing again

### Requirement: Per-peer receipt aggregation
Remote dispatch status SHALL preserve one independently verifiable state and receipt reference per target. An aggregate SHALL distinguish received, running, applied, rejected, expired, unavailable, and pending-evidence targets without replacing peer receipts with a synthetic success.

#### Scenario: Mixed target outcome
- **WHEN** one target returns a verified receipt and another rejects authorization
- **THEN** aggregate status exposes both outcomes and cannot report universal success

### Requirement: Remote availability isolation
Remote transport initialization, slow peers, and unavailable external evidence SHALL never delay local `/health`, local execution, or offline receipt verification. Remote runtime certification SHALL remain `pending_evidence` until a disposable multi-peer battery is executed.

#### Scenario: Slow remote transport startup
- **WHEN** remote transport initialization exceeds the readiness budget
- **THEN** `/health` and local execution remain available while `/ready` reports only the remote subsystem as degraded
