## ADDED Requirements

### Requirement: Publish baseline comparisons and measured results

Grader evaluation SHALL provide an offline comparison against stored results and document precision, recall, correlations, measured confidence, and residual failure modes. Live re-evaluation SHALL remain separately callable.

#### Scenario: Review a subsequent grader change

- **WHEN** a maintainer compares results with the stored baseline
- **THEN** the comparison can flag schema or prompt regressions without a live grader run, and the results document reports the observed tradeoffs
