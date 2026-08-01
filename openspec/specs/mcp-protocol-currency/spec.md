# mcp-protocol-currency Specification

## Purpose
TBD - created by archiving change change-mcp-001-uar-build-unblock. Update Purpose after archive.
## Requirements
### Requirement: The build is protected by a constraint, not only by a lockfile

universal-agent-runtime SHALL declare a sse-stream floor of 0.2.4 in its own dependencies, and SHALL compile with tests passing. A lockfile alone SHALL NOT be treated as protection, because it records a resolution rather than constraining a future one.

#### Scenario: A downgrade below the floor is refused

- **GIVEN** the sse-stream floor is declared
- **WHEN** cargo update is asked for 0.2.2
- **THEN** cargo refuses it as violating the requirement

### Requirement: A git dependency without a rev is a latent break

A git dependency SHALL name an explicit rev. Absent one, cargo update floats to branch HEAD and re-resolves past the committed lockfile. This change SHALL request authorisation first; absent an explicit grant it SHALL be archived BLOCKED with no file modified.

#### Scenario: Silence blocks rather than proceeds

- **GIVEN** no authorisation has been granted for the surreal-memory-server repository
- **WHEN** the change runs
- **THEN** it is archived BLOCKED and no file in that repository is modified

### Requirement: Recorded findings are re-verified, not merely written

Each recorded finding SHALL carry a reproduction command re-run at write time. The existence of a decision record SHALL NOT by itself satisfy this requirement, because a record of false findings would pass such a check.

#### Scenario: A stale finding fails the check

- **GIVEN** a recorded finding no longer reproduces
- **WHEN** its command is re-run at write time
- **THEN** the finding is corrected rather than recorded as-is

