# execution-certification-evidence Specification

## Purpose
TBD - created by archiving change change-exec-004-remote-mcp-docs. Update Purpose after archive.
## Requirements
### Requirement: Receipt-backed evidence references
Certification records SHALL resolve a cited execution run to its signed receipt, request hash, public verification identity, referenced content-addressed artifacts, and verification result without requiring the generating session or a running daemon.

#### Scenario: Independently verifiable use case
- **WHEN** a certification cites a completed execution run
- **THEN** a verifier with only the archived bundle and public key can validate the receipt and every cited artifact hash

### Requirement: Honest pending-evidence semantics
A required but unavailable execution environment, remote peer, physical device, or receipt bundle SHALL produce `pending_evidence`, not pass or fail. Judge unavailability SHALL remain the distinct `pending_review` state.

#### Scenario: Remote peers unavailable
- **WHEN** the certification profile requires a two-peer remote run but no isolated peers can be executed
- **THEN** remote certification is recorded as `pending_evidence` while independently completed local requirements retain their status

### Requirement: Method-independent certification
Certification SHALL bind requirements to independently verifiable evidence properties and SHALL NOT require `prometheus-exec`, Bash, Python, or any other specific production method when equivalent evidence satisfies the declared contract.

#### Scenario: Equivalent external evidence
- **WHEN** a producer supplies independently verifiable evidence matching every declared evidence property without using `prometheus-exec`
- **THEN** certification evaluates that evidence under the same rules rather than rejecting it because of the producing tool

### Requirement: Evidence status separation
Release evidence SHALL separately report artifact/source certification, disposable runtime certification, installed-host state, remote deployment evidence, mobile size status, and physical-device status.

#### Scenario: Desktop pass with mobile blocker
- **WHEN** desktop Tier W passes but the mobile size budget or physical-device run is unavailable
- **THEN** the report marks desktop evidence complete and mobile release readiness blocked or pending without collapsing them into one green status
