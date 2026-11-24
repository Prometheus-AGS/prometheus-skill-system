## ADDED Requirements

### Requirement: Portable Cucumber JavaScript guidance

The pack SHALL ship a project-agnostic bdd-cucumber-js authoring skill covering Cucumber 13, playwright-bdd 9.2, and the tsx runner. The former bdd-testing entry SHALL direct users to the new skill.

#### Scenario: Existing project finds portable guidance

- **GIVEN** a project opens the historical bdd-testing entry
- **WHEN** its author follows the redirect
- **THEN** the new skill provides Cucumber profiles and helps select HTTP-only or Playwright steps without requiring Next.js
