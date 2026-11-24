# judge-gateway-availability Specification

## Purpose
Record how the review judge gateway is supplied, keep proxy availability independent of the main build, and report availability explicitly.
## Requirements
### Requirement: The openai-proxy vendoring decision is recorded

A written decision SHALL select vendoring, a sibling reference with a doctor check, or optional vendoring for the `openai-proxy` judge gateway. It SHALL record rejected alternatives and cite the `liter-llm` installation failure.

#### Scenario: Prior dependency failure informs the decision

- **WHEN** the decision explains the risk of a mandatory dependency
- **THEN** it records that the unbuildable `liter-llm` submodule made `cargo metadata` exit 101, aborting `install-binaries.sh` and leaving 7 of 14 binaries stale

#### Scenario: The decision names the choice and the rejected alternatives

- **GIVEN** the recorded decision
- **WHEN** it is read
- **THEN** it names the selected option
- **AND** it names the alternatives that were rejected and why
- **AND** it cites the `liter-llm` mid-run install failure as evidence.

### Requirement: Vendoring does not make the build depend on the proxy

If `openai-proxy` is vendored, it SHALL be added without making the build depend on it. A
required submodule that fails to build would abort the installer for every user, including
those who never invoke a judge.

#### Scenario: A missing or unbuildable proxy does not abort installation

- **GIVEN** an environment where the vendored proxy is absent or fails to build
- **WHEN** the binary installer runs
- **THEN** the installer completes
- **AND** the remaining binaries are installed
- **AND** the proxy's unavailability is reported rather than raised as a fatal error.

### Requirement: Judge-gateway availability is reported explicitly

`prometheus doctor` SHALL report judge-gateway availability as its own check. Silent
degradation to harness-native review is the failure this phase exists to eliminate, and it is
invisible unless something reports it.

#### Scenario: An available gateway is reported

- **GIVEN** a reachable judge gateway
- **WHEN** `prometheus doctor` runs
- **THEN** it reports the gateway as available
- **AND** it names the endpoint that answered.

#### Scenario: An unavailable gateway is reported as a distinct failure

- **GIVEN** no reachable judge gateway
- **WHEN** `prometheus doctor` runs
- **THEN** it reports judge-gateway availability as failing
- **AND** it states that reviews would degrade to same-model self-review
- **AND** it does not report the check as passing or as not-implemented.

