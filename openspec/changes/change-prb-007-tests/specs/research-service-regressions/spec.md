## ADDED Requirements

### Requirement: Exercise job lifecycle, MCP, and streaming regressions

The historical eight-case research regression suite SHALL cover checkpoint creation, status and cancellation, MCP responses, and SSE headers and event delivery. Stream scenarios SHALL use ephemeral ports and deterministic synchronization.

#### Scenario: Observe event delivery at the research HTTP boundary

- **WHEN** the SSE integration starts a server on an ephemeral port and synchronizes subscription before emitting an event
- **THEN** it verifies the stream content type and receives the broadcast event using a bounded synchronization mechanism
