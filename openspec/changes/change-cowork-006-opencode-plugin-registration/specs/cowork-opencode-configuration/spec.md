## ADDED Requirements

### Requirement: Register the pack OpenCode plugin

After installing OpenCode skills, Cowork SHALL merge the absolute pack .opencode path into the user's plugin array and maintain the pack's plugin, SDK, and zod dependencies.

#### Scenario: Preserve existing OpenCode plugin entries

- **GIVEN** the user already has other plugins registered
- **WHEN** Cowork installs the OpenCode target twice
- **THEN** existing plugins remain, the pack path appears once, and package dependencies are available
