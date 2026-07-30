## ADDED Requirements

### Requirement: Shared producer-model guard

The producer-model refusal SHALL be implemented once, as `kbd_require_producer_model()` in
`shared/scripts/lib/kbd-model-resolve.sh`, and SHALL be sourced by every creator entry point
that dispatches an adversarial review. A per-creator reimplementation would let the two
creators drift, so that one fails closed and the other silently fabricates a producer.

This requirement covers the *mechanism*. The behavioural contract it enforces —
"Producer model is required, never synthesized" — is already specified by
`change-arc-001-ratify-goal-wording` and is not restated here.

#### Scenario: Guard is defined in the shared resolver

- **GIVEN** the shared model-resolution library
- **WHEN** it is sourced
- **THEN** `kbd_require_producer_model` is defined as a shell function
- **AND** it is defined in exactly one file across the repository.

#### Scenario: Guard returns 2 without side effects

- **GIVEN** the shared library has been sourced
- **WHEN** `kbd_require_producer_model` is called with `KBD_PRODUCER_MODEL` unset or empty
- **THEN** it returns status 2
- **AND** it writes an explanatory refusal to stderr naming the variable
- **AND** it does not assign any default value to `KBD_PRODUCER_MODEL`.

#### Scenario: Guard is transparent when the producer is known

- **GIVEN** the shared library has been sourced
- **WHEN** `kbd_require_producer_model` is called with `KBD_PRODUCER_MODEL` set to a non-empty value
- **THEN** it returns status 0
- **AND** it writes nothing to stderr
- **AND** the value of `KBD_PRODUCER_MODEL` is unchanged.

#### Scenario: Both creator entry points invoke the guard

- **GIVEN** the skill creator and the agent creator
- **WHEN** either is about to dispatch an adversarial review
- **THEN** it has sourced `shared/scripts/lib/kbd-model-resolve.sh`
- **AND** it calls `kbd_require_producer_model` before building the review packet
- **AND** a non-zero return aborts the dispatch rather than being logged and ignored.
