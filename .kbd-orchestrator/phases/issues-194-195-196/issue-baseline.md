# Issue source baseline

Captured for phase initialization on 2026-10-26 (America/Chicago). These issue requirements are phase inputs; no acceptance result or exit-contract adoption is claimed.

## #194: kbd-apply reconcile --repair cannot fix duplicate change ids; begin-task refuses non-unique subject

Source: https://github.com/Prometheus-AGS/prometheus-skill-system/issues/194
Updated: 2026-10-25T22:30:22Z
State at capture: open

## Summary
`kbd-apply reconcile --repair` cannot repair a phase whose ledger holds two changes with the same id. `begin-task` / `end-task` then refuse every task of that change, so the phase can never reach "all changes complete".

## Observed (prometheus 1.11.3, kbd-process-orchestrator skill)
- `prometheus kbd status --json` lists two changes with id `site-product-pages`: a planned, in-progress one with tasks 1.1 to 1.5, and a second, complete one with only task 1.1. `site-seo-prerender` appears to have the same shape.
- `kbd-apply reconcile usefulness-update --repair` returns: `boundary site-product-pages/1.1 has no matching start receipt`.
- Manual `begin-task` / `end-task` for the change returns: `Task subject site-product-pages/1 is not a unique canonical work item`.
- The runtime history is append-only, so the duplicate cannot be removed or merged from the CLI.

## Likely cause
A bulk script recorded tasks for changes that already existed in the ledger (it called `begin-task`/`end-task` per task with a total taken from `progress`). Registering by change id created a second change rather than reusing the first. The driver's "reuse the registered task" rule applies to tasks, not to changes.

## Expected
1. Registering a change whose id already exists should reuse it, or refuse with a clear message, never create a duplicate.
2. A documented repair path for an existing duplicate (for example `prometheus kbd change merge --from <id> --into <id>` or a signed tombstone for the stray change), so `reconcile --repair` can finish.
3. `reconcile` should name duplicate change ids explicitly instead of reporting a missing start receipt.

## Impact
One parent phase is stuck at 5 of 12 with every change implemented and merged; verify/archive and the phase reflection are blocked.


---

## #195: Propagate the selected VLBV phase policy through bootstrap, teams and recovery

Source: https://github.com/Prometheus-AGS/prometheus-skill-system/issues/195
Updated: 2026-10-27T01:28:02Z
State at capture: open

## Purpose and priority

