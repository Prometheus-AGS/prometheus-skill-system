## ADDED Requirements

### Requirement: Waypoint defects are detected in this repo

A check SHALL exit non-zero when current-waypoint.json names a phase disagreeing with the active phase directory, or when next is self-referential. The fix itself SHALL NOT be applied to installed skills from this repository, because such edits are destroyed by the next install.

#### Scenario: The next occurrence is caught

- **GIVEN** the waypoint next field is self-referential
- **WHEN** the check runs
- **THEN** it exits non-zero naming the defect
