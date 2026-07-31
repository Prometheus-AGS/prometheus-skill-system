## ADDED Requirements

### Requirement: Fabric decisions are recorded and checkable in-repo

Each fabric decision SHALL be recorded with its alternatives and rationale, and any edit
to a file outside this repository SHALL be evidenced by a `knowme_sync` block naming the
path, the external repository commit, and the resulting file hash — so a reviewer scoped to
this repository can verify it. No code SHALL land in the consuming repositories.

#### Scenario: Decisions are recorded with alternatives

- **GIVEN** the fabric decision set
- **WHEN** the records are read
- **THEN** each names the selected option and the rejected alternatives with rationale

#### Scenario: External edits are evidenced in-repo

- **GIVEN** an edit to the KnowMe integration guide
- **WHEN** the decision record is written
- **THEN** it contains a `knowme_sync` block with the external path, that repository's commit, and the guide hash

#### Scenario: No consuming repository receives code

- **GIVEN** this change complete
- **WHEN** the consuming repositories are inspected
- **THEN** no source change was made to flint-realtime-fabric, universal-agent-runtime, or know-me-system
