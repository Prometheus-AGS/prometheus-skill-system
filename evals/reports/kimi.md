# VLBV protocol eval — kimi

Generated 2026-11-16T09:13:00Z at commit `78abfd843d88ed5f6cb88e2bc139a22013452541`. Status: **run**.

Harness: 2.5.3; model: kimi-code/k3 (family moonshot). Grader: MiniMax-M3. Samples declared: 3; decision rule: A sample is PASS only when every deterministic check passes AND the grader returns PASS. A failed deterministic check makes the sample FAIL regardless of the grader.

| Pass | Fail | Blocked | Not exercised |
|---|---|---|---|
| 6 | 0 | 17 | 0 |

| Case | Class | Verdict | Reason |
|---|---|---|---|
| AT01 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT02 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT03 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT04 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT07 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT08 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT11 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT12 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT13 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT14 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT16 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT17 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT18 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT19 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT20 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT21 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| S18 | agent | BLOCKED | 0 pass, 0 fail, 3 blocked of 3; the rule cannot be decided |
| AT05 | machinery (shared) | PASS | exit=2 stdout=BLOCKED phase=closure-phase reason=evidence-missing(/private/var/folders/ln/0wnpd96j26z2qhvx9m6hwt2r0000gn/T/vlbv-machinery-1HAnFf/AT05/project/.kbd-orchestrator/phas |
| AT06 | machinery (shared) | PASS | exit=2 stdout=BLOCKED phase=closure-phase reason=empty-evidence:CS-1 |
| AT09 | machinery (shared) | PASS | exit=2 stdout=BLOCKED phase=closure-phase reason=evidence-run-mismatch(evidence=project-20261116T083756Zx canonical=project-20261116T083756Z) |
| AT10 | machinery (shared) | PASS | exit=2 stdout=BLOCKED phase=closure-phase reason=not-pass:CS-2(NOT_EXERCISED) |
| AT15 | machinery (shared) | PASS | exit=2 stdout=BLOCKED phase=closure-phase reason=evidence-stale-sources(stale(src.txt)) |
| AT22 | machinery (shared) | PASS | scanner refused the secret artifact: true (json-token-field) |

## Isolation (AUTH-ISO-1)

- Real harness state unchanged: false
- Keychain item listing unchanged: true
- Copied auth files left behind: 0
- Auth files in the repository working tree: 0
- changed: ~/.claude.json
