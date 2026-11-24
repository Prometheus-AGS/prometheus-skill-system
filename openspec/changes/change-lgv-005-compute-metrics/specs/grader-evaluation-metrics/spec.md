## ADDED Requirements

### Requirement: Compare actual grades to gold annotations

The metric tool SHALL report misconception precision and recall and Pearson and Spearman correlations for completeness, accuracy, and clarity. It SHALL identify the largest deviations for review.

#### Scenario: Summarize a completed evaluation batch

- **WHEN** the metrics tool reads raw grades and matching index annotations
- **THEN** it writes machine-readable metrics, a human-readable table, and examples with the largest deviations from gold
