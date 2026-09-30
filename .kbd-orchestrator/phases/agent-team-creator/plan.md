# PLAN: agent-team-creator

Date: 2026-09-24. Projects: prometheus-skill-pack and prometheus-skills-mini; only their isolated codex/agent-team-creator worktrees are writable. OpenSpec backend. Each repository has its own project UUID and canonical ledger. Register TWO changes and SEVEN tasks in EACH ledger: consistency [1.1,2.1,3.1] plus team [1.1,2.1,3.1,4.1]. That is FOUR change records and FOURTEEN task records across the two independent projects. IDs are scoped by project/phase/change; paired delivery does not deduplicate their progress. Canonical tasks are registered at Execute; current implementation count is 0/0, not evidence of completion.

## Ordered changes

1. **kbd-consistency-before-agent-teams** — restore truthful KBD phase creation, progress and current instructions. Depends on: none. Complexity score: High (Rust runtime, Node/Bash helper, generated guidance and state semantics cross boundaries). Model class: frontier. Recommended agent: Codex, current GPT-6 producer. Library: cand-001. Customer value: prevents stale authorization, lost hooks and false completion.
   - Task 1.1: complete production corrections before any test authoring/running. Root owns source instructions/constraints and orchestration. Bounded workers may own full substrate/kbd-runtime projection source, mini scripts/kbd-new-phase.mjs, and full existing Bash phase helper respectively; no concurrent edits to shared files, no worker Cargo builds. Preserve run journal and existing test files.
   - Task 2.1: after all corrections are coherent, add new real-process/filesystem integration scenarios. Root serializes all local builds. Exercise actual Runtime public API, journal replay, repeated projection, stage/parent and earlier run evidence; invoke both production phase helpers against actual canonical CLI and configured real hook, including malformed metadata and legacy modes. No mocks, no test-first, no protected test edits.
   - Task 3.1: deterministic QA from current governing rules, independent cumulative diff review from initial branch base, fix findings, record evidence, verify/archive and commit. Distribution reconciliation is explicitly deferred to agent-team-creator task 2.1 within this same phase under full C-01. No distribution certification or push until reconciliation finishes.

2. **agent-team-creator** — guided creation and portable management across eight native harnesses. Depends on completed, reviewed consistency change. Complexity score: High (new cross-harness artifacts, ownership and model/memory boundaries). Model class: frontier. Recommended agent: Codex, GPT-6. Libraries: cand-002..004 and cand-006..015. Customer value: novices can choose a minimal team, experts can retain native configuration, and teams can hand work across tools.
   - Task 1.1: complete self-contained Node >=22 / TypeScript 7.0.2 runtime in full creator skill; source and emitted .mjs copied byte-for-byte to mini. Role discovery, explicit tier/capability/cost model policies and liter-llm catalog mapping, eight native exports, local task revision/ownership, accepted handoff receipts, optional memory with durable local fallback. Native options use explicit native configuration/file escape hatches; no claim of identical permissions or team execution semantics. UAR/BossFang source contracts in uar-bossfang-contracts.md are separate. Registration/activation/execution are distinct; no live mutation or global installation in verification.
   - Task 2.1: finish four skills, native agent/plugin/marketplace support and capability tables, all options through native overrides and versioned documentation references, root READMEs, docs/codex-plugin.md and the relevant CLAUDE.md Codex CLI Integration section when that surface changes, and both Docusaurus guides/catalogs/navigation. This is the final generated-surface reconciliation owner for BOTH changes. Regenerate via existing inventory/plugin/rule/doc tooling, never hand-edit output.
   - Task 3.1: only after coherent production and docs, exercise compiled packaged CLI across actual process/filesystem boundaries: guided simple vs complex outcomes, eight exports/native escaping, unsupported fields and collisions, concurrent revision/ownership conflicts, cancellation/reassignment, handoff acceptance and source receipts, optional memory unavailable/retry, model unknown price/capabilities and constraint selection, KBD-linked identity. No network mutation to product services. Verify native formats where actual tooling is installed; report source-only or unverified support otherwise.
   - Task 4.1: QA then independent cumulative diff review, resolve all criticals, verify/archive, commit/push both branches after local gates, verify remote SHAs and KBD reflect. Do not conflate actual push with deployment, installation or merge.

