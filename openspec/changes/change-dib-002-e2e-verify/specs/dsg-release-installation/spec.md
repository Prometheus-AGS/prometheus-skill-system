## ADDED Requirements

### Requirement: Verify the managed DSG installation end to end

The DSG v0.1.1 delivery SHALL provide the four named target artifacts and record local end-to-end installation with the expected DSG version.

#### Scenario: Install on a supported local machine

- **WHEN** the pack binary installer runs against the intended DSG release or pinned source
- **THEN** the installer completes and the installed dsg --version resolves to the release under qualification
