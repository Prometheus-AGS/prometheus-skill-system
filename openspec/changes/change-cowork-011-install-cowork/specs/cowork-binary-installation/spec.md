## ADDED Requirements

### Requirement: Install both Cowork command aliases

The managed binary installer SHALL install cowork and co, building from tools/cowork-skills/cli when present and using an applicable release download otherwise.

#### Scenario: Install from a checkout without the source submodule

- **GIVEN** the source directory is absent and a matching release artifact is available
- **WHEN** the managed installer reaches the Cowork component
- **THEN** it installs both executable command names from the release
