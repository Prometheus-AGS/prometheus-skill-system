## ADDED Requirements

### Requirement: Retain varied explanations with reviewed gold labels

The dataset SHALL contain at least twenty varied Feynman explanations across three domains, with scores for four dimensions and identified misconceptions. Draft annotations SHALL remain provisional pending human review.

#### Scenario: Inspect an annotated misconception

- **WHEN** a dataset item repeats a misconception from its corpus
- **THEN** the item identifies the misconception and records a zero misconceptions_absent score in its draft annotation without presenting that annotation as reviewed
