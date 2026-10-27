---
paths: ['**/*.go', '**/go.mod']
---

# Go

Loaded when a Go file is read. Not resident.

Batch implementation until a production path is complete. Use a narrow build or vet
command earlier only to unblock work. At a completed change boundary, run the
smallest integration target that exercises the real package entry point and external
collaborators, commonly a repository selector with `-tags=integration`. Unit,
filtered-function, mock-only, and per-edit tests are not completion evidence. Reserve
broad race-detector and workspace gates for the final applicable phase or release.

## Hard rules

- Race detection costs 5-10x memory and 2-20x execution time, and only finds
  races on paths the test actually exercises. It is a final boundary gate, not a
  continuous check.
- Errors are values. Wrap with context at the boundary, do not swallow.

<!-- Replace example boundaries with this project's real production-path gates. -->

## Selected phase method

Before this stack work, recover the active phase method-context.md and adopted contract from signed canonical KBD status/decisions. For selected VLBV, complete the agreed whole value phase before generated-code builds, tests, imports, lint/typecheck or verification probes; a stack-specific compiler/check example is not permission. Surface host-policy conflicts before dependent action. Retain supersession, evidence/candidate freshness, consumed budgets and unresolved effects; reconcile uncertain effects before retry. This path rule is instruction-only, not universal interception.
