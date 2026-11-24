## ADDED Requirements

### Requirement: Run research jobs through the CLI

The research CLI SHALL start a background job with a unique ID, persist its query, depth, source limit, citation style, PID, and progress in a checkpoint, read that checkpoint for status, and support cancellation by ID.

#### Scenario: Start, inspect, and cancel a job

- **WHEN** the operator starts an active research job and invokes status and cancel with its returned ID
- **THEN** start returns promptly, status reports checkpoint progress, and cancel signals the process and records cancelled