**P1.** Integrate the selected [Value-Led Behavioral Verification (VLBV) method](https://ipfs.prometheusags.ai/ipfs/bafybeihp44qigr3g5zoalkt3p6xvovaglcuzbsp3l4os5vba26glfz6zya/index.html) consistently into existing Prometheus lifecycle and context surfaces. Keep the external method independently usable; this pack owns its integration and routing.

The owner’s three acceptance goals are:
1. Reliably exit KBD phases with the code actually needed.
2. Measure whether the need is met to an agreed level expressed in an exit contract.
3. Verify that the component or system delivers its intended value in how it behaves when run.

This issue concerns policy propagation and recovery. It does not implement another test runner, contract authority, or orchestration framework.

## Source baseline and existing capability

Read-only source assessment; acceptance below is proposed, not executed. References are pinned to full main `bfe572ff8a97a21c7610c44ea6b39d5f97a857b7`.

- [Execute](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/process/kbd-process-orchestrator/skills/kbd-execute/SKILL.md#L80) already defers tests and verification builds until every planned production change is complete.
- [Adopted contract](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/process/kbd-process-orchestrator/references/schemas/adopted-contract.schema.json#L157) already requires `boundaryPacket.postImplementationOnly: true` and `singleRun: true`. #186 delivered planning/readiness; #192/#193 delivered typed canonical closure. These closed issues are history, not unresolved dependencies.
- [Bootstrap base](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/process/prometheus-context-bootstrap/references/AGENTS.base.md#L34) retains a compiler-check exception; [reanchor](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/process/prometheus-context-bootstrap/assets/hooks/reanchor.sh#L22) restores position but does not explicitly reconstruct VLBV contract/calibration/supersession.
- [Team prompt export](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/process/agent-team-creator/runtime/src/adapters-codecs.mts#L64) carries role, outcome, ownership, dependencies and skills. It has no explicit selected-method/phase-authority propagation.
- [BDD lifecycle](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/testing/bdd-lifecycle-loop/SKILL.md#L52) still mandates failing-test-first execution, and [KBD BDD integration](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/process/kbd-process-orchestrator/references/integrations/bdd-testing.md#L13) prescribes per-change execution. The companion workflow issue owns that procedural update.
- The source and installed package can share version labels while differing in closure semantics. Diagnose actual paths, revisions and content identity; never assume the declared version establishes equivalence.

## Bounded changes

- Discover selected method, authoritative phase/contract, governing instructions and actual host capabilities during Init/Assess. Command inventory is not execution permission.
- Propagate compact references through bootstrap resident/path rules, KBD dispatch, coding-agent creation/export, child handoffs, pause/resume and compaction recovery. Reuse existing prompt/input/skill/handoff fields where sufficient.
- For VLBV, define acceptance/scenarios before generation; finish the complete agreed value phase before executing generated-code checks. A task, hour, commit, reviewer request, compiler problem or renamed slice is not the boundary. Builds, lint/typecheck, hooks and probes cannot disguise prohibited verification.
- Distinguish scenario definition from executable test authoring and execution. Resolve conflicting repository/skill wording explicitly; do not claim a generic skill overrides higher-priority host policy or silently rewrite operator-owned rules.
- Preserve the latest authorized supersession of retired methods, original evidence, candidate identity, consumed budgets, unresolved effects and authority restrictions. Old commands and memory are context, not new execution authority.
- Align cadence with prospectively agreed complete value phases. Publication cadence never certifies incomplete work or authorizes release.
- Describe control honestly: instruction-only, observed, enforced within a named interception boundary, or unsupported. #185 owns trusted tool admission; do not add a generic shell/mutation fence or make that broader work a prerequisite for advisory integration.

## Finite acceptance

1. A fresh session and a delegated coding role recover the same selected method, contract revision, included value, exclusions and stage authority from compact references.
2. Planning/authoring, a task checkpoint, an elapsed hour, a stale next-command and a reviewer request trigger no premature generated-code checks. A governing-policy conflict is surfaced before dependent execution.
3. Compaction/resume preserves current supersession, evidence freshness, budget consumption and unresolved effects; uncertain side effects are reconciled before retry.
4. The completed phase can reach its separately authorized real-path verification using existing contract machinery. Authored code, measured acceptance, demonstrated value, release eligibility and release authority remain distinct.
5. Source, generated and selected installed surfaces preserve semantics, with actual versions/content identities and unsupported surfaces reported. Qualification happens only after this issue’s agreed implementation phase is complete.
6. Closure produces no automatic successor. Optional findings do not silently expand the adopted contract.

## Dependencies and non-goals

Coordinate the companion scenario/BDD workflow and mini policy issue. Reuse open #184 for canonical exit semantics and #185 only for advertised trusted enforcement. No automatic installation, global context rewrite, new daemon, universal harness, blanket test expansion, or product implementation is part of this feature request.

---

## #196: Add value-led scenario discovery and separate BDD authoring from phase-end execution

Source: https://github.com/Prometheus-AGS/prometheus-skill-system/issues/196
Updated: 2026-10-27T01:29:21Z
State at capture: open

## Purpose and priority

**P1.** Deliver a small, usable integration of [VLBV](https://ipfs.prometheusags.ai/ipfs/bafybeihp44qigr3g5zoalkt3p6xvovaglcuzbsp3l4os5vba26glfz6zya/index.html) with three distinct responsibilities: discover useful scenarios, author BDD from approved scenarios, and execute those scenarios at the authorized whole-value-phase boundary.

The owner’s acceptance goals are:
1. Reliably exit KBD phases with the code actually needed.
2. Measure whether the need is met to an agreed level expressed in an exit contract.
3. Establish intended value from how the component or system actually behaves when run.

Depends on the policy semantics in #195. The external VLBV method remains independently usable; this pack supplies routing and adapters.

## Existing capability and precise gaps

Source assessment at `bfe572ff8a97a21c7610c44ea6b39d5f97a857b7`; no acceptance executions were performed for this request.

- [Assess](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/process/kbd-process-orchestrator/skills/kbd-assess/SKILL.md#L151) already elicits the useful operation, baseline, expected result, environment, exclusions and owner. Extend this with relevance calibration and a bounded candidate frontier rather than replace it.
- [Spec](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/process/kbd-process-orchestrator/skills/kbd-spec/SKILL.md#L116) and [Plan](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/process/kbd-process-orchestrator/skills/kbd-plan/SKILL.md#L240) already require a finite adopted matrix and identity-bound packet. #186 is delivered.
- [The schema](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/process/kbd-process-orchestrator/references/schemas/adopted-contract.schema.json#L74) already has stable scenario IDs, owning task, production entry, oracle, negative control, fixtures and evidence. Its boundary packet already fixes `postImplementationOnly` and `singleRun` to true. Reuse and link these fields; no parallel acceptance ledger.
- [bdd-cucumber-js](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/testing/bdd-cucumber-js/SKILL.md#L79) already covers feature files, assertions, step definitions and running. Rust has its own adapter. `bdd-testing` is a deprecated redirect.
- [bdd-lifecycle-loop:54–64](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/testing/bdd-lifecycle-loop/SKILL.md#L54) mandates failing-test-first execution; [KBD BDD integration:17–32,74–80](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/skills/process/kbd-process-orchestrator/references/integrations/bdd-testing.md#L17) prescribes per-change checks and treats exit zero as PASS. Both conflict with the selected method.
- Typed closure is delivered by #192/#193. [Current runtime:797–800](https://github.com/Prometheus-AGS/prometheus-skill-system/blob/bfe572ff/substrate/kbd-runtime/src/phase_closure.rs#L797) refuses cancelled/deferred dispositions without an authorized path. Do not invent a “closed unmet” transition or re-file generic closure repair.

## Small delivery design

### 1. Discovery: one reusable procedure
Provide a small separately invocable discovery procedure, also called by Assess/Spec/Plan. Choose the final skill name during design. Preserve the actor, useful outcome, source requirement or explicit assumption, reason it matters now, credible likelihood or UNKNOWN, mandatory constraints, excluded alternatives and next decision informed.

Calibrate with a few owner-reviewed positive, rejected/deferred and rare-critical examples. Separate mandatory constraints from ordinary priority. Review unfamiliar candidates after meaningful model/context changes. Keep the frontier finite; do not turn every imagined case into required scope.

Output is a proposal until adopted through existing authority. Discovery neither grants execution nor silently adds blocking requirements.

### 2. Authoring: extend existing BDD adapters
Add an explicit author-only path to existing JS/Rust skills; a thin routing entry is acceptable if it improves invocation. Consume approved scenario IDs and produce declarative features, real assertions, step definitions and configuration linked to the contract. Identify the oracle’s source of expected truth and the false positive it excludes.

Acceptance/scenario definitions precede generation. Executable test authoring follows the selected governing policy and never implicitly runs tests, imports test-bearing code, installs dependencies or changes protected scenarios. Resolve current authoring-policy conflicts under #195.

### 3. Execution and closure: thin shared procedure
At the complete agreed value-phase boundary, resolve current contract/candidate, scenario selection, runner capability, required permissions, limits and resource ownership. Invoke the smallest adequate existing real-path runner. Authoring, a task/hour boundary or successful build is not execution authority.

Capture contract/scenario/assertion identity, source/build/configuration/fixture provenance, actual observations, all material attempts, real versus substituted boundaries, restoration and limitations. Challenge material oracle families safely at phase end; returning false or exiting zero without the required assertion cannot establish acceptance.

Classify product failures separately from fixture, permission, prerequisite and evidence failures. Unknown effects remain explicit. Finish any authorized repair implementation before targeted reverification; preserve failures and consumption. Pass evidence to the actual canonical writer.

## Finite acceptance

1. A plausible but irrelevant available test is deferred with a reason; an ordinary useful journey and a rare mandatory constraint are selected from reviewed examples. No automatic scope expansion.
2. An approved requirement maps through stable scenario ID to executable assertion and retained evidence using the existing adopted contract. Any supplementary document is linked and identity-bound where acceptance depends on it.
3. Discovery and author-only invocations execute no generated-code checks. An incomplete phase remains ineligible even if one task is complete.
4. A completed authorized phase runs one bounded real-value journey and a safe sensitivity challenge. A green process lacking the required observation/assertion cannot pass.
5. Wrong candidate/configuration, stale oracle, unavailable runner, unexercised obligation, failed cleanup and uncertain effect receive honest dispositions. Prior failure and targeted repair lineage remain visible.
6. Closure separately reports authored implementation, measured contract acceptance, intended runtime value established, unresolved gaps and release authority. No new task/phase or publication follows automatically.
7. Source/generated/selected installed entry points agree on a finite corpus at this implementation’s completed phase boundary; record versions and untested surfaces.

## Dependencies and boundaries

#195 owns propagation. Open #184 owns canonical finite-exit semantics; #185 owns trusted tool admission where advertised. Closed #186/#192 are delivered foundations, not blockers. Coordinate mini’s compact counterpart and [mini #62](https://github.com/Prometheus-AGS/prometheus-skills-mini/issues/62) for existing shared parity work, without duplicating its telemetry/Windows scope.

Instruction-only integration must say so; a skill cannot override host policy or claim universal interception. No universal harness, replacement runner, new daemon, forced browser/video stack, global prerequisite matrix, automatic repair loop, or generic coverage target.
