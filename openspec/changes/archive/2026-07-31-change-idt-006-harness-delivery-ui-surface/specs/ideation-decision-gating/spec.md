## ADDED Requirements

### Requirement: Delivery is verified on a non-Claude harness

The ideation flow SHALL emit a `UiIntent` rather than rendering directly, and Tier 1
delivery SHALL be verified by running it on one named non-Claude harness. Tier 0 text is a
floor, not evidence of harness delivery.

#### Scenario: Tier 1 round trip completes on a non-Claude harness

- **GIVEN** the flow running on a named non-Claude harness
- **WHEN** it emits a UiIntent
- **THEN** `__ui_intent__.json` is written
- **AND** a `__ui_response__.json` placed there is consumed within the timeout
- **AND** the flow continues using that response

#### Scenario: Tier 0 remains a working floor

- **GIVEN** a harness resolving to `tier0_text`
- **WHEN** the flow runs
- **THEN** it completes in text
