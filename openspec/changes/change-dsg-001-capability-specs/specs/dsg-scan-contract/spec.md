## ADDED Requirements

### Requirement: Define safe bounded scans

DSG SHALL sort scan results by decreasing size, report path, size, modification time, entry type and ecosystem, and continue past per-entry access failures. Default and deep scans SHALL bound directory depth to 8 and 20 levels.

#### Scenario: Scan a tree with restricted and linked entries

- **WHEN** a bounded scan finds a permission-denied entry and a symlink with a measurable target
- **THEN** the scanner continues past the restricted entry and can include the linked target’s size while recording the symlink identity
