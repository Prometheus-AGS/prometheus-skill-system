## ADDED Requirements

### Requirement: Expose research jobs and event streams over HTTP

The research HTTP service SHALL expose health, job creation, checkpoint status, cancellation, component routes, static assets, and `text/event-stream` updates on the configured port, defaulting to 7891. Surface bridge emission failures SHALL leave local jobs and event delivery available.

#### Scenario: Subscribe to an active research job

- **WHEN** a client starts a job against the HTTP service and subscribes to its event route
- **THEN** the response is text/event-stream and includes emitted AG-UI events scoped to that job ID
