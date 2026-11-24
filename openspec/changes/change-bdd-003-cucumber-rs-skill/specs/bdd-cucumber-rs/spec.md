## ADDED Requirements

### Requirement: Asynchronous Rust BDD guidance

The pack SHALL document Cucumber 0.23 with an async World, Tokio and reqwest HTTP steps, thirtyfour 0.37 browser steps, and migration from Cucumber 0.20.

#### Scenario: Author updates a legacy Cucumber project

- **GIVEN** an author starts from the older async-trait setup
- **WHEN** they follow the migration and browser references
- **THEN** they can identify the async-trait removal and Rust version floor, implement native async steps, and compare the documented alternative browser drivers
