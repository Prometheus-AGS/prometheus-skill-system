## ADDED Requirements

### Requirement: Register the research daemon in the user domain

The macOS installer SHALL install the research binary and register com.prometheus.research in the user's launchd domain with MCP mode, the installed executable path, RunAtLoad, and KeepAlive.

#### Scenario: Activate the installed research service

- **WHEN** the user installs the binary and its LaunchAgent in an eligible macOS session
- **THEN** the service is registered to the installed user binary and the binary's MCP mode accepts stdio requests
