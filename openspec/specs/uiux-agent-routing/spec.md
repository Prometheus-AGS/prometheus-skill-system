# uiux-agent-routing Specification

## Purpose
Defines how UI/UX work resolves incumbent implementation context and selects installed design-review capabilities without inventing unavailable requirements.

## Requirements

### Requirement: Impeccable context uses an existing implementation target
The UI/UX routing workflow MUST resolve a named file, route, or surface to an existing incumbent target before loading Impeccable context.

#### Scenario: Proposed destination does not exist
- **WHEN** a plan names a future UI path that is not present in the workspace
- **THEN** the workflow resolves the current implementing surface and uses that existing target for context while retaining the future path only as a design destination

#### Scenario: Existing target cannot be resolved
- **WHEN** no incumbent implementation target can be found
- **THEN** the workflow records the unresolved target and does not claim that Impeccable analyzed a concrete surface

### Requirement: UX review requirements are capability-aware
The UI/UX routing workflow SHALL consult named skills only when they are present in the active catalog and SHALL use a documented installed fallback when an optional skill is absent.

#### Scenario: ux-designer is installed
- **WHEN** the active catalog contains an `ux-designer` capability
- **THEN** the workflow consults it and records that consultation

#### Scenario: ux-designer is absent
- **WHEN** the active catalog does not contain `ux-designer`
- **THEN** the workflow uses UI/UX Pro Max plus `frontend-design` for the UX-review pass and records the fallback without treating the missing name as an unfulfilled blocking requirement

### Requirement: Injected routing updates remain fenced
Refreshing the UI/UX routing pack MUST modify only its managed fence and preserve all surrounding project instructions byte-for-byte.

#### Scenario: Existing managed block is refreshed
- **WHEN** the injector runs against a file containing one valid UI/UX routing fence
- **THEN** only the fenced content changes and a second run is idempotent
