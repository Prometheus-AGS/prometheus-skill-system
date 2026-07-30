## ADDED Requirements

### Requirement: Producer model is required, never synthesized

A creator SHALL refuse to dispatch an adversarial review when `KBD_PRODUCER_MODEL` is
unset, rather than substituting a default value. A synthesized producer identity would
cause the review to record `cross_model_check: verified-distinct` for a comparison that
never occurred.

#### Scenario: Unset producer model refuses dispatch

- **GIVEN** a creator about to dispatch an adversarial review
- **WHEN** `KBD_PRODUCER_MODEL` is unset or empty
- **THEN** the creator exits with status 2
- **AND** no findings artifact is written
- **AND** an explanatory refusal is emitted on stderr.

#### Scenario: Set producer model permits dispatch

- **GIVEN** a creator about to dispatch an adversarial review
- **WHEN** `KBD_PRODUCER_MODEL` names the model running the session
- **THEN** the review dispatches
- **AND** the findings artifact records `cross_model_check: verified-distinct`.

### Requirement: Single enforced sycophancy gate

`validate-skill.sh` SHALL be the single enforcement point for the sycophancy screen, and
SHALL invoke the existing `check-findings-sycophancy.sh` helper rather than reimplementing
its logic. Creators SHALL invoke `validate-skill.sh` and SHALL NOT call the helper directly.

#### Scenario: Sycophancy failure fails validation

- **GIVEN** a generated skill whose report contains no edge cases or failure modes
- **WHEN** `validate-skill.sh` runs
- **THEN** the sycophancy check group reports a failure
- **AND** the helper's non-zero exit is propagated into the `FAIL` counter
- **AND** the feedback appears in the `=== RESULT ===` block.

#### Scenario: Creators do not bypass the gate

- **GIVEN** a creator completing its Reflect phase
- **WHEN** it enforces the sycophancy screen
- **THEN** it invokes `validate-skill.sh`
- **AND** it does not invoke `check-findings-sycophancy.sh` directly.
