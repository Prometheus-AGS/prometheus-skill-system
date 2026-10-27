## ADDED Requirements

### Requirement: Coding roles retain phase-policy references
Coding-role exports and handoffs MUST carry compact references sufficient to recover the selected method, authoritative contract and permitted stage operations. They MUST preserve native permissions and report unsupported or unresolved references without transferring execution authority.

#### Scenario: POL-001 delegated role
- **WHEN** the same phase is exported or handed to a coding role in a supported harness
- **THEN** the role receives current contract/method identities and the same included value, exclusions and authority as the lead, with actual path/content identity recorded

#### Scenario: POL-003 changed referenced content
- **WHEN** a referenced policy or supersession document changes after handoff creation
- **THEN** receiving inspection reports the changed identity and resolves current authorized context before dependent action rather than trusting the old digest alone
