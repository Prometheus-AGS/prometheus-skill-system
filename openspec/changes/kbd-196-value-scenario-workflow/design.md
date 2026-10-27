## Context

See proposal and #195 method-reference contract. Existing scenarioMatrix and boundaryPacket already own scenario identity and execution; JS/Rust skills already own runner details. Recalled lesson: “Reuse existing finite contract and typed closure.”

## Goals / Non-Goals

Create small independently invocable procedures, not an orchestration framework, acceptance ledger or runner. No generic coverage target, universal harness, forced browser/video stack or global prerequisite matrix.

## Decisions

1. Name discovery `value-scenario-discovery`. Its output is a finite proposal document linked from existing phase context; the adopted contract references acceptance-bearing calibration content by identity. Assess/Spec/Plan invoke the same procedure. Mandatory constraints are separate from ordinary ranking; likelihood remains qualitative or UNKNOWN without evidence.
2. Name the boundary procedure `value-scenario-verify`. It consumes current method/contract references and invokes existing adapter commands only after complete phase/authority resolution. It does not independently adopt scope or write closure status outside canonical commands.
3. Extend existing BDD skills with explicit `author-only` instructions rather than replacing runners. Accept scenario IDs plus contract path/digest. Produce declarative feature and real assertion mapping with oracle source and false positive. Cucumber dry-run is not author-only: it loads support code. Host policy decides when executable authoring is allowed; this repository permits scenario definition now and executable test authoring only after complete production.
4. Reuse current evidence/closure schema and supported status vocabulary. A missing assertion is NOT_EXERCISED and cannot pass; unavailable environment/permission, stale identity and failed cleanup are BLOCKED. A genuinely exercised product oracle mismatch is FAIL. Preserve an inconclusive/uncertain explanation without inventing a new canonical terminal disposition.
5. Keep all material attempts and invalidation keys, including failed sensitivity controls, substitutions and cleanup. Challenge actual assertion/oracle families safely; do not create fake return-false probes disconnected from production. Authorized repair is another complete implementation batch, followed by the smallest relevant confirming run with lineage preserved.
6. Replace failing-test-first and per-change execution statements in BDD lifecycle/KBD integration. Distinguish definition, executable authoring and execution. No guidance change can repeal operator policy or alter protected scenarios without its existing approval.

## Risks / Trade-offs

- Reviewed calibration examples are required before contract adoption; examples authored in this phase remain proposals until the owner adopts them.
- Instruction-only procedures cannot guarantee model obedience. Qualify selected actual host sessions and runtime paths, and state unsupported surfaces.
- Evidence conversion must not treat formatter success as actual assertion observation. Keep original runner output beside mapped canonical evidence.

## Migration Plan

Keep bdd-testing as the deprecated redirect, route new references to current adapters, and preserve existing consumer runners/pins. Do not execute old commands from memory during migration. No automatic installation.

## Ownership and ordering

After #195, own discovery/verification procedure skills, JS/Rust BDD author-only guidance, BDD lifecycle/integration and discovery calls in KBD stage files. Serialize shared stage edits after #195 and preserve its policy references. The final delivery change owns all generated distribution reconciliation and phase-end tests/review.

## Analyze reuse evidence

- library: cand-002 — Existing adopted contract, decisions, readiness and typed phase closure (adopt). No parallel ledger or generic closure replacement. Readiness: identity, adoptionSource, scenarioMatrix, budgets, workerRoutes and boundaryPacket; existing machinery can support the integration, but #194 repair still needs production code.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/skills/process/kbd-process-orchestrator/references/schemas/adopted-contract.schema.json — Contract has stable scenario IDs, oracle, negativeControl, budgets, evidence invalidation, workerRoutes and a single post-implementation boundary packet.
  - Source: https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff8a97a21c7610c44ea6b39d5f97a857b7/substrate/kbd-runtime/src/phase_closure.rs#L775 — Typed closure already handles pass/refusal and refuses unsupported cancelled/deferred dispositions.
- library: cand-004 — Cucumber.js via existing bdd-cucumber-js adapter (adapt). Keep existing runner; adapt skill routing and scenario/assertion/evidence links. Readiness: scenarioMatrix and boundaryPacket; no new general-purpose runner runtime needed.
  - Source: https://github.com/cucumber/cucumber-js/blob/edaa0bb17a0b9a6004d6179741597e37db87465e/docs/dry_run.md — Dry-run loads support code, skips steps and does not fail on undefined/ambiguous steps; it is neither author-only nor assertion evidence.
  - Source: https://github.com/cucumber/cucumber-js/blob/main/docs/custom_formatters.md — Existing message formatters expose runner results; no replacement runner is necessary.
  - Source: https://registry.npmjs.org/@cucumber%2fcucumber — Read-only npm metadata reports 13.3.0, MIT. This does not authorize changing consumer dependency pins.
- library: cand-005 — cucumber-rs via existing bdd-cucumber-rs adapter (adapt). Adapt the existing Rust skill, preserve package pins and use a targeted real integration entry point at phase end. Readiness: scenarioMatrix and boundaryPacket; no replacement runner.
  - Source: https://github.com/cucumber-rs/cucumber/blob/d95872541ee2a43d71089c64bd3fa8848e24c922/book/src/output/index.md — Existing output supports Cucumber JSON, JUnit and libtest integrations.
  - Source: https://crates.io/api/v1/crates/cucumber — Registry reports stable 0.23.0 and MIT OR Apache-2.0; this matches the current skill major/minor guidance.
- library: cand-006 — Published VLBV method and supersession procedure (reference). Reference the independent method and build a small pack-specific discovery/routing procedure. Readiness: functionalScope, exclusions, scenarioMatrix, amendmentPolicy; generic library adoption cannot supply project value judgments.
  - Source: https://ipfs.prometheusags.ai/ipfs/bafybeihp44qigr3g5zoalkt3p6xvovaglcuzbsp3l4os5vba26glfz6zya/index.html — Method source read during Assess: pre-generation acceptance planning, owner calibration, complete-value-phase verification and explicit supersession; not product qualification.
