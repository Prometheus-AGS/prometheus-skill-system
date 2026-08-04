## ADDED Requirements

### Requirement: Versioned portable execution envelopes
The system SHALL define transport-independent, camel-case JSON representations for signed execution requests, execution receipts, capabilities, limits, code and input identities, provenance, grants, events, artifacts, executing devices, usage, exits, and public error envelopes. Every signed envelope SHALL carry `schemaVersion`, and an unsupported version MUST fail closed.

#### Scenario: Supported schema round trip
- **WHEN** a revision-1 request or receipt is serialized and deserialized
- **THEN** all semantically present fields and enum values are preserved without transport-specific metadata

#### Scenario: Unknown schema version
- **WHEN** a verifier receives an envelope whose `schemaVersion` is not supported
- **THEN** it rejects the envelope before hashing or signature acceptance with a machine-readable version error

### Requirement: Canonical request identity
The system SHALL compute a signed request's identity as lowercase SHA-256 over RFC 8785 canonical JSON with only the top-level `signature` member omitted. The request ID SHALL be a UUID idempotency key and SHALL remain distinct from the canonical request hash.

#### Scenario: Equivalent object ordering
- **WHEN** two request JSON documents contain the same data with different object-member ordering
- **THEN** they produce the same canonical bytes and request hash

#### Scenario: Signed field mutation
- **WHEN** any signed request field is changed while the signature is retained
- **THEN** request signature verification fails

### Requirement: Signed receipt identity
Every execution receipt SHALL record the request hash, evidence class, concrete tier, code and input identities, sandbox profile identity, backend, exit, output identities, usage, timestamps, executing device, grants, and an algorithm-tagged signature. The signature SHALL cover RFC 8785 canonical receipt bytes with only the top-level `signature` member omitted.

#### Scenario: Valid Ed25519 receipt
- **WHEN** a revision-1 receipt is signed by the Ed25519 key named by its executing device
- **THEN** the portable verifier accepts the cryptographic signature and returns the canonical receipt hash

#### Scenario: Receipt field mutation
- **WHEN** any signed receipt field is mutated after signing
- **THEN** verification fails even if the receipt remains syntactically valid JSON

### Requirement: Signature algorithm agility
The contract SHALL discriminate `ed25519` and `p256` signatures from revision 1, SHALL bind the key ID to the algorithm and public-key fingerprint, and SHALL reject key, algorithm, or encoding mismatches. Ed25519 signatures SHALL use 64 raw bytes; P-256 signatures SHALL use fixed-width IEEE P1363 bytes and compressed SEC1 public keys, encoded as unpadded base64url.

#### Scenario: Algorithm mismatch
- **WHEN** a receipt declares `p256` but is presented with an Ed25519 key or signature
- **THEN** verification fails before reporting cryptographic success

#### Scenario: P-256 software fixture
- **WHEN** a receipt is signed by a revision-1 P-256 software test key using the defined encodings
- **THEN** the portable verifier accepts it and derives the expected algorithm-tagged key ID

### Requirement: Honest evidence classification
Tier W receipts SHALL use the `verified` evidence class. Tier P receipts SHALL use the `attested` evidence class. Revision 1 SHALL reject all other tier/evidence combinations and SHALL never represent an unsandboxed native process as attested.

#### Scenario: Invalid class and tier pairing
- **WHEN** a receipt claims Tier P with `verified` evidence or Tier W with `attested` evidence
- **THEN** receipt validation fails with an evidence-class invariant error

### Requirement: Stable public contract artifacts
The system SHALL generate deterministic JSON Schema and OpenAPI 3.1 components from the Rust contract types. Repeated generation from an unchanged revision SHALL be byte-for-byte identical.

#### Scenario: Deterministic generation
- **WHEN** contract artifacts are generated twice from the same source revision
- **THEN** the resulting files have identical bytes and hashes
