## ADDED Requirements

### Requirement: A git dependency without a rev is a latent break

A git dependency SHALL name an explicit rev. Absent one, cargo update floats to branch HEAD and re-resolves past the committed lockfile. This change SHALL request authorisation first; absent an explicit grant it SHALL be archived BLOCKED with no file modified.

#### Scenario: Silence blocks rather than proceeds

- **GIVEN** no authorisation has been granted for the surreal-memory-server repository
- **WHEN** the change runs
- **THEN** it is archived BLOCKED and no file in that repository is modified
