## ADDED Requirements

### Requirement: Include both managed tool submodules in pack checks

The pack’s strict development check SHALL account for initialized Cowork and DSG source checkouts and rebuild the marketplace links from the skills tree.

#### Scenario: Prepare a fresh development checkout

- **WHEN** a contributor initializes the pack and both named tool submodules
- **THEN** the marketplace rebuild uses all native entries and the applicable optional Rust checks can discover their Cargo manifests
