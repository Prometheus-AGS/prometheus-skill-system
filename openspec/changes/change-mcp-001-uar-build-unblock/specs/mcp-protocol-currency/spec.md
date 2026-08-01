## ADDED Requirements

### Requirement: The build is protected by a constraint, not only by a lockfile

universal-agent-runtime SHALL declare a sse-stream floor of 0.2.4 in its own dependencies, and SHALL compile with tests passing. A lockfile alone SHALL NOT be treated as protection, because it records a resolution rather than constraining a future one.

#### Scenario: A downgrade below the floor is refused

- **GIVEN** the sse-stream floor is declared
- **WHEN** cargo update is asked for 0.2.2
- **THEN** cargo refuses it as violating the requirement
