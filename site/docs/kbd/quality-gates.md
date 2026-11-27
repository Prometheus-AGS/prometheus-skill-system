---
id: quality-gates
title: Quality Gates
---

# Quality Gates at the Completed Phase Boundary

## Implementation before evidence

Finish the coherent production implementation for the active specification or
phase before authoring, changing, or running tests. Intermediate compiler
feedback is reserved for a completed edit batch and must be package-scoped.
Acceptance uses the smallest relevant local full-integration gate; unit,
mock-only, snapshot, broad workspace, and hosted-CI results are not completion
evidence.

Only one Cargo or `rustc` process may run on the development machine at a time.
Each worktree keeps a separate target directory and shares reusable objects
through `sccache`. Release, all-target, full-workspace, and Clippy builds are
reserved for a completed implementation's final certification or a requested
artifact.

## Boundary and bottleneck evaluation

KBD records idempotent before/after receipts for OpenSpec tasks and phase/child
transitions. Each
receipt binds the canonical subject, ordinal, total, phase path, and source
revision. The detector emits the exact progress line and current position,
re-anchors outstanding obligations after compaction, and can repair only
derived waypoint/progress projections without changing the canonical revision.

Missing or out-of-order receipts block certification without guessing history.
Ambiguous authority is preserved for recovery rather than mutated. Ordinary
evaluation is deterministic, local, and network-free; adversarial review is
bounded to phase completion, ambiguous authority, or a repeated violation, and
its output is screened for sycophancy before it can certify work.

### Advisory recommendations

`prometheus kbd guard advise` reads canonical state and proposes one next step
when the same blocker was attempted again without new evidence. It writes
nothing and blocks nothing.

| Recommendation | When |
|---|---|
| `setup-repair` | Environment or release setup failed twice the same way |
| `measurement-repair` | A fixture, oracle or runner failed twice; the product result is not established |
| `authority-handoff` | The next step needs a decision the agent cannot make |
| `architecture-child` | The same product failure repeated; a scoped child phase is proposed, not created |

Waiting for an approval or a resource is never counted as a failed attempt.
While a gate is still running the answer is `in-flight: do not stop`.

Record a blocker with `--fingerprint`, `--class` and `--evidence` for it to be
counted. Blockers recorded without a fingerprint are ignored.

Complete every planned production change in the active phase before authoring or running tests, formatters, validators or reviews. A task or change boundary does not independently authorize those operations. During implementation use source inspection and editor diagnostics. The project retired its compiler exception. The first build runs at the authorized B4 boundary after the phase implementation is complete. A failed boundary requires an owner-authorized repair and the affected B5 checks.

At the final local boundary, run the applicable deterministic checks and real integration gate, then independent review with an actual distinct-model route. A fresh context or different alias alone does not demonstrate a distinct model family. Record unresolved review or collaborator limits rather than fabricating certification. Batch fixes and rerun the smallest confirming integration command before the applicable final gate.

Protected BDD scenarios remain unchanged unless the owner supplies the required SSH-signed canonical approval. Documentation-only scope does not waive implementation-first timing, source integrity or local evidence requirements. Implementation, evidence, certification and publication remain separate completion dimensions.

See [local validation](/docs/operations/local-validation-and-docs-automation) and [task model assignments](/docs/kbd/task-model-assignments).
