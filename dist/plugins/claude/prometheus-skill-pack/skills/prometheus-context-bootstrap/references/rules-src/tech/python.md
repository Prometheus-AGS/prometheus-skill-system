---
paths: ['**/*.py', '**/pyproject.toml']
---

# Python

Batch implementation until a production path is complete. Use a narrow type or lint check earlier only to
unblock work. At a completed change boundary, run the smallest integration-marked scenario that exercises
the real entry point and collaborators. Unit, mock-only, filtered-function, and per-edit tests are not
completion evidence. Reserve broad and slow integration suites for the final applicable phase or release.

Organise by capability, not by layer. No `.py` file over 500 lines: turn `module.py` into a package with a
thin `__init__.py`, split by responsibility.
