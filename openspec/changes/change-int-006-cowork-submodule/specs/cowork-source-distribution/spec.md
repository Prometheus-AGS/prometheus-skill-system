## ADDED Requirements

### Requirement: Record a Cowork gitlink at the installer source path

The pack SHALL track tools/cowork-skills as a registered gitlink, with its v0.1.5 historical pin and submodule documentation.

#### Scenario: Initialize the recorded Cowork source

- **WHEN** a fresh checkout initializes tools/cowork-skills
- **THEN** Git materializes the source at 53e6b31 and the managed installer can resolve its cli directory
