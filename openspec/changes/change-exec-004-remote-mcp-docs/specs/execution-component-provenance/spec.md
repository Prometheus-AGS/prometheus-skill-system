## MODIFIED Requirements

### Requirement: Pre-instantiation component authorization
Component bytes SHALL be authorized before validation, compilation, caching, linking, or instantiation. Estate mode SHALL require an active signed-generation manifest entry whose component hash and capability metadata match the installed bytes and whose generation has consistent signed target receipts; standalone and bundled-mobile modes SHALL require an exact configured SHA-256 pin.

#### Scenario: Modified component
- **WHEN** component bytes differ from the authorized manifest or configured digest
- **THEN** loading fails before engine work and no receipt claims execution

#### Scenario: Inconsistent target receipt
- **WHEN** the active generation's local component is valid but a required target receipt binds different bytes or metadata
- **THEN** release certification fails without invalidating historical receipts from previously certified generations
