## ADDED Requirements

### Requirement: Provide research actions over MCP stdio

The research MCP stdio server SHALL expose research_start, research_status, research_cancel, research_export, and render_component. Start, status and cancel SHALL preserve the CLI job identity and checkpoint semantics. The initial export and component-rendering registrations SHALL be discoverable with their documented interim behavior.

#### Scenario: Round-trip an MCP research job

- **WHEN** an MCP client starts research and sends status and cancel for the returned job ID
- **THEN** the server returns checkpoint stage data and acknowledges the cancellation for the same job
