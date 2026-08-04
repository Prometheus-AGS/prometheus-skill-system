# native-execution-sandbox Specification

## Purpose
TBD - created by archiving change change-exec-002-tier-p-sidecar. Update Purpose after archive.
## Requirements
### Requirement: Supported sandbox required
Tier P SHALL execute Python, Node, and Bash only through a supported OS sandbox selected before process creation. If the requested capabilities cannot be enforced, the run MUST be rejected as `tier_unavailable` and MUST NOT emit an attested execution receipt.

#### Scenario: Missing sandbox backend
- **WHEN** no supported sandbox is available on the host
- **THEN** the request fails before interpreter spawn and no receipt claims attestation

### Requirement: Default isolation and resource limits
The sandbox SHALL deny network by default, expose declared inputs read-only, expose only `outputs/` as writable, pass only allowed environment values, bound memory/time/output, and terminate the complete process group on timeout.

#### Scenario: Filesystem escape attempt
- **WHEN** sandboxed code tries to write outside the granted output directory
- **THEN** the OS sandbox denies the write and the receipt records the exact sandbox profile hash

#### Scenario: Wall-clock timeout
- **WHEN** a process exceeds `wallClockMs`
- **THEN** its process group is terminated and a terminal failed receipt records the timeout

### Requirement: Honest Tier P receipt
Every completed Tier P run SHALL produce an `attested` receipt containing the interpreter/toolchain hash, sandbox backend/profile hash, bounded output identities, resource usage, and executing-device signature.

#### Scenario: Successful Python run
- **WHEN** an approved Python request completes under Seatbelt or bwrap
- **THEN** the signed receipt verifies offline and all referenced outputs resolve through the CAS
