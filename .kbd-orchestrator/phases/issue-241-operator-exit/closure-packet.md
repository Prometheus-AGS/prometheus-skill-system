# Closure packet — issue-241-operator-exit

Disposition: **CLOSED** (budget-exhausted). This packet is a derived view of
the bound contract (sha256:99f3d33dc245a1af798b86b3b5ecec33dbd4dab0ac63113f55ae01a0e2f7f93a) and
`boundary-results.json` (sha256:f0886e8823755866be7702b85e1f50a9c87bc11bd4b516ac79ef1b4950504725); the canonical KBD disposition is authoritative.

## What works, and for whom

- KBD-1 — An operator can close on 15 evidenced scenarios and start the separately authorized successor (for KBD project operator)
- KBD-2 — An omitted bound scenario refuses the exit before a journal write (for KBD project operator)
- KBD-3 — Withdrawn work stays withdrawn in the immutable record and status (for KBD project operator)
- KBD-4 — A restricted process can read without poisoning the next process writing (for KBD project operator)
- KBD-5 — An operator can recover a stale probe classification while intentional restrictions persist (for KBD project operator)
- KBD-6 — A contract with an unregistered owner task never receives a signable manifest (for KBD project operator)
- KBD-7 — Full delivers accepted reflection and terminal actions to the correct phase and reconciles task counts (for KBD project operator)
- KBD-8 — Mini delivers accepted reflection and terminal actions to the correct phase and reconciles task counts (for KBD project operator)

## Under which conditions

- environment: macOS external APFS checkout, Full and Mini
- KBD-1 real collaborator: prometheus runtime and CLI
- KBD-1 real collaborator: local filesystem and operator key
- KBD-1 real collaborator: generated skill package
- KBD-2 real collaborator: prometheus runtime and CLI
- KBD-2 real collaborator: local filesystem and operator key
- KBD-2 real collaborator: generated skill package
- KBD-3 real collaborator: prometheus runtime and CLI
- KBD-3 real collaborator: local filesystem and operator key
- KBD-3 real collaborator: generated skill package
- KBD-4 real collaborator: prometheus runtime and CLI
- KBD-4 real collaborator: local filesystem and operator key
- KBD-4 real collaborator: generated skill package
- KBD-5 real collaborator: prometheus runtime and CLI
- KBD-5 real collaborator: local filesystem and operator key
- KBD-5 real collaborator: generated skill package
- KBD-6 real collaborator: prometheus runtime and CLI
- KBD-6 real collaborator: local filesystem and operator key
- KBD-6 real collaborator: generated skill package
- KBD-7 real collaborator: prometheus runtime and CLI
- KBD-7 real collaborator: local filesystem and operator key
- KBD-7 real collaborator: generated skill package
- KBD-8 real collaborator: prometheus runtime and CLI
- KBD-8 real collaborator: local filesystem and operator key
- KBD-8 real collaborator: generated skill package

## Limits

- none

## What remains

- nothing: every scenario passed

## Pre-authorization runner artifacts

- no runnerOutputGlobs declared in the contract; scan not possible

This scan is passive evidence (VLBV 04, 10): it reports and never blocks.

## Closure states

| State | Value |
|---|---|
| Implementation complete | true |
| Behavior accepted | true |
| Release eligible | true |
| Release authorized | false (a separate owner decision) |

Method: VLBV.
Executed by: Codex desktop Codex Desktop 0.171.0-alpha.13 (gpt-6.2-astra).
