## ADDED Requirements

### Requirement: Register the DSG tool as a pinned submodule

The pack SHALL register disk-space-guardian as a submodule at tools/disk-space-guardian and document its purpose and pin policy.

#### Scenario: Find the source of the managed DSG binary

- **WHEN** a contributor inspects the submodule configuration and documentation
- **THEN** the source is registered at the installer’s expected path and the pin policy is discoverable
