## ADDED Requirements

### Requirement: The librefang ABI fork is decided under review

The choice among port, keep-both, and retire SHALL be recorded via decision-log.sh with alternatives, a stated falsifier, and outcome_status pending, and SHALL pass decision-mode review with cross_model_check verified-distinct. No code SHALL be written.

#### Scenario: The decision carries a falsifier

- **GIVEN** the decision record is reviewed
- **WHEN** the falsifier field is read
- **THEN** it names a measurable condition that would reverse the choice
