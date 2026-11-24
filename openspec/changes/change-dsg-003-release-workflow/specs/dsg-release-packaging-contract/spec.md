## ADDED Requirements

### Requirement: Document the installer release fallback contract

The version-tag packaging definition SHALL cover the published target matrix, and the Cowork command reference SHALL document the DSG binary artifact route used by the installer fallback.

#### Scenario: Resolve the macOS release fallback

- **WHEN** a user follows the documented fallback to select a macOS arm64 asset
- **THEN** the documented asset naming matches the packaging definition for that target
