## ADDED Requirements

### Requirement: Normalize legacy video manifest keys

The migration SHALL map legacy hex keys to canonical slug keys where possible, archive unmappable hex entries, preserve supported single-part slugs, and replace the manifest atomically. Dry runs SHALL report unmappable entries without writing.

#### Scenario: Dry run and migration of mixed keys

- **GIVEN** the video manifest contains legacy hex and valid slug keys
- **WHEN** the migration runs first in dry-run mode and then applies
- **THEN** the dry run reports the unmappable keys and leaves both manifests intact; apply writes a manifest with only valid slug keys, preserves its `videos` object, and saves unmappable entries in `docs/videos-manifest-legacy.json`
