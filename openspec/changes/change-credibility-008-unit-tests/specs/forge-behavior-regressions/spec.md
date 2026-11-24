## ADDED Requirements

### Requirement: Preserve historical Forge regression coverage

The historical Forge regression suite SHALL contain at least 15 checks and cover skill loading, constitution checking, drift-path parsing, task construction, language detection, metric extraction, and registry operations. Embedded doctest examples SHALL parse as Rust.

#### Scenario: Read the recorded check surface

- **WHEN** a maintainer inspects the source and historical test inventory
- **THEN** the seven behavior areas are represented and the documentation example uses valid Rust syntax
