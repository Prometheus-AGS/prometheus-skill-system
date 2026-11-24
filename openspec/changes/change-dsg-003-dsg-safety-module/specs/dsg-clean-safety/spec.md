## ADDED Requirements

### Requirement: Guard cleanup with activity, exclusion, and age checks

DSG SHALL preview by default, honor configured exclusions and modification age, and check process activity and dirty Git state before moving an eligible item to Trash. A per-item failure SHALL be reported and allow the batch to continue.

#### Scenario: Try to clean entries with distinct safety outcomes

- **WHEN** the cleanup candidates include an active path, dirty repository, recent path, excluded path, or eligible inactive item
- **THEN** DSG reports the reason for each protected entry and only an eligible item reaches Trash when force is explicit
