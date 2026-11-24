## ADDED Requirements

### Requirement: Discover MiniMax skill consumers

Cowork SHALL use ~/.minimax/skills for both MiniMax Code and Desktop and detect Desktop through ~/Library/Application Support/MiniMax Agent as a fallback.

#### Scenario: Desktop installation without CLI configuration

- **GIVEN** the Desktop application-support directory exists and ~/.minimax does not yet exist
- **WHEN** Cowork enumerates detected agents
- **THEN** minimax is available with the shared skills directory; an mmx media CLI alone does not make it available
