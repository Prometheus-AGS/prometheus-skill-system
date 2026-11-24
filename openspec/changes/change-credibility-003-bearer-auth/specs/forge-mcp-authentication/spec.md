## ADDED Requirements

### Requirement: Require a bearer token for MCP requests

Forge SHALL protect /mcp with a generated startup token or FORGE_MCP_TOKEN and leave /health available to unauthenticated health probes.

#### Scenario: Check both service routes

- **WHEN** a caller sends an unauthenticated MCP request, an authenticated MCP request, and a health probe
- **THEN** the first receives 401, the configured token admits the second, and the health probe receives 200 without a token
