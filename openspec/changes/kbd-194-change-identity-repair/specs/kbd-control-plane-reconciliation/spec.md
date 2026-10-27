## ADDED Requirements

### Requirement: Change identity registration preserves existing work
Canonical registration MUST refuse conflicting reuse of a phase-scoped change identity without changing its tasks, status or evidence. Equivalent retries MUST be idempotent or return an explicit existing-identity result.

#### Scenario: IDR-001 conflicting registration
- **WHEN** a change ID already exists and a caller registers it with conflicting task or metadata content
- **THEN** registration identifies the conflict, preserves the original record and does not create a duplicate

#### Scenario: IDR-001 duplicate legacy rows
- **WHEN** legacy input contains repeated IDs or conflicting map keys and embedded IDs
- **THEN** import names every competing identity and preserves all input bytes without silently overwriting a row

### Requirement: Reconciliation diagnoses identity before repairing it
Inspection MUST distinguish duplicate change IDs, key/embedded-ID mismatch, duplicate projection rows, missing qualified tasks, ambiguous titles and unmatched boundary receipts. It MUST identify the affected phase and competing identities without changing state.

#### Scenario: IDR-001 duplicate projection and missing task
- **WHEN** reconciliation encounters a repeated progress ID or a qualified subject whose task key is absent
- **THEN** it reports the repeated ID or missing key explicitly rather than treating either as proof of a missing start receipt or canonical duplication

### Requirement: Canonical identity repair retains history and authority
Canonical repair MUST require an authorized, explicit identity mapping bound to the current revision and source content. It MUST append a signed operation, retain source records and task/evidence/receipt lineage, and reject unresolved conflicts without mutation.

#### Scenario: IDR-002 authorized repair and replay
- **WHEN** an operator supplies an unambiguous mapping for a confirmed damaged canonical record
- **THEN** one signed operation restores unique effective identity, preserves original history and consumption, and replay produces the same effective result

#### Scenario: IDR-002 stale or conflicting repair
- **WHEN** the mapping is stale, unauthorized, incomplete or conflicts with a task outcome or outstanding receipt
- **THEN** repair refuses with the unresolved identities and leaves the journal and work unchanged

### Requirement: Task continuation uses repaired canonical identity
After authorized repair, apply and reconcile MUST resolve the effective registered task identity and retain receipt lineage. Projection-only repair MUST use confirmed canonical state without creating a canonical merge or fabricated completion receipt.

#### Scenario: IDR-003 continuation after repair
- **WHEN** the backend ordinal differs from a surviving registered task key and its receipt lineage is resolvable
- **THEN** the normal apply entry point continues that task exactly once and reconciliation converges without creating another change or task

#### Scenario: IDR-003 unavailable receipt lineage
- **WHEN** task continuation has an unmatched start or uncertain prior effect
- **THEN** it remains blocked with that obligation explicit and does not synthesize evidence from an implementation-complete flag
