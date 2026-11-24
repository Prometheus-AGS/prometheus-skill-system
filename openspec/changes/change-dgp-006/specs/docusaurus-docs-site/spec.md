## ADDED Requirements

### Requirement: Generate and index the native skills catalog

The site prebuild SHALL generate per-skill pages and category indexes from native skill frontmatter, exclude imported submodules, and include the catalog route in local search.

#### Scenario: Search for a native skill

- **WHEN** the catalog prebuild runs twice against unchanged skills
- **THEN** generated content is identical, its count matches the native validator inventory, and the named skill appears in local search
