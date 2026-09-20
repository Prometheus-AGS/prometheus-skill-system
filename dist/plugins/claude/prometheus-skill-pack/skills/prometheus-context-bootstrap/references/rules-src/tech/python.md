---
paths: ['**/*.py', '**/pyproject.toml']
---

# Python

| Tier | Commands |
|---|---|
| T0 every edit | `ruff check`; `mypy` |
| T1 unit complete | `pytest path::test_name` |
| T2 phase complete | `pytest` |
| T3 milestone only | slow and integration-marked suites |

Organise by capability, not by layer. No `.py` file over 500 lines: turn `module.py` into a package with a
thin `__init__.py`, split by responsibility.
