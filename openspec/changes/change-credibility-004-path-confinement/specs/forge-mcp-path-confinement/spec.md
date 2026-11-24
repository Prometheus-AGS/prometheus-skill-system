## ADDED Requirements

### Requirement: Confine enrichment reads to the project

The forge_enrich handler SHALL canonicalize the requested task path before verifying that it is under the configured project root. Invalid paths SHALL produce a JSON-RPC error.

#### Scenario: Enrich a task with an invalid path

- **WHEN** a task resolves outside the project root
- **THEN** the handler rejects the request before reading the task and returns a JSON-RPC error
