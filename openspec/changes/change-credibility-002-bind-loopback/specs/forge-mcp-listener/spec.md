## ADDED Requirements

### Requirement: Bind Forge MCP to loopback by default

Forge serve SHALL default to 127.0.0.1 and accept an explicit bind address. Selecting an all-interface bind SHALL produce a warning.

#### Scenario: Compare default and explicit binds

- **WHEN** the operator starts Forge first with defaults and then with an explicit all-interface bind
- **THEN** the default listener is loopback-only and the explicit selection is acknowledged on stderr
