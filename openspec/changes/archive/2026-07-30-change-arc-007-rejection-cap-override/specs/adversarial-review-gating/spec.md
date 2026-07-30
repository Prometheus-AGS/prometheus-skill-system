## ADDED Requirements

### Requirement: The sycophancy-screen cap is user-overridable within a hard ceiling

`check-findings-sycophancy.sh` SHALL read its rejection cap from
`PROMETHEUS_ADV_REJECT_CAP`, defaulting to 2, following the same pattern as
`PROMETHEUS_REFLECT_STRICTNESS`. A hardcoded cap decides on the operator's behalf in contexts
the author of the constant never saw.

The override SHALL be bounded by a hard ceiling of 5. A value above the ceiling SHALL be an
error rather than being clamped or ignored, so a typo cannot silently disable the bound.

This requirement governs the **sycophancy-screen** cap only. The two-round retry cap in the
skill and agent creators is a separate bound and is unchanged.

#### Scenario: The default cap is unchanged

- **GIVEN** `PROMETHEUS_ADV_REJECT_CAP` is unset
- **WHEN** the sycophancy screen runs
- **THEN** the cap is 2.

#### Scenario: A value within the ceiling is honoured

- **GIVEN** `PROMETHEUS_ADV_REJECT_CAP` is set to a value between 1 and 5
- **WHEN** the sycophancy screen runs
- **THEN** that value is used as the cap.

#### Scenario: A value above the ceiling is an error

- **GIVEN** `PROMETHEUS_ADV_REJECT_CAP` is set above 5
- **WHEN** the sycophancy screen runs
- **THEN** it exits non-zero with an error naming the ceiling
- **AND** it does not silently fall back to the default or to the ceiling.

#### Scenario: The creator retry cap is unaffected

- **GIVEN** `PROMETHEUS_ADV_REJECT_CAP` is set to 5
- **WHEN** a creator's review retry loop runs
- **THEN** the retry loop still stops after two rounds.

### Requirement: An overridden cap is recorded in the findings artifact

When the cap is overridden, the findings artifact SHALL record `cap_overridden: true` and the
value used. An override that leaves no trace makes a lenient run indistinguishable from a
strict one after the fact.

#### Scenario: An override is recorded

- **GIVEN** a run with `PROMETHEUS_ADV_REJECT_CAP` set above the default
- **WHEN** the findings artifact is written
- **THEN** it records `cap_overridden: true`
- **AND** it records the cap value used.

#### Scenario: A default run is recorded as not overridden

- **GIVEN** a run with `PROMETHEUS_ADV_REJECT_CAP` unset
- **WHEN** the findings artifact is written
- **THEN** `cap_overridden` is false or absent.

### Requirement: The override prompt never blocks non-interactive execution

When the cap is about to be exceeded in an interactive terminal, the screen MAY prompt once,
defaulting to accept. It SHALL NOT prompt when stdin is not a TTY, so CI runs and hooks can
never hang waiting for input.

#### Scenario: A non-interactive run does not prompt

- **GIVEN** the screen running with stdin not attached to a TTY
- **WHEN** the cap is reached
- **THEN** no prompt is emitted
- **AND** the run proceeds to completion without waiting for input.

#### Scenario: An interactive run prompts at most once

- **GIVEN** the screen running in an interactive terminal
- **WHEN** the cap is reached
- **THEN** at most one prompt is emitted
- **AND** the default answer is to accept.
