## ADDED Requirements

### Requirement: The loaded pack version is knowable at runtime

The pack SHALL emit a version manifest and UAR SHALL expose the loaded version, commit, and skill count without shelling out to git (impossible on mobile). Drift of the kind that went 359 commits undetected SHALL be visible through this surface.

#### Scenario: Drift becomes visible

- **GIVEN** the loaded pack is behind the manifest it was built from
- **WHEN** the provenance surface is queried
- **THEN** it reports a version distinguishable from the current one
