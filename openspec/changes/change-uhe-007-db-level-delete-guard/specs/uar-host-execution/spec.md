## ADDED Requirements

### Requirement: Builtin deletion is refused at the database

A DELETE of a Builtin skill SHALL fail at the database layer. A guard present only in SkillService SHALL NOT satisfy this requirement, because a caller reaching the storage provider directly bypasses it.

#### Scenario: The bypass route is closed

- **GIVEN** a caller invokes the storage provider directly, bypassing SkillService
- **WHEN** it deletes a Builtin skill
- **THEN** the delete is refused
