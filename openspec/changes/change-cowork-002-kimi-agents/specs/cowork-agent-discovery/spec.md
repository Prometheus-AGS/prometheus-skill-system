## ADDED Requirements

### Requirement: Discover Kimi Code and Desktop independently

Cowork SHALL expose kimi-code and kimi-desktop as separate installation targets and detect each from its own home configuration directory. Kimi Code SHALL install under `~/.kimi-code/skills`. Kimi Desktop SHALL install under `~/Library/Application Support/kimi-desktop/daimon-share/daimon/skills`.

#### Scenario: Only Kimi Code is installed

- **GIVEN** the home contains ~/.kimi-code and no Desktop configuration
- **WHEN** Cowork enumerates installed agents
- **THEN** kimi-code is detected independently, while kimi-desktop remains an addressable target without appearing installed
