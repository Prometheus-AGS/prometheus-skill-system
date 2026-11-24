## ADDED Requirements

### Requirement: Document and verify the installed Cowork upgrade

The Cowork command reference SHALL explain the pack upgrade path, with local evidence of cowork --version, cowork pack status, and cowork toolchain status for the v0.2.0 source build.

#### Scenario: Read back the upgraded Cowork installation

- **WHEN** the operator installs Cowork from the updated source and runs the documented commands
- **THEN** the CLI reports 0.2.0, pack status lists skill counts, and toolchain status reports readiness
