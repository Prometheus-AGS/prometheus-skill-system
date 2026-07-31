## ADDED Requirements

### Requirement: Analysis is withheld until the user commits

The flow SHALL refuse to emit analysis until a user judgement is recorded, and output
SHALL carry a machine-checkable confidence field, a non-empty `what_would_change_this`
field, and at least one disconfirming item. Confidence claims that cannot be checked are
not acceptable substitutes.

#### Scenario: Analysis is refused without a prior judgement

- **GIVEN** a decision flow with no recorded user judgement
- **WHEN** analysis is requested
- **THEN** the flow refuses
- **AND** no analysis is emitted

#### Scenario: Output carries checkable calibration fields

- **GIVEN** a completed decision analysis
- **WHEN** its artifact is validated
- **THEN** `confidence` is present
- **AND** `what_would_change_this` is non-empty
- **AND** at least one disconfirming item is present
