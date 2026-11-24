## ADDED Requirements

### Requirement: Comparable HTTP and browser examples

The TypeScript skill SHALL include a feature and step definitions for HTTP-only and Playwright-driven versions of the same business behavior.

#### Scenario: Author compares both example paths

- **GIVEN** an author opens the examples directory
- **WHEN** they compare the two demonstrated scenarios
- **THEN** the same business result is exercised through HTTP and through the browser, with guidance for choosing either
