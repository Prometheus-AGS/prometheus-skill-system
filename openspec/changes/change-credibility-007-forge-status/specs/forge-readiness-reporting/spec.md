## ADDED Requirements

### Requirement: Expose Forge capabilities and gated features

Forge status SHALL report constitutions, skills, drift reports, knowledge service configuration, and active versus gated features. Stub output and help SHALL identify experimental functionality and its prerequisites.

#### Scenario: Inspect an experimental command

- **WHEN** the operator requests status and invokes a stub optimization command
- **THEN** status lists configured surfaces and the experimental command explains its required feature or service
