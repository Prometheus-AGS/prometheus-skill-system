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

Complete every planned production change in the active phase before authoring or running tests, formatters, validators or reviews. A task or change boundary does not independently authorize those operations. During implementation use source inspection and editor diagnostics. The project retired its compiler exception. The first build runs at the authorized B4 boundary after the phase implementation is complete. A failed boundary requires an owner-authorized repair and the affected B5 checks.

At the final local boundary, run the applicable deterministic checks and real integration gate, then independent review with an actual distinct-model route. A fresh context or different alias alone does not demonstrate a distinct model family. Record unresolved review or collaborator limits rather than fabricating certification. Batch fixes and rerun the smallest confirming integration command before the applicable final gate.

Protected BDD scenarios remain unchanged unless the owner supplies the required SSH-signed canonical approval. Documentation-only scope does not waive implementation-first timing, source integrity or local evidence requirements. Implementation, evidence, certification and publication remain separate completion dimensions.

## Evidence references in the closure packet

`boundary-results.json` may record which candidate a result belongs to
(`candidate.baseHead`, `candidate.treeDigest`), where it ran (`venue`) and, per
scenario, `scopeRef`, `fixtureRef` and `oracleRef`. The closure packet shows
them under **Evidence references** and compares the recorded candidate with
the checkout it is generated in.

| Candidate | Meaning |
|---|---|
| `CURRENT` | The recorded commit and tree are the current ones |
| `STALE` | The results belong to an earlier candidate; for the current one every scenario is NOT_EXERCISED |
| `UNKNOWN` | Uncommitted source changes prevent a comparison |
| not recorded | The evidence carries no candidate |

The label is a report. It changes no verdict and no disposition, and adds no
refusal.

See [local validation](/docs/operations/local-validation-and-docs-automation) and [task model assignments](/docs/kbd/task-model-assignments).
