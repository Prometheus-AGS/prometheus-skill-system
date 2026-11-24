## ADDED Requirements

### Requirement: Check in the npm resolution

The pack SHALL track package-lock.json and use npm ci for locked dependency installation.

#### Scenario: Install a clean checkout

- **WHEN** the dependency installer runs against the committed manifest and lockfile
- **THEN** npm ci installs the recorded dependency graph without regenerating the lock
