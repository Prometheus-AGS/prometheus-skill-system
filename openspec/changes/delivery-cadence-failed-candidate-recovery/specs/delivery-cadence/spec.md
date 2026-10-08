## ADDED Requirements

### Requirement: Audited disposition of a failed delivery
Cadence SHALL allow an operator-authorized, evidence-linked retirement of the immediately preceding finalized failed delivery without changing its failed outcome, canonical completion, source identity, or publication obligations.

#### Scenario: Continue after independent phase evidence
- **WHEN** a failed candidate has been finalized and a separate phase gate passed on a different source set
- **THEN** the operator may record the passed gate as independent evidence and retire the failed Cadence candidate
- **AND** the candidate remains failed and is not counted as a successful delivery
- **AND** ordinary review, hook, budget, and publication-capacity gates still apply to new work

### Requirement: Corrective admission is limited to the failed predecessor
Cadence SHALL accept an evidence-backed corrective iteration only when it names the immediately preceding finalized failed iteration and its observed failed operation.

#### Scenario: Correct the last failed operation
- **WHEN** a finalized failed iteration has a matching local failure receipt
- **THEN** a corrective iteration may start with that receipt and a reason
- **AND** a receipt naming another iteration cannot bypass the failure gate

### Requirement: Bounded multi-source frozen artifact reconciliation
Cadence SHALL preserve the complete ordered source identity of a frozen artifact candidate. A multi-source frozen-artifact reconciliation may admit untracked files only from secondary preserved sources; the primary application source remains clean, every tracked patch remains empty, and every admitted preserved file must match the captured current input bytes.

#### Scenario: Reconcile a frozen application with immutable secondary inputs
- **WHEN** a candidate has a clean primary application snapshot, immutable secondary snapshots, a successful frozen build and launch, and operation evidence naming the primary application revision
- **THEN** Cadence may reconcile the frozen external driver without rebuilding the application
- **AND** it verifies every source reference and each secondary preserved untracked file before adopting or finishing
- **AND** a changed, missing, linked, added, removed, reordered, or application-source untracked input is rejected
