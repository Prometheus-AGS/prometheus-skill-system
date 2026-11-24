## ADDED Requirements

### Requirement: Require evidence for rubric changes

Systematic grader failures SHALL receive a targeted rubric edit and an affected-item rerun with recomputed metrics. If the deviations do not form a systematic pattern, the evaluation record SHALL document the evidence for leaving the rubric as-is.

#### Scenario: Decide whether observed deviations justify tuning

- **WHEN** reviewers investigate the evaluation items with the largest deviations
- **THEN** a systematic issue yields a measured before-and-after comparison, while isolated variance yields a documented no-op
