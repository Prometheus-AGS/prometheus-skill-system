# sovereign-sync-ci Specification

## Purpose

Defines the CI coverage required to keep sovereign-sync substrate crates formatted, linted, tested, and free of committed Cargo build artifacts.

## Requirements

### Requirement: Sovereign sync Rust CI

The repository SHALL provide a GitHub Actions workflow that validates the Rust substrate crates used by sovereign sync.

#### Scenario: CI runs for sovereign sync crate changes

- **GIVEN** a pull request or push changes files under `substrate/storage-provider`, `substrate/sovereign-sync`, or `substrate/sovereign-client`
- **WHEN** GitHub Actions evaluates the workflow triggers
- **THEN** the sovereign-sync Rust workflow runs for the changed branch.

#### Scenario: CI checks Rust formatting

- **GIVEN** the sovereign-sync Rust workflow is running
- **WHEN** the formatting job executes for each relevant crate
- **THEN** it runs `cargo fmt --check` from that crate's directory.

#### Scenario: CI checks Rust clippy diagnostics

- **GIVEN** the sovereign-sync Rust workflow is running
- **WHEN** the lint job executes for each relevant crate
- **THEN** it runs `cargo clippy --all-targets --all-features` from that crate's directory.

#### Scenario: CI runs Rust tests

- **GIVEN** the sovereign-sync Rust workflow is running
- **WHEN** the test job executes for each relevant crate
- **THEN** it runs `cargo test --all-targets --all-features` from that crate's directory.

### Requirement: Sovereign sync build artifacts are ignored

The repository SHALL ignore generated Cargo build output for substrate crates.

#### Scenario: Local validation creates target directories

- **GIVEN** a contributor runs Cargo commands under `substrate/*`
- **WHEN** Cargo creates `target` directories
- **THEN** those directories are ignored by git.
