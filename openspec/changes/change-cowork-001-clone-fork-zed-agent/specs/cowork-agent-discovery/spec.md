## ADDED Requirements

### Requirement: Discover Zed installation directories

Cowork SHALL detect Zed at ~/.config/zed or ~/.zed and support symlink installs, preferring ~/.config/zed/skills with ~/.zed/skills as fallback.

#### Scenario: Detect a legacy Zed installation

- **GIVEN** only ~/.zed exists
- **WHEN** Cowork enumerates installed agents
- **THEN** Zed appears as an available target and its fallback skill directory is selected
