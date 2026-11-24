## ADDED Requirements

### Requirement: Map research events to UI intents

The research emitter SHALL map agent.status to progress, agent.message and agent.error to feedback, and a2ui.component to prompt. Each intent SHALL carry title, body, a job-ID request_id, empty options, and multiselect false.

#### Scenario: Deliver the research renderer through Tier 2

- **WHEN** the renderer emits an a2ui.component event
- **THEN** the bridge receives a prompt intent with the component name and serialized properties, and the documented MCP-to-iframe flow describes actual fragment rendering
