## ADDED Requirements

### Requirement: Private key material is owner-restricted
Private signing key material SHALL be readable only by its owner and by principals the host cannot exclude. The installer SHALL assert this before use and SHALL refuse to proceed when it does not hold. The assertion SHALL be expressed per platform without weakening the guarantee.

#### Scenario: POSIX host
- **WHEN** the key file is inspected on a POSIX host
- **THEN** the assertion requires mode `0600` and ownership by the current user

#### Scenario: Windows host
- **WHEN** the key file is inspected on Windows
- **THEN** the assertion requires that the owner security identifier matches the process token, that the discretionary access control list is protected against inheritance, and that every non-inherited trustee is the owner, the local system account, or the local administrators group

#### Scenario: Over-granted access control list
- **WHEN** a Windows key file grants read access to an additional principal
- **THEN** the assertion fails and names the unexpected trustee

#### Scenario: Inherited access control list
- **WHEN** a Windows key file inherits access control entries from its parent
- **THEN** the assertion fails because inheritance makes the effective trustee set unbounded

### Requirement: Trustees are compared as security identifiers
Windows principal comparison SHALL use well-known and resolved security identifiers, never display names.

#### Scenario: Non-English host
- **WHEN** the host reports localized principal names
- **THEN** the assertion is unaffected because comparison is over security identifiers

### Requirement: Remediation is reported, not applied
When the assertion fails, the installer SHALL report the exact remediation command and SHALL NOT execute it.

#### Scenario: Failed assertion
- **WHEN** key protection does not hold
- **THEN** the installer exits with a structured reason and the remediation command, leaving the file unchanged
