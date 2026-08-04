## MODIFIED Requirements

### Requirement: Pin-aware budget enforcement
Garbage collection SHALL honor the configured byte budget while never collecting blobs referenced by unarchived Tier P or Tier W receipts, open certifications, or explicit pins. The default budget SHALL be 2 GiB on desktop and 256 MiB in mobile builds, with an explicit configuration override.

#### Scenario: Mobile budget pressure with verified receipt
- **WHEN** a mobile artifact store exceeds 256 MiB and its oldest blobs are referenced by an unarchived Tier W receipt
- **THEN** collection preserves every referenced blob and removes only eligible unpinned content even if the store remains above budget
