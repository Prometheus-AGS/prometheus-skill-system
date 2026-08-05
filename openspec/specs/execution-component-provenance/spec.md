# execution-component-provenance Specification

## Purpose
TBD - created by archiving change change-exec-003-tier-w-mobile. Update Purpose after archive.
## Requirements
### Requirement: Pre-instantiation component authorization
Component bytes SHALL be authorized before validation, compilation, caching, linking, or instantiation. Estate mode SHALL require an active signed-generation manifest entry whose component hash and capability metadata match the installed bytes and whose generation has consistent signed target receipts; standalone and bundled-mobile modes SHALL require an exact configured SHA-256 pin.

#### Scenario: Modified component
- **WHEN** component bytes differ from the authorized manifest or configured digest
- **THEN** loading fails before engine work and no receipt claims execution

#### Scenario: Inconsistent target receipt
- **WHEN** the active generation's local component is valid but a required target receipt binds different bytes or metadata
- **THEN** release certification fails without invalidating historical receipts from previously certified generations

### Requirement: Immutable provenance binding
A Tier W receipt SHALL bind the exact component digest, authorization mode, immutable generation identity when applicable, engine/backend identity, and capability-manifest hash.

#### Scenario: Generation rollback
- **WHEN** the active plugin pointer rolls back to an older signed generation
- **THEN** newly submitted runs accept only that generation while historical receipts continue to verify against their recorded immutable provenance

### Requirement: Trust and cache coherence
Compiled/interpreted component caches SHALL be keyed by the authorized component digest and engine configuration, and every load SHALL re-check current authorization before using a cached artifact.

#### Scenario: Revoked active generation
- **WHEN** a cached component is absent from the newly active signed generation
- **THEN** the cache entry is not instantiated even if its compiled bytes remain on disk
