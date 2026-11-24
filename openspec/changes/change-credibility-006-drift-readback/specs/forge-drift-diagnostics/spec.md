## ADDED Requirements

### Requirement: Surface stale skills from recorded drift

Forge enrichment SHALL load drift reports for the selected language before skill resolution and warn about skills below 0.5 acceptance.

#### Scenario: Run with stale or missing drift input

- **WHEN** enrichment runs for a language with a recorded acceptance rate below 0.5 and then with no drift directory
- **THEN** the recorded case names the stale skill, and the missing-directory case proceeds without a drift warning
