## ADDED Requirements

### Requirement: Publish the standalone DSG Claude plugin

The DSG source SHALL include a valid .claude-plugin/plugin.json and the pack marketplace SHALL list the plugin from the corresponding pinned revision.

#### Scenario: Discover the plugin through the marketplace

- **WHEN** a user selects Disk Space Guardian in the Claude marketplace
- **THEN** the entry resolves to the DSG plugin manifest and identifies its CLI
