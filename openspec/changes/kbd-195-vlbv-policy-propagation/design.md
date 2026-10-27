## Context

See phase analysis and proposal. Team role inputs, prompt fields and selected file provenance already exist; KBD handoff has readInOrder, warnings, targetChecks and startPrompt. Recalled lesson: “Reuse existing finite contract and typed closure.”

## Goals / Non-Goals

Propagate a compact reference chain through existing fields; do not add a second contract authority, blanket shell interception or global context rewriting. Preserve operator-owned immutable policy.

## Decisions

1. Use a phase-local `method-context.md` as a derived readable index referencing canonical adoption/decisions and contract-bound policy/calibration documents. It grants no authority. Include project/run/phase, contract digest, latest selected method/supersession, included value/exclusions, stage operations, candidate/evidence references, consumed budgets, unresolved effects and control capability classification. Missing critical references are explicit blockers; never reconstruct acceptance from a stale command.
2. Reuse role inputs and prompt, selected sourceFiles/evidenceFiles provenance, and harness handoff readInOrder/startPrompt. Add only narrowly scoped schema support if a required reference cannot be represented. Hash referenced acceptance-bearing artifacts through contract identity; do not freeze a mutable method pointer without its target identity. Consumers resolve latest non-superseded authorized decisions, then compare content.
3. Bootstrap resident/path rules point to the index and governing policies; recovery stays compact. Init/Assess refresh discovery, Execute dispatch and child/harness handoff carry it, pause/resume and compaction restore it. Describe unavailable host capabilities honestly rather than inventing hook support.
4. Keep host authority and selected method separate. If no compatible course exists, surface the specific conflict before dependent execution. Here no generated-code check occurs during generation; the repository's compiler exception is not edited or used to bypass selected VLBV. Repository test-authoring order remains in force.
5. Preserve current supersession and evidence/budget/effect lineage in the derived view, not a second mutable ledger. Uncertain effects require reconciliation, not a fresh retry budget. Delivery-cadence guidance treats the adopted complete value phase as the boundary.

## Risks / Trade-offs

- Instruction payload fidelity does not prove model compliance. Qualification separately observes selected real harness sessions; unsupported capabilities remain unqualified.
- Automatic bootstrap can overwrite managed regions. Stage proposed changes and preserve user-owned text; do not rewrite the repository's immutable policy.

## Migration Plan

Source/templates first; no automatic user installation. Existing phases without selected-method data report unknown and require adoption before dependent verification. Generated/isolated-install reconciliation is owned by the final delivery change.

## Ownership and ordering

Own context-bootstrap, team runtime source, KBD authority-discovery/recovery references, delivery-cadence and docs. Defer team payload compilation and generated outputs to kbd-194-196-delivery-qualification. #196 subsequently adds discovery calls and BDD-specific workflow to shared KBD stage files; serialize those edits. Do not touch #194 runtime identity or protected scenarios. No implementation review or tests before complete parent production.

## Analyze reuse evidence

- library: cand-002 — Existing adopted contract, decisions, readiness and typed phase closure (adopt). No parallel ledger or generic closure replacement. Readiness: identity, adoptionSource, scenarioMatrix, budgets, workerRoutes and boundaryPacket; existing machinery can support the integration, but #194 repair still needs production code.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/skills/process/kbd-process-orchestrator/references/schemas/adopted-contract.schema.json — Contract has stable scenario IDs, oracle, negativeControl, budgets, evidence invalidation, workerRoutes and a single post-implementation boundary packet.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/substrate/kbd-runtime/src/phase_closure.rs#L775 — Typed closure already handles pass/refusal and refuses unsupported cancelled/deferred dispositions.
- library: cand-003 — Bootstrap, KBD handoff and team prompt/provenance surfaces (adapt). Reuse inputs, selected file provenance and handoff references before proposing additive schema fields. Readiness: workerRoutes, identity, amendmentPolicy, evidenceInvalidation and budgets; compact propagation needs implementation, not a new runtime authority.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/skills/process/agent-team-creator/runtime/src/adapters-codecs.mts#L64 — Role prompt already exports outcome, ownership, inputs, outputs, dependencies and skills.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/skills/process/agent-team-creator/runtime/src/handoff-provenance.mts — Selected provenance already hashes source/evidence files and warns that observation transfers no authority.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/skills/process/kbd-process-orchestrator/references/schemas/harness-handoff.schema.json — Handoff has readInOrder, targetChecks, warnings and startPrompt.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/skills/process/prometheus-context-bootstrap/assets/hooks/reanchor.sh — Recovery restores position, not VLBV method/calibration/supersession references.
- library: cand-006 — Published VLBV method and supersession procedure (reference). Reference the independent method and build a small pack-specific discovery/routing procedure. Readiness: functionalScope, exclusions, scenarioMatrix, amendmentPolicy; generic library adoption cannot supply project value judgments.
  - Source: https://ipfs.prometheusags.ai/ipfs/bafybeihp44qigr3g5zoalkt3p6xvovaglcuzbsp3l4os5vba26glfz6zya/index.html — Method source read during Assess: pre-generation acceptance planning, owner calibration, complete-value-phase verification and explicit supersession; not product qualification.
