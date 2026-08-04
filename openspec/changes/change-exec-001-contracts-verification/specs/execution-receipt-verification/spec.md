## ADDED Requirements

### Requirement: Offline verification
`prometheus-exec verify` and the underlying library SHALL validate execution evidence without starting a daemon, opening a listening socket, initializing runtime storage, contacting KBD, contacting Sovereign Sync, or making a network request.

#### Scenario: Verification with networking unavailable
- **WHEN** a caller supplies a receipt, its explicit public key, and local artifacts on a host with networking denied
- **THEN** verification completes with the same result as on a connected host

### Requirement: Layered receipt validation
Receipt verification SHALL check schema support, key identity, signature, request hash when a request is supplied, terminal-state consistency, evidence-class/tier compatibility, timestamp ordering, output hash syntax, artifact path safety, and artifact bytes when supplied. The result SHALL distinguish cryptographic, semantic, and artifact failures.

#### Scenario: Signature valid but artifact corrupted
- **WHEN** a receipt signature is valid but a supplied artifact's bytes do not match its recorded SHA-256 hash
- **THEN** overall verification fails and identifies the artifact mismatch

#### Scenario: Wrong request paired with receipt
- **WHEN** a valid receipt is verified with a different signed request
- **THEN** overall verification fails because the canonical request hash does not match

### Requirement: Safe artifact resolution
Artifact verification SHALL accept only relative normalized paths rooted below `outputs/` and SHALL reject absolute paths, parent traversal, symlink escape, and duplicate conflicting paths before reading artifact content.

#### Scenario: Parent traversal reference
- **WHEN** a receipt contains an artifact path such as `outputs/../../secret`
- **THEN** verification fails without opening the referenced host path

### Requirement: Side-effect-bounded identity initialization
`prometheus-exec init` SHALL atomically create an Ed25519 identity with filesystem mode `0600`, SHALL refuse silent replacement, and SHALL print public identity information without printing private key bytes. A failure SHALL not leave a partial identity file.

#### Scenario: First initialization
- **WHEN** initialization targets an absent identity path
- **THEN** exactly one valid identity is atomically installed with mode `0600` and its public key ID is returned

#### Scenario: Existing identity
- **WHEN** initialization targets an existing identity without an explicit replacement option
- **THEN** the command fails without modifying the existing file

### Requirement: Machine-readable verification result
The verification API SHALL return a deterministic result containing overall validity, canonical receipt hash when computable, verified checks, and structured failures with stable codes. Human-readable CLI output SHALL be derived from the same result.

#### Scenario: Automated consumer detects failure
- **WHEN** verification fails and JSON output is requested
- **THEN** the command returns a non-zero exit status and a structured failure code without translating empty or erroneous output into success
