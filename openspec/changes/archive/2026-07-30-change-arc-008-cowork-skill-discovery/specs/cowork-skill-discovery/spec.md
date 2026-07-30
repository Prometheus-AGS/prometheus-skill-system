## ADDED Requirements

### Requirement: Skill discovery is available as a build-vs-adopt input during ideation

A discovery step SHALL be invocable from ideation, running `cowork search <capability>` with a
result limit, and SHALL surface its results as a build-vs-adopt input mirroring the shape of
`library-candidates.json`. `cowork search`, `generate`, and `install` already exist, so this
is wiring rather than new capability.

#### Scenario: Discovery surfaces candidates for a capability

- **GIVEN** an ideation session that has identified a needed capability
- **WHEN** the discovery step runs for that capability
- **THEN** `cowork search` is invoked with a bounded result limit
- **AND** the results are presented as adopt candidates alongside the build option.

#### Scenario: No results leaves the build path intact

- **GIVEN** a capability for which `cowork search` returns nothing
- **WHEN** the discovery step completes
- **THEN** the session proceeds with the build option
- **AND** no candidate is fabricated.

### Requirement: Discovered skills are never auto-installed

A discovered third-party skill SHALL be proposed, never installed automatically. Any adoption
SHALL be routed through `cowork audit` and `cowork verify` first. Auto-installing a skill
found by search would execute unreviewed third-party code from an ideation step.

#### Scenario: Adoption requires audit and verify

- **GIVEN** a discovered candidate the operator wants to adopt
- **WHEN** adoption proceeds
- **THEN** `cowork audit` and `cowork verify` run against the candidate first
- **AND** installation occurs only after the operator accepts.

#### Scenario: Discovery alone installs nothing

- **GIVEN** a discovery step returning several candidates
- **WHEN** the step completes without operator action
- **THEN** no skill has been installed
- **AND** no file outside the session's own notes has been modified.

### Requirement: cowork is documented on the site

`docs/guide/16a-cowork.md` SHALL document the `cowork` subcommands, stating the never-auto-
install security posture up front rather than in a trailing caveat. The tool currently has a
full skill but only one passing site mention.

#### Scenario: The page documents the subcommands and posture

- **GIVEN** the published guide
- **WHEN** the cowork page is read
- **THEN** it covers the `cowork` subcommands
- **AND** it states the audit-and-verify-before-adopt posture before describing installation.

#### Scenario: The site builds without broken links

- **GIVEN** the documentation site including the new page
- **WHEN** the site is built
- **THEN** the build succeeds
- **AND** it reports zero broken links.
