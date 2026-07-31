## ADDED Requirements

### Requirement: storage-provider can target wasm32

`iroh-docs` SHALL be behind a feature flag rather than an unconditional dependency, and
the `iroh` floor SHALL be at least 1.0.2. An unconditional non-wasm dependency silently
forecloses every browser target.

#### Scenario: Native behaviour is unchanged

- **GIVEN** the default feature set
- **WHEN** `cargo build` runs
- **THEN** it succeeds
- **AND** IrohDocsAdapter remains available

#### Scenario: wasm32 progresses past iroh-docs

- **GIVEN** the feature disabled
- **WHEN** `cargo check --target wasm32-unknown-unknown` runs
- **THEN** compilation does not fail on `iroh-docs`