## Execution and ownership

KBD owns task boundaries through mini scripts/kbd-apply.mjs (usable with either cwd) or full driver. Typed prometheus kbd change/task commands register canonical work. One task is active per change; worker subtasks are inside that single boundary. Root records completion only after inspecting outputs. Generated progress and waypoint are never manually edited. Hooks run on actual boundaries; failures are recorded, never invented as success. Existing source fixes may retain their existing Rust/Bash/Node language; all NEW team hooks/scripts are TypeScript 7 on Node.js.

## Exact verification surfaces

Correction: cargo test --manifest-path substrate/kbd-runtime/Cargo.toml --test phase_scope (new full production integration target); node --test scripts/tests/kbd-phase-consistency.integration.test.mjs (new full process integration); node --test scripts/kbd-new-phase.integration.test.mjs (new mini process integration). These are authored only after task 1.1. Build a local corrected prometheus binary if projection refresh is needed, never overwrite installed binaries. Before Cargo, inspect active local compiler processes and serialize.

Team: node --test <creator>/tests/*.integration.mjs against compiled entry point and its real file boundary. Full only: avoid broad unit-test commands under its immutable policy. Mini keeps its separately mandated final compatibility suite. Full strict skill validation for each of the four paths; npm run validate:plugins; npm run validate:codex; npm run generate:skills-index; npm run check:skills-index; npm run build:distribution then direct generator --check (avoid broad scripts that implicitly run unit tests); npm run docs:sync and docs:sync:check; npm --prefix site run build. Mini: node rules/build.mjs; npm run check; npm test once after coherent production as the existing compatibility gate (its unit tests do not count as acceptance evidence); npm run build:distribution and check:distribution; node --test scripts/tests/skill-system-distribution.test.mjs if it exercises real generation; npm run spec:validate; site catalog generator then npm --prefix site run build. Confirm exact script paths against package scripts when executing. Record command and result, not blanket all tests passed.

At final boundary regenerate affected outputs twice and compare tracked content hashes. Final ordering: complete QA/review and archive, create local commits, run full protected-test integrity check from those committed SHAs, then push. If a check requires fixes, commit the fix and re-run the implicated committed-state gate before push. No bypass flags. Scope OpenSpec validation to both new changes; the broad scan observed pre-existing unrelated full change failures, not blockers attributable to this work. Both new changes validate strictly. Review packets exclude their own receipts but include cumulative source diff and both repository constraints plus operator authorization. A passing anti-theater screen is not a passing substantive review.

## Limits and decisions

No remote product registration, no Cedar implementation claims, no UAR/BossFang/The Boss edits, no host installs. Mini default reference-only full-pack policy is overridden solely by this explicit two-repository phase. Personal/team knowledge-log publication stays forbidden. Full local-only validation and mini CI policy retain distinct scopes; all evidence here is local. macOS execution does not certify Windows. Native CLI availability determines live format evidence; source preservation does not imply semantic validation of unknown options. Existing optional services may be down; local creation/management must remain usable.

## Review and completion

Spec review PASS; warnings resolved for supported marketplaces and truthful export verification status. Analyze review findings remain preserved with dispositions and per-harness candidates. Plan review must finish before Execute. Each change requires QA and substantive diff review before archive; final phase requires regeneration, docs builds, branch publication and reflection. No skipped lifecycle stages, synthetic predecessor handoffs or reset of historic completion.

## Unresolved review findings (two-round limit)

The second review returned BLOCK on task cardinality. Original finding is retained verbatim below; the clarification above names all seven tasks in each independent project ledger (fourteen records across projects). This was a scope ambiguity, not seven versus fourteen tasks in one ledger. Verify registration totals before starting execution. Other first-round findings were resolved before round two. No claim of Plan review PASS.

```json
{
  "mode": "artifact",
  "verdict": "BLOCK",
  "judge_model": "gpt-5.5",
  "producer_model": "gpt-6",
  "isolation_mode": "rest-gateway:http://localhost:4000/v1",
  "cross_model_check": "verified-distinct",
  "findings": [
    {
      "severity": "CRITICAL",
      "file": "plan.md",
      "line": 0,
      "claim": "The plan gives contradictory task cardinality, so Execute cannot register unambiguous canonical KBD tasks/progress totals.",
      "evidence": "The opening states \"Two changes, seven tasks per repository,\" but the ordered changes list only seven task bullets total: three under `kbd-consistency-before-agent-teams` (`Task 1.1`, `Task 2.1`, `Task 3.1`) and four under `agent-team-creator` (`Task 1.1` through `Task 4.1`). This conflicts with the same paragraph's requirement that \"Canonical tasks are registered at Execute\" and the execution section's \"One task is active per change.\"",
      "suggested_fix": "State the exact canonical task list and totals for Execute: either seven total deduplicated cross-repository tasks, or fourteen per-repository tasks with distinct IDs/owners for full and mini."
    }
  ],
  "checked_classes": []
}
```

## Execution correction — backend progress failure
Task 1.1 end-task reproduced a false final-change message when the global OpenSpec launcher failed. Actual canonical tasks 2.1 and 3.1 remain pending. Verification task 2.1 includes the minimal root-cause fix in both existing apply drivers before integration authoring, plus real process coverage with an actual OpenSpec success and missing-backend failure. No new phase, task count, service or dependency. Original hook receipt is retained as historical evidence; it is not acceptance or certification. Final cumulative review covers this amendment.

## QA corrections discovered at the completed-change boundary
The adapter progress contract now rejects missing, negative, non-integer or inconsistent counts instead of manufacturing completion. Mini stage-gate remediation names its real Node API. Mini compatibility run reproduced a stale expectation in lib/doctor/skills.test.mjs: it demanded overwriting user edits after commit d3fa2b5 explicitly changed production to preserve them. This unprotected legacy compatibility assertion is aligned with that committed behavior (preserved bytes and continued drift); no full-pack protected test is edited and no acceptance gate is removed. Mini distribution mirrors are regenerated now to satisfy its per-change compatibility gate; final phase reconciliation still belongs to team task 2.1. Whole-tree constraint scans also match pre-existing source and archived review text; change-scope provenance is recorded, with no global cleanup or blanket clean claim.

## Independent review round1 correction
Preserve unrelated tracked phase histories by restoring their baseline files through Git, never rewriting canonical events; automatic projection refreshes from shared state are outside this delivery scope. Execute dispatch records execution.md and execute-dispatch.json only. The completion handoff for Reflect is emitted after the entire phase execution and its required QA/review/archive gates, not when dispatch is prepared. Existing premature receipt is retained as historical dispatch evidence with completion:false. Task3.1 stays IN_PROGRESS through this review.

## AgentSkills metadata compatibility
The published AgentSkills specification requires string-valued metadata and places version there. New skills use that format. Adapt full strict validation to accept metadata.version and nonempty string tags while retaining legacy top-level version/array tags; both site catalogs read either representation. This resolves an observed compatibility conflict without rewriting existing skill metadata. Source: https://agentskills.io/specification (2026-09-24). Verification exercises the actual strict validator and generated catalog after production completion.

## Packaging compatibility correction
The completed mini compatibility run exposed source-cache and resource-root assumptions in two existing unprotected integration checks. Distribution parity now excludes declared build/cache directories only on the source side and inspects every packaged byte, so leaked dependencies still fail. Carried-resource validation resolves the nearest AgentSkills root as well as legacy repository resources and ignores installed development dependencies. Companion runtime references are explicitly qualified. No production behavior or protected full test is relaxed.

## Completion boundary
Task 4.1 completes the reviewed implementation after deterministic QA and independent review pass. Its aggregate title also names publication and reflection, which are phase postconditions: verify/archive the completed change, commit, certify committed protected-test integrity, push and confirm remote SHAs, then complete Execute and Reflect. These outcomes have separate dated receipts. Requiring reflection before this task closes would form a cycle because Reflect requires completed Execute and archived changes. The implementation counter must never be presented as proof that publication or reflection already happened. Preserve the canonical task title as historical intent; the completion summary records this narrower implementation boundary. The phase remains active until every postcondition is actually satisfied.
