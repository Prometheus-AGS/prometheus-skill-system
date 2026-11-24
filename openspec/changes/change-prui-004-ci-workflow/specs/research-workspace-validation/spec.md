## ADDED Requirements

### Requirement: Provide scoped research crate checks

The research crate SHALL have formatting, lint with warnings denied, and applicable test entry points scoped to substrate/prometheus-research/Cargo.toml.

#### Scenario: Check a local research crate change

- **WHEN** a maintainer follows the crate’s scoped validation procedure
- **THEN** each command targets the research manifest and an applicable failed check blocks a complete result
