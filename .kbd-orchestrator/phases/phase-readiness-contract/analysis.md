ANALYSIS: phase-readiness-contract
Date: 2026-10-18
Mode: bounded build-vs-adopt from the completed in-repo inventory (assessment.md). External research tiers intentionally not run — operator ruled current adoption inputs sufficient and prohibited broad research. All candidates are in-repo; no third-party library decision is required.

## Decision 1 — Where the adopted contract lives and how it is written

Options:
(a) New Rust runtime types + new CLI subcommand in substrate/kbd-runtime — rejected: requires Cargo while a foreign 3-day Cargo process occupies the machine (single-Cargo rule), exceeds the phase's time budget, and rebuilds evidence primitives that already exist.
(b) Contract as a phase artifact (`adopted-contract.json` per schema) + adoption recorded through the EXISTING canonical writer `prometheus kbd decision record` (immutable, plan-revision-bound, `supersedes` lineage) carrying the contract path + SHA-256 digest + adoption source — ADOPTED.
(c) Plain markdown note without canonical record — rejected: not immutable, not revision-bound, not auditable.

Rationale: (b) satisfies #184's canonical foundation (identity binding to project/run/phase/plan revision + content digest + immutable history) with zero new runtime code and zero Cargo invocations. Amendment/restart semantics come free: an amendment is a new `decision record` with `--supersedes`; the readiness checker rejects any digest ≠ current adopted digest, so stale identity cannot silently pass; consumed budgets/history remain in the append-only journal.

## Decision 2 — Where the Plan→Execute readiness gate lives

Options:
(a) Extend `shared/lib/stage-gate.sh` with semantic handoff validation + add a read-only readiness checker script `shared/scripts/kbd-phase-readiness-check.sh`, invoked by kbd-execute/kbd-apply before dispatch — ADOPTED.
(b) Bake the gate into the Rust runtime — rejected for the same Cargo/scope reasons as 1(a); also #185 (host enforcement) is explicitly out of scope, so the gate is a deterministic pre-dispatch check with honest "advisory where unenforced" labeling.
(c) Prompt-only instruction — rejected: prompts drift; a script gives deterministic READY/BLOCKED with a minimal missing list (#186 requirement).

Rationale: stage-gate.sh already enforces canonical-phase agreement (`_sg_assert_canonical_phase`); extending it to validate handoff schema + identity fields closes the "any parseable JSON passes" hole (stage-gate.sh L173) at the exact seam the issue cites. The checker is read-only against canonical state (`prometheus kbd status --json`, phase files) so negative controls cannot pollute canonical history.

## Decision 3 — Contract content model

ADOPTED: single JSON contract per phase (schema-validated) containing: contractVersion; identity (projectId, runId, phaseId, planRevision, contractDigest); adoptionSource (operator instruction reference); decisionOwner; releaseClaim (real user operation, supported environments/configs, current capabilities, explicit UNKNOWNs — capability inventory, not task counts); scenarioMatrix (finite list, stable scenario IDs, production entry point, real collaborators, oracle, negative control, fixtures with identity, evidence requirements, scenario owner); functionalScope vs resilienceScope (separate lists; resilience overflow → backlog, never silently waived safety/isolation checks); exclusions; budgets (timeMinutes, toolCalls, tokenBudget = measured-or-UNKNOWN+fallback, repairAttempts, noProgressLimit, exhaustion → checkpoint+BLOCKED); evidenceInvalidation policy (digest/revision-keyed, only affected evidence invalidated, history preserved); boundaryPacket (exact commands/argv, expected effects, required approvals, receipts to produce, cleanup steps, stop conditions, post-implementation-only flag).

Markdown template mirrors the JSON for operator authoring; the JSON is the machine-checked artifact.

## Decision 4 — Skill prompt/skill updates (contract elicitation upstream)

ADOPTED (minimal diffs, no behavior rewrite):
- `prompts/assess.md` + `skills/kbd-assess/SKILL.md`: add adopted-outcome elicitation block (user operation, baseline/UNKNOWN, supported env, safety checks, exclusions, decision owner).
- `skills/kbd-analyze/SKILL.md`: add readiness build-vs-adopt mapping + explicit READY/BLOCKED recommendation field.
- `skills/kbd-spec/SKILL.md`: require the finite scenario matrix with stable IDs via the extended acceptance template; separate functional vs resilience scope.
- `prompts/plan.md` + `skills/kbd-plan/SKILL.md`: require contract identity/digest, scenario→task coverage map, worker routes, freshness fields, budgets, and the single boundary packet.
- `skills/kbd-execute/SKILL.md` + `skills/kbd-apply/SKILL.md`: invoke the readiness checker before dispatch; READY required to dispatch; BLOCKED lists minimal missing items; never fabricate PASS.
- `references/templates/acceptance.template.md`: add stable scenario ID, fixture/permission readiness, status vocabulary (PASS / FAIL-product / BLOCKED-verification / NOT_EXERCISED), evidence-invalidation keys.

## Decision 5 — Handoff schema extension

ADOPTED (backward compatible): add optional `identity` object (projectId, runId, phaseId, planRevision, contractDigest) to handoff.schema.json. stage-gate.sh validates schema + identity match when present; the Plan→Execute boundary REQUIRES identity (older stages keep optional-stage walk-back semantics unchanged).

## Decision 6 — Acceptance evidence strategy (boundary, post-implementation only)

Per AGENTS.md integration-only policy and rule 9: all verification happens once, after complete production implementation, through real entry points:
- The readiness checker + adoption writer run against the REAL canonical runtime for THIS phase's own contract (dogfood = real production path: adopt → READY → eligible to dispatch).
- Negative controls run against scratch fixture phase dirs under `$TMPDIR` (copied trees, no live services, no writes to real canonical state): missing adoption → BLOCKED; stale digest → rejected; wrong phase identity → rejected; malformed handoff → rejected without writing evidence.
- No unit/mock/schema-only evidence cited; script tests exercise the shipped CLI + real scripts + real filesystem boundary.
- No Cargo involvement → foreign Cargo occupation does not block this phase; if implementation discovers an unavoidable Rust need, record BLOCKED instead of building.

## Open questions (non-blocking, recorded)

- Token telemetry remains UNKNOWN (no provider metering); contract carries UNKNOWN+fallback (time/tool-call bounds) per rule 7.
- `prometheus kbd gate` exists for signed gate receipts; whether to route the boundary integration run through it is a Plan-stage decision (default: yes for the single boundary gate if its receipt format fits; otherwise direct command + recorded argv/exit/observed).

ANALYSIS COMPLETE — recommendation: READY to spec (no material scope/acceptance decision missing; no operator question required).
