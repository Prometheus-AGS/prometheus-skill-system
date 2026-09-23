---
paths: ['**/*.py', '**/pyproject.toml']
---

# Python

Loaded when a Python file is read. Not resident.

Batch implementation until a production path is complete. Use a narrow type or lint
check earlier only to unblock work. At a completed change boundary, run the smallest
integration-marked scenario that exercises the real entry point and collaborators.
Unit, mock-only, filtered-function, and per-edit tests are not completion evidence.
Reserve broad and slow integration suites for the final applicable phase or release.

## Hard rules

- Type hints on public functions. `mypy` is a gate, not a suggestion.
- Never test code not yet wired into the call graph.

<!-- Replace example boundaries with this project's real production-path gates. -->
