## ADDED Requirements

### Requirement: Skill creator dispatches an adversarial review

`pmpo-skill-creator` SHALL dispatch `/adversarial-review --mode skill` during its Reflect
phase, after `validate-skill.sh` has run and before the loop-decision table is evaluated.
Reviewing before validation would judge an artifact that is already known to be malformed;
reviewing after the loop decision would let a skill ship before it was judged.

#### Scenario: Review runs between validation and the loop decision

- **GIVEN** the skill creator reaching its Reflect phase
- **WHEN** the phase executes
- **THEN** `validate-skill.sh` runs first
- **AND** `/adversarial-review --mode skill` is dispatched next
- **AND** the loop-decision table is evaluated only after the review returns.

### Requirement: Sycophancy screen is an enforced validator check group

`validate-skill.sh` SHALL carry a check group that shells out to
`check-findings-sycophancy.sh`, propagate that helper's non-zero exit into its own `FAIL`
counter, and surface the helper's feedback in the existing `=== RESULT ===` block. A helper
whose exit is discarded is advisory, not a gate.

#### Scenario: A sycophantic report fails validation

- **GIVEN** a generated skill whose report names no edge cases or failure modes
- **WHEN** `validate-skill.sh` runs
- **THEN** the sycophancy check group reports a failure
- **AND** the `FAIL` counter is incremented
- **AND** `validate-skill.sh` exits non-zero
- **AND** the helper's feedback appears in the `=== RESULT ===` block.

#### Scenario: A clean report does not fail validation

- **GIVEN** a generated skill whose report states its limitations and failure modes
- **WHEN** `validate-skill.sh` runs
- **THEN** the sycophancy check group passes
- **AND** the `FAIL` counter is not incremented by that group.

### Requirement: CRITICAL findings block, bounded by a two-round retry

A CRITICAL finding SHALL block the skill creator from declaring the skill ready. The creator
SHALL re-review after a fix, SHALL stop after two rounds, and SHALL then append an
"Unresolved review findings" section rather than looping indefinitely or silently passing.

#### Scenario: A CRITICAL finding triggers a fix and re-review

- **GIVEN** a review returning a CRITICAL finding on the first round
- **WHEN** the creator applies a fix
- **THEN** it re-reviews the corrected artifact
- **AND** it does not declare the skill ready on the basis of the first review.

#### Scenario: Repeated CRITICALs stop at two rounds

- **GIVEN** a review returning a CRITICAL finding on both rounds
- **WHEN** the second round completes
- **THEN** the creator stops re-reviewing
- **AND** it appends an "Unresolved review findings" section naming the surviving findings
- **AND** it does not report the skill as clean.
