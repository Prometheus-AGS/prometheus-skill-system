## ADDED Requirements

### Requirement: Advance the Cowork gitlink to v0.2.0

The pack SHALL update tools/cowork-skills to the commit referenced by its v0.2.0 tag.

#### Scenario: Initialize the updated Cowork source

- **WHEN** the checked-out pack initializes its Cowork gitlink
- **THEN** the submodule commit matches the v0.2.0 tag rather than the earlier 53e6b31 revision
