## ADDED Requirements

### Requirement: Document live research binary entry points

The parent research skill SHALL explain service readiness, MCP job tools, HTTP job events, and component rendering. It SHALL distinguish the macOS login service from manually running the HTTP server.

#### Scenario: Use the service from the skill instructions

- **WHEN** an operator wants to inspect a job's live progress and render a fragment
- **THEN** the skill gives the job event path and render_component tool and explains how to start the relevant service mode
