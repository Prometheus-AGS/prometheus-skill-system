# judge-gateway-availability Specification

## Purpose
TBD - created by archiving change change-arc-009-openai-proxy-vendoring-decision. Update Purpose after archive.
## Requirements
### Requirement: The openai-proxy vendoring decision is recorded

A written decision SHALL record whether `openai-proxy` is vendored, referenced as a sibling
with a doctor check, or vendored-but-optional, together with the rejected alternatives. It is
what `kbd-judge` resolves to, and it is currently a referenced sibling rather than a tracked
dependency, so its absence degrades every review to harness-native.

The decision SHALL cite the `liter-llm` precedent observed in this repository: an unbuildable
submodule caused `cargo metadata` to exit 101, which aborted `install-binaries.sh` mid-run and
left 7 of 14 binaries stale.

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

