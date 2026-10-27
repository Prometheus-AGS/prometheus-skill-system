# Specification: issues-194-195-196

Date: 2026-10-27, America/Chicago. Backend: OpenSpec from the existing project root, with no explicit backend override. Baseline: bfe572ff8a97a21c7610c44ea6b39d5f97a857b7. This is a completed specification draft, not implemented or accepted behavior.

## Change set and dependency order

| Change | Requirements | Production tasks | Dependencies and ownership |
|---|---|---|---|
| kbd-194-change-identity-repair | IDR-001–003 | 1.1–1.4 | Runtime/import/CLI/apply identity and repair; no bootstrap/team/BDD edits |
| kbd-195-vlbv-policy-propagation | POL-001–003 | 1.1–1.4 | Context-bootstrap, team source, KBD authority/recovery and cadence; separate from #194 |
| kbd-196-value-scenario-workflow | SCN-001–002, VER-001–003 | 1.1–1.4 | Follows #195 in shared KBD stage files; owns discovery/verify procedures and BDD guidance |
| kbd-194-196-delivery-qualification | PKG-001 and parent corpus | 1.1–1.2 | Follows all source changes; sole C-01 generated reconciliation owner; 2.1–2.2 are evidence-only final-boundary obligations |

Every change has proposal.md, design.md, tasks.md, capability delta(s) and verification.md. `specification-manifest.json` lists paths. No canonical implementation changes/tasks were registered by Spec; Plan owns registration and worker routing.

## Finite contract

`adopted-contract.json` is authored against the existing shape with exactly 12 stable IDs. Contract digest and plan revision remain null for the canonical adoption writer; no decision/bind-contract command was issued. ImplementationComplete is false and authoringState is DRAFT_NOT_ADOPTED. Budgets (180 minutes, 240 tool calls, two repair attempts, two no-progress events), macOS source/generated/scratch-installed surfaces and calibration are proposed defaults, not approved ceilings or platform qualification.

`proposed-calibration.md` supplies ordinary-positive, irrelevant/deferred and rare-mandatory examples for owner review. Its SHA-256 reference binds it into the draft contract. Plan must resolve adoption inputs, current authority and prior consumption without resetting history. The request to write this specification does not adopt its proposed details or authorize execution.

Scenario definitions precede generation; executable integration tests are not authored here. The future `scripts/tests/kbd-194-196.integration.mjs` gate is a planned boundary-only acceptance artifact, not an available or passing command. Fixtures/candidate/build/configuration hashes must be frozen before execution; absent identities block rather than pass.

## Required distinctions

- #194 source evidence identifies possible mechanisms, not the actual 1.11.3 cause. Require diagnostics and a typed signed explicit repair without journal rewriting or fabricated receipts. Original-incident repair remains NOT_EXERCISED until the affected journal and authorized mapping exist.
- #195 carries policy, contract, supersession, evidence, budgets and effects through existing references. Host policy remains authoritative; conflicts are surfaced and no compiler exception bypasses selected VLBV. Instruction-only integration is not universal enforcement.
- #196 keeps discovery proposed, authoring policy-sensitive and execution separately authorized at the complete value-phase boundary. Existing runners and canonical closure stay authoritative; green exit without observed assertion cannot pass.
- Implementation, acceptance, demonstrated value, unresolved gaps, release eligibility and release authority stay separate. No automatic successor, publication, global install, paid/remote agent or unrelated mini-repository work.

## Constraints and verification disposition

Repository implementation-first policy overrides the generic OpenSpec task-template suggestion to run tests within each task group. Production groups deliver their documentation; all executable test authoring, generated-code checks and implementation review wait for complete parent production. The final change owns one initial packet, retained failures and targeted confirming runs after coherent authorized repairs. No broad unit suite or mock-only acceptance is specified.

OpenSpec validation, strict skill checks, formatting, builds, tests and distribution checks were not run during Spec. Lifecycle refresh succeeded at OpenSpec 1.15.0 with authoredPathsChanged empty. Artifact discovery/status reads are lifecycle metadata, not acceptance evidence. Final committed-state protected-test certification remains required when applicable, using existing signed approval for intentional protected changes.

Independent review remains PENDING under the completed-production reviewer boundary. Prior preflight did not establish distinct reviewer identity; no remote judge or reviewer was dispatched. Keep pending receipts in Plan/Execute; they do not satisfy final certification.

## Recalled constraints

Prior context records “Reuse existing finite contract and typed closure” and “issue #194 incident root cause remains unconfirmed without affected journal/version and map-key versus embedded-ID inspection.” The changes preserve both. Historical repair anecdotes do not authorize task replay or another phase's mutation.

## Next stage

READY for `/kbd-plan issues-194-195-196` to order/register work, resolve adoption/readiness inputs, preserve evidence-only classifications and adopt the finalized contract through its existing canonical writer. No implementation starts here and no parent-phase completion is claimed.
