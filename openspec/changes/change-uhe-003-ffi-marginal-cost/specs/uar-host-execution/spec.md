## ADDED Requirements

### Requirement: The FFI pattern is confirmed or reversed by measurement

Adding a second function SHALL have its hand-written glue counted. Exceeding the decision's threshold SHALL reverse the pattern choice and record that reversal.

#### Scenario: Exceeding the threshold reverses the decision

- **GIVEN** adding one function needs more than the threshold of hand-written glue
- **WHEN** the measurement is recorded
- **THEN** the pattern decision is reversed rather than retained
