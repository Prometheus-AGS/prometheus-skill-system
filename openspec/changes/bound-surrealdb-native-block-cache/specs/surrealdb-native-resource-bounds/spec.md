## Purpose

Keep the owned local SurrealDB RocksDB block cache within a stable budget on
shared agent workstations.

## ADDED Requirements

### Requirement: The native RocksDB block cache is explicitly bounded

The managed SurrealDB launchd and systemd services SHALL set the documented
`SURREAL_ROCKSDB_BLOCK_CACHE_SIZE` environment variable to 1,073,741,824 bytes.
The installed service definition MUST preserve that value across reinstall and
service restart.

#### Scenario: SurrealDB starts on a high-memory workstation
- **WHEN** the managed native database starts with RocksDB storage
- **THEN** its log reports a 1 GiB block-cache size
- **AND** it does not derive a half-host-memory cache from the workstation size

#### Scenario: The skill pack is reinstalled
- **WHEN** service templates are rendered for launchd or systemd
- **THEN** both installed service definitions retain the same explicit cache limit
