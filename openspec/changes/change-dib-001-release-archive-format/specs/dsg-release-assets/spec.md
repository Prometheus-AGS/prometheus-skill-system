## ADDED Requirements

### Requirement: Package DSG binaries as versioned archives

The DSG release pipeline SHALL package its target binary under the versioned `dsg-<semver>-<target>.tar.gz` asset name expected by the pack installer.

#### Scenario: Handle the v0.1.1 release tag

- **WHEN** the release packaging step builds the native target
- **THEN** the release contains the matching versioned tarball with an extractable executable
