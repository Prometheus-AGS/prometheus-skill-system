## ADDED Requirements

### Requirement: Define Forge workspace checks

The Forge Rust workspace validation SHALL cover formatting, lint with warnings treated as errors, and the applicable test targets, scoped to tools/forge-rs/Cargo.toml.

#### Scenario: Catch a format defect in Forge

- **WHEN** a local Forge formatting check examines an intentionally misformatted source in an isolated copy
- **THEN** it fails in the Forge workspace before the documented validation can be recorded as passing
