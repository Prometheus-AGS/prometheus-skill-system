## ADDED Requirements

### Requirement: Merge Codex MCP and goals configuration

After installing Codex skills, Cowork SHALL merge the pack's MCP stanzas and goals.enabled into ~/.codex/config.toml and copy continuation.md and budget_limit.md to ~/.codex/goals.

#### Scenario: Repeat a Codex install with existing TOML

- **GIVEN** the operator has unrelated Codex configuration
- **WHEN** the installer is invoked twice
- **THEN** the existing configuration is preserved, each pack server is registered once, goals are enabled, and both prompt files are present
