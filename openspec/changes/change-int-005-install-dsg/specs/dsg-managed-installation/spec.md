## ADDED Requirements

### Requirement: Install and report the DSG executable

The binary installer SHALL prefer a DSG source build and fall back to a matching release artifact. Toolchain health and essential command guidance SHALL include DSG.

#### Scenario: Install without an initialized source submodule

- **WHEN** the operator installs the managed binaries with a matching DSG release available
- **THEN** the installer obtains dsg from the release and toolchain health reports the installed executable
