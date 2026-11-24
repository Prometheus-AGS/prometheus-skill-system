## ADDED Requirements

### Requirement: Distribute DSG OpenCode and Codex registration artifacts

The pinned DSG source SHALL supply OpenCode package metadata with the plugin dependency and Codex configuration with a plugin marker and reserved MCP configuration.

#### Scenario: Inspect the pinned DSG source for harness registration

- **WHEN** an operator selects DSG for either supported harness
- **THEN** the matching metadata resolves from the pinned submodule and a reserved configuration does not claim an active MCP server
