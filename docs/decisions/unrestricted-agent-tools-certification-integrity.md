# Decision: keep agent tools unrestricted and enforce integrity at certification

**Status:** accepted · 2026-08-03 · release 1.7.0

## Context

Tool matchers cannot reliably determine whether an arbitrary Bash or Python
program mutates a protected test. Blocking named tools removes legitimate
diagnostics and creative implementation paths while leaving other mutation
mechanisms uncovered.

## Decision

Bash, Python, Edit, Write, and MultiEdit remain unrestricted. Final local
certification compares a declared base commit with the candidate commit and
detects content changes, deletion, rename, and mode changes under protected BDD
paths. Intentional changes require an SSH-signed canonical manifest naming both
commits, every path and SHA-256, reason, approver, and timestamp. Verification
uses the `prometheus-test-change` namespace and the checked-in allowed-signers
policy.

## Alternatives considered

- Shell parsing and command allow-lists were rejected as incomplete and hostile
  to composable tools.
- Blocking Edit/Write but allowing Bash was rejected as method-dependent.
- Relying on code review alone was rejected because mode/rename changes are easy
  to miss.

## Consequences

Agents can continue without approval, but an uncertified protected change cannot
ship. Approval is auditable and bound to exact Git objects.

## Verification

Temporary repositories prove Bash and Python can mutate freely while
`scripts/verify-protected-tests.mjs` detects every protected change class and
accepts only a valid signed manifest.
