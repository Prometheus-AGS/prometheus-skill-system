## ADDED Requirements

### Requirement: Expose a research binary and library scaffold

The substrate SHALL provide a prometheus-research binary and library with MCP, server, daemon, and status modes and configuration for port, job directory, and surface bridge URL.

#### Scenario: Inspect available research modes

- **WHEN** the operator invokes prometheus-research --help
- **THEN** usage identifies the supported modes and permits selecting the server with port 7891
