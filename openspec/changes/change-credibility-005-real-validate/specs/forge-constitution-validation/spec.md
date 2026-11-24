## ADDED Requirements

### Requirement: Enforce constitution decisions in CLI and MCP

Both Forge validation entry points SHALL run the loaded constitution against the input. The CLI SHALL exit 1 on error-severity findings and identify an absent constitution.

#### Scenario: Validate a file with a binding rule violation

- **WHEN** the operator checks a file containing an error-severity violation through the CLI and MCP entry points
- **THEN** both report the violation and the CLI exits 1
