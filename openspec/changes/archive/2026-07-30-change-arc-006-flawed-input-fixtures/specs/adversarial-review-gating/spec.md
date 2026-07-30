## ADDED Requirements

### Requirement: The gate is proven to discriminate, not merely to run

Committed fixtures SHALL prove that the review gate distinguishes flawed artifacts from clean
ones. A test asserting only that a review completes would have passed throughout the period in
which eight consecutive reviews were same-model self-grades that all returned `PASS`.

Four fixtures SHALL be committed: `flawed-skill`, `flawed-agent`, `clean-skill`, `clean-agent`.

#### Scenario: Flawed fixtures are blocked and cross-model verified

- **GIVEN** the `flawed-skill` and `flawed-agent` fixtures
- **WHEN** each is submitted to the review gate
- **THEN** the verdict is `BLOCK`
- **AND** the findings artifact records `cross_model_check: verified-distinct`.

#### Scenario: Clean fixtures pass

- **GIVEN** the `clean-skill` and `clean-agent` fixtures
- **WHEN** each is submitted to the review gate
- **THEN** the verdict is `PASS`.

#### Scenario: An inverted result fails the suite

- **GIVEN** a run in which a flawed fixture passes or a clean fixture is blocked
- **WHEN** the suite evaluates the results
- **THEN** the suite exits non-zero
- **AND** the inversion is named in the failure output.

### Requirement: Fail-closed behaviour is asserted per creator

The suite SHALL assert the producer-model refusal at both creator entry points, since a guard
that exists but is unsourced by one creator fails silently exactly where it matters.

#### Scenario: Each creator refuses with the producer model unset

- **GIVEN** the skill creator and the agent creator
- **WHEN** each is invoked with `KBD_PRODUCER_MODEL` unset
- **THEN** each exits with status 2
- **AND** no findings file is written by either
- **AND** each emits a refusal on stderr.

### Requirement: The retry bound is asserted per creator

The suite SHALL assert that repeated CRITICAL findings terminate at two rounds for both
creators and that the "Unresolved review findings" section is appended.

#### Scenario: Both creators stop at two rounds

- **GIVEN** a fixture that yields a CRITICAL finding on every round
- **WHEN** each creator processes it
- **THEN** each stops after two review rounds
- **AND** each appends an "Unresolved review findings" section.

### Requirement: The suite is bounded and does not run on every commit

The suite SHALL cap live judge calls at six per run and SHALL execute on demand and at the
release gate only. An unbounded cross-model suite on every commit would make the gate too
expensive to keep enabled, and a disabled gate proves nothing.

#### Scenario: Judge calls stay within the ceiling

- **GIVEN** a full run of the fixture suite
- **WHEN** the run completes
- **THEN** no more than six live judge calls were issued.

#### Scenario: The suite does not run on ordinary commits

- **GIVEN** an ordinary commit to the repository
- **WHEN** the standard checks run
- **THEN** the fixture suite is not invoked
- **AND** it remains invocable on demand and at the release gate.
