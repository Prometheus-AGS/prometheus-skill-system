## ADDED Requirements

### Requirement: Filesystem primitives are probed, never inferred
The installer SHALL determine symlink, junction, hardlink, and executable-bit support by attempting each operation in the generation store root and observing the result. Support SHALL NOT be inferred from the reported platform, and the probe SHALL run in the store root because capability varies by volume.

#### Scenario: Executable bit is unsupported
- **WHEN** toggling the owner execute bit and re-stating the file reports an unchanged mode
- **THEN** executable-bit support is recorded as absent and the manifest, not the filesystem, becomes the authority for the executable bit

#### Scenario: Store root is on a different volume than the probe default
- **WHEN** the store root and the temporary directory are on volumes with different capabilities
- **THEN** the recorded capability reflects the store root

### Requirement: Probe results are cached and invalidated
Probe results SHALL be persisted and reused across invocations, and SHALL be discarded when the installer version changes.

#### Scenario: Installer is upgraded
- **WHEN** a cached capability record was written by a prior installer version
- **THEN** the record is discarded and the probe re-runs before materialization

### Requirement: Degradation is recorded out of band
When a capability is absent and a payload entry is materialized by a weaker primitive, the installer SHALL record the substitution in a materialization record stored alongside the manifest. That record SHALL NOT contribute to generation identity.

#### Scenario: A link is materialized as a copy
- **WHEN** no link primitive is available and a link entry is written as a copy
- **THEN** the substitution is recorded with the entry path, the intended primitive, and the realized primitive, and generation identity is unchanged
