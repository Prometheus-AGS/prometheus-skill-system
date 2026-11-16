# VLBV machinery cases

Generated 2026-11-16T08:38:16Z at commit `78abfd843d88ed5f6cb88e2bc139a22013452541` with prometheus 1.14.0.
These cases are harness-independent: they run once and their verdicts are carried into every harness report.

| Case | Requirement | Verdict | Observed |
|---|---|---|---|
| AT05 | VLBV 05 | PASS | exit=2 stdout=BLOCKED phase=closure-phase reason=evidence-missing(/private/var/folders/ln/0wnpd96j26z2qhvx9m6hwt2r0000gn/T/vlbv-machinery-1HAnFf/AT05/project/.k |
| AT06 | VLBV 06 | PASS | exit=2 stdout=BLOCKED phase=closure-phase reason=empty-evidence:CS-1 |
| AT09 | VLBV 09 | PASS | exit=2 stdout=BLOCKED phase=closure-phase reason=evidence-run-mismatch(evidence=project-20261116T083756Zx canonical=project-20261116T083756Z) |
| AT10 | VLBV 10 | PASS | exit=2 stdout=BLOCKED phase=closure-phase reason=not-pass:CS-2(NOT_EXERCISED) |
| AT15 | VLBV 15 | PASS | exit=2 stdout=BLOCKED phase=closure-phase reason=evidence-stale-sources(stale(src.txt)) |
| AT22 | VLBV 22 | PASS | scanner refused the secret artifact: true (json-token-field) |
