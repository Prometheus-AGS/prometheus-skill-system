## ADDED Requirements

### Requirement: Report, update, and repair the pack

Cowork SHALL expose pack status, update, and repair. Update SHALL invoke the pack’s flat installer. Pack root resolution SHALL prefer PROMETHEUS_SKILL_PACK, then ~/.cowork/prometheus-skill-pack, then the local development fallback.

#### Scenario: Repair an installation with a broken symlink

- **GIVEN** the resolved pack has installed skills on multiple platforms and one damaged platform
- **WHEN** the operator inspects status and requests repair
- **THEN** the version and per-platform counts are reported and the installer repairs the affected platform
