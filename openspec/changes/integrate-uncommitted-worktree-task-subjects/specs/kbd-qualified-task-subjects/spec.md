# Spec Delta

## Purpose

Allow callers to select a task unambiguously when different changes contain the same task identifier.

## ADDED Requirements

### Requirement: Compatible change-qualified task subjects
The task boundary guard SHALL accept `change/task`, `change:task`, and `change::task` for a task belonging to the selected change in the active phase. It SHALL retain existing rejection of ambiguous unqualified subjects.

#### Scenario: Repeated task identifiers
- **WHEN** two changes each contain task `1` and the caller selects either change using any supported qualified form
- **THEN** the guard resolves the selected change and task without choosing the other change

#### Scenario: Ambiguous bare identifier
- **WHEN** a bare task identifier matches multiple changes
- **THEN** the guard continues to reject the ambiguous selection
