## ADDED Requirements

### Requirement: Install the pinned first-release DSG build

The pack SHALL pin tools/disk-space-guardian to the v0.1.0 release revision and support a source-built executable reporting that version.

#### Scenario: Install the release from source

- **WHEN** the managed binary install uses the initialized release submodule
- **THEN** the executable is installed in the user's binary directory and reports 0.1.0
