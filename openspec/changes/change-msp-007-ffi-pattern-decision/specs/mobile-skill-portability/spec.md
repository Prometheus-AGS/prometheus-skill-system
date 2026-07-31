## ADDED Requirements

### Requirement: The FFI pattern is decided under adversarial review

The choice SHALL be recorded via decision-log.sh with alternatives, a stated falsifier, and outcome_status pending, and SHALL pass decision-mode review with cross_model_check verified-distinct. No FFI code SHALL be written in this change.

#### Scenario: The decision carries a falsifier

- **GIVEN** the decision record is reviewed
- **WHEN** the falsifier field is read
- **THEN** it names a measurable condition that would reverse the choice
