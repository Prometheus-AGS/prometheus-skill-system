## ADDED Requirements

### Requirement: Enforce the documented cleanup safety rules

DSG SHALL check open files, repository dirtiness, modification age, and exclusions before moving an eligible item to Trash. It SHALL protect its own binary and config directory and log each attempted cleanup.

#### Scenario: Handle a symlink or per-item Trash failure

- **WHEN** a cleanup batch contains a symlink and an eligible item that fails to move to Trash
- **THEN** the linked target is preserved, the failure is logged, and subsequent eligible entries are still considered
