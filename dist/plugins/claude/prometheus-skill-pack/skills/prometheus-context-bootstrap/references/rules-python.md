---
paths: ['**/*.py', '**/pyproject.toml']
---

# Python

Loaded when a Python file is read. Not resident.

| Tier | Commands |
|---|---|
| T0 every edit | `ruff check`; `mypy <module>` |
| T1 unit complete | `pytest path::test_name` |
| T2 phase complete | `pytest` |
| T3 milestone only | slow and integration-marked suites |

## Hard rules

- Type hints on public functions. `mypy` is a gate, not a suggestion.
- Never test code not yet wired into the call graph.

<!-- Replace the commands above with this project's real ones if they differ. -->
