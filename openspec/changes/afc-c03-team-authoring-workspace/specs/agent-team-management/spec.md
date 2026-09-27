## ADDED Requirements

### Requirement: Bounded file-backed team authoring

The agent-team creator SHALL support a project-local authoring workspace for one team while preserving the existing inline JSON request-file interface. Workspace-backed and inline inputs MUST pass through the same canonical compiler and produce byte-equivalent package content for equivalent input.

#### Scenario: Create a project-local authoring workspace

- **WHEN** an operator starts authoring a team for an authorized project and team identity
- **THEN** the creator stores mutable authoring state only beneath that team's `.agent-team` directory, records a revision for each mutation, and reports the exact files it owns

#### Scenario: Reject an escaping authoring path

- **WHEN** an authoring request, identity-derived path, or existing link would resolve outside the authorized project and team workspace
- **THEN** the operation fails before writing any file and leaves the current workspace unchanged

#### Scenario: Preserve inline input

- **WHEN** an operator supplies the existing `--input <request.json>` form without selecting a workspace
- **THEN** the creator validates and compiles that request without creating authoring files, and returns the same canonical result as an equivalent workspace-backed request

### Requirement: Guided team graph completion

The guided authoring flow SHALL represent team construction as inspectable questions and answers covering definitions and their graph relationships. It MUST identify every unanswered mandatory question before package creation rather than inventing a member, edge, ownership assignment, policy, or contract.

#### Scenario: Author a nested team graph

- **WHEN** an operator describes a team containing agents, nested teams, communication routes, workflow dependencies, ownership, limits, budget, input/output contracts, and task acceptance policy
- **THEN** the creator asks bounded questions for missing nodes and edges, preserves answered values, and reports the remaining mandatory questions by stable identifiers

#### Scenario: Resume guided authoring

- **WHEN** an operator reopens a previously written authoring workspace
- **THEN** the creator reconstructs the same answered and unanswered graph from files and continues without repeating or replacing accepted answers

#### Scenario: Mandatory graph answer remains unknown

- **WHEN** a required member, coordinator, dependency, acceptance, limit, budget, or contract answer is absent
- **THEN** validation identifies the corresponding field and package creation remains blocked

### Requirement: Complete definition and graph validation

Before versioning or package creation, the creator MUST validate every top-level portable document and all nested team and workflow relationships against the pinned UAR profile. Unknown mandatory semantics, malformed nested fields, unresolved immutable references, duplicate identities, invalid cardinalities, invalid coordinator or communication roles, cycles, and unsatisfied workflow dependencies MUST fail with field-level diagnostics.

#### Scenario: Valid complete team graph

- **WHEN** every document conforms to the pinned profile and all immutable references, member roles, communication edges, cardinalities, coordinator role, and workflow dependencies resolve consistently
- **THEN** validation succeeds and returns the exact profile and schema-source revision used

#### Scenario: Nested member is invalid

- **WHEN** a nested team member has an invalid immutable reference, cardinality, role, or responsibility
- **THEN** validation fails with a JSON-pointer diagnostic for the nested field and no package or version snapshot is written

#### Scenario: Unknown mandatory semantics

- **WHEN** input contains an unknown required extension or another mandatory field that the pinned profile cannot enforce
- **THEN** validation returns `required-unsupported` for that field and prohibits build, installation, and execution claims

### Requirement: Immutable next-version maintenance

Maintenance SHALL create a new semantic version from an explicitly selected immutable base. Existing definition and package versions MUST remain byte-identical, dependency references MUST resolve by identity, version, and digest, and no alias or mutable latest pointer may replace the locked closure.

#### Scenario: Create the next version

- **WHEN** an operator selects an existing base, supplies a new semantic version, and changes one or more definitions
- **THEN** the creator writes a distinct draft revision, recomputes affected definition and package digests and locks, retains unchanged source values, and leaves the base revision unchanged

#### Scenario: Attempt to reuse an immutable version

- **WHEN** changed content is assigned an identity and version that already exists
- **THEN** the operation fails before mutation and reports the existing and proposed digests

#### Scenario: Dependency changes beneath a team

- **WHEN** a referenced agent, nested team, or workflow moves to a new version
- **THEN** the maintained team or package explicitly adopts the new identity/version/digest tuple or remains pinned to the old tuple; it never floats silently

### Requirement: Lossless migration diagnostics

The creator SHALL migrate existing schema-v1 team manifests and UAR Draft 0.1.0-draft.1 authoring packages into the new workspace without deleting the original source. Every field conversion MUST receive a JSON-pointer diagnostic with source profile or revision, target profile, disposition, reason, and effective target reference when one exists.

#### Scenario: Lossless legacy migration

- **WHEN** every source field has an exact or translated representation in the target profile
- **THEN** migration retains all source values, emits only `exact` or `translated` diagnostics, and produces a target draft that can be validated independently

#### Scenario: Optional source field is unsupported

- **WHEN** an optional source field has no target representation
- **THEN** migration retains it in the preserved source record, emits `optional-unsupported`, and excludes it from effective behavior without claiming it was applied

#### Scenario: Required source field is unsupported

- **WHEN** a required source field, required skill version/configuration, required extension, or mandatory policy cannot be represented or enforced
- **THEN** migration emits `required-unsupported`, preserves the source for repair, and blocks package creation

### Requirement: Authoritative UAR profile consumption and portable authority separation

The creator MUST consume the exact official UAR collaboration profile `urn:prometheus:uar:collaboration:0.1.0-draft.2` schemas from a recorded immutable UAR revision. Portable definitions and packages MUST contain requested capabilities and references only; credential values, installed RepresentationGrant records, approval grants, and other host authority MUST remain outside authoring workspaces and exported packages.

#### Scenario: Validate against the pinned 0.1.0-draft.2 schema set

- **WHEN** a `0.1.0-draft.2` package is validated or built
- **THEN** the result records the official UAR source revision and schema digests, and a profile or schema mismatch fails instead of being reinterpreted

#### Scenario: Private authority appears in portable input

- **WHEN** a definition or package includes a credential value, installed grant, executable approval, or private binding authority
- **THEN** validation rejects the field and identifies the trust boundary without copying the value into diagnostics or output

#### Scenario: Binding references remain separate

- **WHEN** a portable package requires later deployment binding
- **THEN** the package may declare capability and reference requirements but contains no private binding identifiers unless the receiving principal can resolve an explicitly exportable reference

### Requirement: Generated parity and final live acceptance

The canonical TypeScript source, compiled Node.js payload, packaged skill, and generated Claude and Codex distributions MUST expose the same authoring and migration behavior. Completion evidence MUST include one final local live-UAR gate after the coherent implementation and generated artifacts are complete.

#### Scenario: Packaged authoring parity

- **WHEN** the source skill and each generated distribution process the same inline or workspace fixture
- **THEN** their canonical package bytes and field-level diagnostics are identical and no packaged execution depends on repository-root imports or runtime package installation

#### Scenario: Final live UAR gate

- **WHEN** the completed packaged creator migrates a legacy team, authors and validates a nested `0.1.0-draft.2` team, creates a next version, and submits it to an isolated live UAR persistent runtime
- **THEN** UAR preflight and install accept the supported package, cold readback returns the same immutable identity/version/digest closure, negative mandatory-field cases are rejected, and inspection confirms that no grants or secrets entered the package
