## ADDED Requirements

### Requirement: Immutable hash-linked receipt segments
The system SHALL represent archived execution receipts as immutable segments containing a versioned header, sequence number, previous-segment hash, creation time, ordered entries, and a segment hash over canonical segment content excluding only the segment-hash field.

#### Scenario: Valid linked segment
- **WHEN** a segment's previous hash matches the verified prior segment and all entry and segment hashes match
- **THEN** segment verification succeeds and returns its canonical segment hash

#### Scenario: Broken archive link
- **WHEN** a segment declares a previous hash different from the expected verified predecessor
- **THEN** verification fails with a chain-link error

### Requirement: Receipt entry integrity
Every segment entry SHALL record the canonical hash of its embedded receipt. Segment verification SHALL recompute each receipt hash, validate its signed receipt using an explicit key resolver, preserve entry order, and confirm the header's receipt count.

#### Scenario: Embedded receipt replacement
- **WHEN** an embedded receipt is replaced without updating the segment entry
- **THEN** verification fails at the entry hash before the segment is accepted

#### Scenario: Missing signer key
- **WHEN** an entry names an executing device for which the caller supplied no trusted public key
- **THEN** verification fails closed and identifies the unresolved key ID

### Requirement: Independent segment verification
A receipt-log segment SHALL be verifiable without KBD journal types, service state, or a running execution engine. The format SHALL impose explicit maximum segment, entry, and receipt sizes.

#### Scenario: Standalone archive inspection
- **WHEN** an exported segment and explicit signer keys are supplied to the contracts library
- **THEN** it can validate the segment and receipts with no estate dependencies

#### Scenario: Oversized segment
- **WHEN** an untrusted segment exceeds a revision-1 size or entry-count limit
- **THEN** verification rejects it before unbounded allocation or recursive processing
