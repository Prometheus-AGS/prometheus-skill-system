## ADDED Requirements

### Requirement: Identity is host-independent
The canonical manifest SHALL record, per entry, a type in `{file, directory, symlink}`, a single normalized executable bit for files, a content hash, a size, and for symlinks the recorded target text. It SHALL NOT record permission modes, timestamps, ownership, security identifiers, file attributes, or access control lists. Payload bytes SHALL be canonicalized under RFC 8785 before hashing.

#### Scenario: Same payload on three hosts
- **WHEN** an identical payload is materialized on Linux, macOS, and Windows
- **THEN** all three compute a byte-identical bundle identity

#### Scenario: Differing umask
- **WHEN** two POSIX hosts with different umasks materialize the same payload
- **THEN** the computed identity is unchanged, because modes are re-applied from the manifest rather than observed

#### Scenario: Unsupported entry type
- **WHEN** a payload contains a device, socket, or other entry outside the permitted types
- **THEN** ingest fails rather than recording an approximation

### Requirement: Link entries hash their recorded target
A symlink entry SHALL contribute its recorded target text to the identity, never the bytes of whatever the host materialized.

#### Scenario: Degraded materialization
- **WHEN** a host without link support writes a link entry as a copy of its target
- **THEN** the entry still hashes as a link over its recorded target and identity is unchanged

### Requirement: Schema version is enforced asymmetrically
The verifier SHALL accept the prior manifest schema version for generations already installed. The creator SHALL refuse to emit it.

#### Scenario: Pre-existing generation
- **WHEN** a generation recorded under the prior schema version is verified
- **THEN** verification proceeds under the prior rules and the generation is not invalidated

#### Scenario: New generation
- **WHEN** a new generation is created
- **THEN** it is emitted under the current schema version and the pinned bundle identity is updated in the same change
