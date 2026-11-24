## ADDED Requirements

### Requirement: Define scanner, cleaner, and cache commands

DSG SHALL specify scan, clean, and caches commands with human-readable and JSON outputs, and define exit status 0 for success, 1 for error, and 2 for dry-run candidates.

#### Scenario: Preview cleanup

- **WHEN** an operator runs clean without force and candidates exist
- **THEN** DSG returns 2 and reports candidates without deleting files
