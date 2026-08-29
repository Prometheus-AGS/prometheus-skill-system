## Purpose

Defines reliable KBD lifecycle-event mirroring and prior-context recall through the supported local surreal-memory service contract.

## ADDED Requirements

### Requirement: KBD discovers the installed local memory service
KBD SHALL prefer explicit tool, environment, and project configuration hints, and SHALL probe the canonical local service when no explicit hint is present.

#### Scenario: Canonical local service is healthy
- **WHEN** no memory endpoint override is configured and the canonical local health route succeeds
- **THEN** KBD treats memory as available and uses the canonical server origin for REST calls

#### Scenario: Explicit endpoint override is configured
- **WHEN** a supported environment or project configuration endpoint is present
- **THEN** KBD uses that endpoint ahead of the canonical local default and normalizes any MCP path to the server origin

### Requirement: Lifecycle hooks use the supported entity contract
KBD SHALL write lifecycle events as valid entities with string observations through the current entity REST route and SHALL remain non-blocking when the write fails.

#### Scenario: Hook event is mirrored
- **WHEN** a KBD hook fires while the memory service is reachable
- **THEN** the service receives a lifecycle entity whose observation preserves project, phase, kind, edge, index, total, source tool, and timestamp metadata

#### Scenario: Memory write fails
- **WHEN** the entity endpoint rejects or cannot receive the hook event
- **THEN** the hook reports a bounded diagnostic and does not block the KBD operation

### Requirement: Recall produces relevant project context
KBD SHALL retrieve lifecycle entities through a supported search route, rank same-project and phase-relevant events ahead of unrelated events, and write a deterministic top-five digest.

#### Scenario: Relevant lifecycle events exist
- **WHEN** recall runs for a phase and matching events exist
- **THEN** `prior-context.md` lists at most five ranked prior events and summarizes their event kinds

#### Scenario: Reachable service has no matching events
- **WHEN** recall reaches the service but no usable lifecycle event is returned
- **THEN** `prior-context.md` states that no prior matches were found without claiming the endpoint is unreachable

#### Scenario: Memory service is unreachable
- **WHEN** no configured or canonical endpoint passes its bounded health probe
- **THEN** recall writes the documented unreachable stub and exits successfully
