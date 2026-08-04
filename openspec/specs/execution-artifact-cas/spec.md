# execution-artifact-cas Specification

## Purpose
TBD - created by archiving change change-exec-002-tier-p-sidecar. Update Purpose after archive.
## Requirements
### Requirement: Atomic content-addressed outputs
The artifact store SHALL identify content by SHA-256, install blobs atomically without overwrite, reject hash disagreement, and never follow an output symlink outside the run root.

#### Scenario: Duplicate content
- **WHEN** two runs store identical bytes
- **THEN** both receipts reference the same immutable CAS identity without rewriting the blob

### Requirement: Pin-aware budget enforcement
Garbage collection SHALL honor the configured byte budget while never collecting blobs referenced by unarchived receipts, open certifications, or explicit pins.

#### Scenario: Budget pressure with pinned blob
- **WHEN** the store exceeds budget and its oldest blob is pinned
- **THEN** collection skips that blob and removes only eligible unpinned content
