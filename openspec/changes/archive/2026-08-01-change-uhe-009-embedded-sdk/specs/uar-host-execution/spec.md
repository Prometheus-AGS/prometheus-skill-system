## ADDED Requirements

### Requirement: An embedder uses a public API, not internals

UAR SHALL expose a public skill facade (list, get, install, toggle, query) consumable from an external crate, proven by an integration test in tests/ that uses only the public API. Runtime internals SHALL remain private.

#### Scenario: The facade is usable without reaching into internals

- **GIVEN** an external crate consumes the SDK
- **WHEN** it lists and toggles a skill
- **THEN** it does so without importing uar::runtime::skills internals
