## ADDED Requirements

### Requirement: Builtin skills are visibly undeletable

The admin UI SHALL make builtin skills visually distinguishable and their delete affordance absent or disabled. A delete control that returns 409 SHALL NOT satisfy this requirement.

#### Scenario: No dead delete button

- **GIVEN** a builtin skill is shown in the admin UI
- **WHEN** the user looks for a delete control
- **THEN** it is absent or disabled rather than present and failing
