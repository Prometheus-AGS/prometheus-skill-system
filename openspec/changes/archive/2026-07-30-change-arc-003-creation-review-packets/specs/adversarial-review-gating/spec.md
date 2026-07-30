## ADDED Requirements

### Requirement: Skill review packet mode

`build-review-packet.sh` SHALL support `--mode skill`, producing a manifest-level packet for a
generated SKILL.md tree. The packet SHALL carry the `SKILL.md` body, its parsed frontmatter, an
inventory of `scripts/`, a cross-reference map of `references/` links, the output of the skill
validator, and the original generation intent. Full script source SHALL NOT be included — the
judge reviews the manifest, not the implementation.

#### Scenario: Skill packet carries the manifest surface

- **GIVEN** a generated skill directory containing `SKILL.md`, `scripts/`, and `references/`
- **WHEN** `build-review-packet.sh --mode skill` runs against it
- **THEN** the packet contains the `SKILL.md` body and its parsed frontmatter
- **AND** the packet contains a script inventory listing each file in `scripts/`
- **AND** the packet contains a cross-reference map of links into `references/`
- **AND** the packet contains the validator output and the original generation intent.

#### Scenario: Skill packet excludes full script source

- **GIVEN** a generated skill whose `scripts/` directory contains executable files
- **WHEN** the skill packet is built
- **THEN** the packet records each script's path and purpose
- **AND** the packet does not embed the full body of any script.

### Requirement: Agent review packet mode

`build-review-packet.sh` SHALL support `--mode agent`, producing a manifest-level packet for a
generated Cargo workspace. The packet SHALL carry `agent.toml`, `system_prompt.md`, the
workspace member list with a stated purpose per crate, the configured `mcp_servers`, and the
`cargo check` result. A generated workspace exceeds any judge's context window, so full source
SHALL NOT be included.

#### Scenario: Agent packet carries the workspace surface

- **GIVEN** a generated agent workspace
- **WHEN** `build-review-packet.sh --mode agent` runs against it
- **THEN** the packet contains `agent.toml` and `system_prompt.md`
- **AND** the packet lists each workspace member with its stated purpose
- **AND** the packet lists the configured `mcp_servers`
- **AND** the packet records the `cargo check` result.

#### Scenario: Agent packet excludes crate source

- **GIVEN** a generated workspace with multiple member crates
- **WHEN** the agent packet is built
- **THEN** the packet does not embed the source of any crate.

### Requirement: Truncation is recorded, never silent

Each creation packet SHALL be capped in size, and when the cap truncates content the packet
SHALL record that truncation occurred and what was dropped. A truncated packet that looks
complete would let a judge return `PASS` on material it never saw.

#### Scenario: Truncation is disclosed inside the packet

- **GIVEN** an artifact whose packet content exceeds the configured cap
- **WHEN** the packet is built
- **THEN** the packet contains a truncation record naming the cap and the omitted sections
- **AND** the truncation record is inside the packet the judge receives.

#### Scenario: An uncapped packet records no truncation

- **GIVEN** an artifact whose packet content fits within the cap
- **WHEN** the packet is built
- **THEN** the packet records no truncation
- **AND** the packet is not marked as partial.
