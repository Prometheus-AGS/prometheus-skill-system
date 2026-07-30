## ADDED Requirements

### Requirement: Agent creator dispatches an adversarial review before declaring ready

The native-agent generator SHALL dispatch `/adversarial-review --mode agent` at the end of
`generate.md`, after `cargo check` and `npm install` have run and before the readiness banner
is emitted. `cargo check` proves the workspace compiles; it says nothing about whether the
agent is any good, so a banner emitted before the review would declare an unjudged artifact
ready.

#### Scenario: Review runs after build steps and before the banner

- **GIVEN** the agent generator completing a workspace
- **WHEN** generation reaches its final stage
- **THEN** `cargo check` and `npm install` have already run
- **AND** `/adversarial-review --mode agent` is dispatched next
- **AND** the readiness banner is emitted only after the review returns.

### Requirement: The review blocks the readiness declaration, not the workspace

A blocking review SHALL suppress only the readiness declaration. The generated workspace
SHALL persist on disk regardless of the verdict, so the operator can inspect and repair what
was flagged rather than losing the work.

#### Scenario: A CRITICAL finding suppresses the banner but keeps the workspace

- **GIVEN** a generated workspace whose review returns a CRITICAL finding
- **WHEN** the generator finishes
- **THEN** the readiness banner is not emitted
- **AND** the generated workspace remains on disk
- **AND** the findings artifact is written alongside it.

#### Scenario: A clean review permits the banner

- **GIVEN** a generated workspace whose review returns no CRITICAL findings
- **WHEN** the generator finishes
- **THEN** the readiness banner is emitted
- **AND** the workspace remains on disk.

### Requirement: Agent creator uses the same bounded retry as the skill creator

The agent creator SHALL apply the same two-round retry loop as the skill creator, appending
an "Unresolved review findings" section when CRITICAL findings survive both rounds. One
retry policy across both creators keeps the bound from drifting between them.

#### Scenario: Repeated CRITICALs stop at two rounds

- **GIVEN** an agent review returning a CRITICAL finding on both rounds
- **WHEN** the second round completes
- **THEN** the creator stops re-reviewing
- **AND** it appends an "Unresolved review findings" section
- **AND** the readiness banner is not emitted.
