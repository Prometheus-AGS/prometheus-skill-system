## ADDED Requirements

### Requirement: A merged fix is verified on the installed surface
A change that repairs an installed script SHALL be verified by running the INSTALLED copy,
not the source, and under the conditions that reproduced the original failure.

#### Scenario: The reinstall completes
- **WHEN** the pack is reinstalled after a source fix
- **THEN** the installed copy SHALL contain the fix, verified by inspecting the installed
  path rather than the source path

#### Scenario: The original failure condition is reproduced
- **WHEN** the repaired behaviour depended on an environment variable used as a workaround
- **THEN** verification SHALL run with that variable UNSET, so a passing result cannot come
  from the workaround
