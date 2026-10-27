# Assessment: issues-194-195-196

Project: prometheus-skill-system (package name: prometheus-skill-pack).
Assessment date: 2026-10-26, America/Chicago.
Source baseline: `bfe572ff8a97a21c7610c44ea6b39d5f97a857b7`, with preexisting dirty generated OpenSpec integrations, knowledge files and surreal-memory submodule work preserved. No production implementation was changed for this assessment.
Canonical phase: `issues-194-195-196`; assessment entered through typed runtime commands. Phase implementation is 0/0 registered changes, PENDING; this is an unplanned phase, not a completed implementation. No cross-tool phase changes are registered.

## Scope and authority

The user requested this assessment after creating a phase for issues #194, #195 and #196. `goals.md` and `issue-baseline.md` contain its scope and captured issue requirements. This assessment inventories gaps; it neither adopts a finished exit contract nor authorizes implementation, verification, publication, installation, remote agents or successor work.

The repository's Immutable Implementation-First and Integration-Only Policy governs this session. Generated-code builds, lint/type checks, tests, dry-run test imports and probes are deferred until the complete agreed implementation phase. Existing generic skills cannot override it. The bootstrap compiler exception conflicts with selected VLBV semantics and must be resolved in the integration, without silently rewriting operator-owned repository instructions. Reviewer/verifier roles remain dormant until the repository's final implementation boundary. The Assess skill's earlier artifact-review requirement is recorded as pending rather than used to launch a reviewer contrary to that policy.

## Recalled lessons and their implications

From `prior-context.md`:

- “Canonical runtime, not preexisting stale waypoint/project activePhase, is authoritative.” Assessment used canonical status; phase creation has now aligned project metadata and projections. A stale command or recalled phase does not select work.
- “`--repair` checks the selected phase is active before any write and uses the canonical task identity (`c-t1`, not `c/1`).” Preserve these existing protections while investigating #194; task identity repair must not be mistaken for change identity repair.
- “The ~110-second run's external hook effects are unknown.” That is a historical reported incident, not evidence of a current effect. Future reproduction must explicitly isolate hook roots, signer, data paths and resources and reconcile uncertain effects before retrying.
- “`kbd-apply mark-done` ... does **not** sync the runtime ledger.” This recalled historical claim cannot override current source behavior. Inspect current mark-done and reconcile paths rather than replaying an old repair script automatically.
- “The test sourced the helper directly and never exercised installer dispatch.” Qualification must exercise actual production dispatch and selected installed surfaces, rather than infer parity from shared version labels or helper tests.

The recalled previous reflection names an unrelated historical phase. Its unfinished work is not inherited into this phase.

## Implementation status and source evidence

| Area | Status against requested behavior | Evidence and bounded gap |
| --- | --- | --- |
| #194: new change identity | PARTIAL | `substrate/kbd-runtime/src/lib.rs:2073` rejects `ChangeRegistered` when the phase map already contains the change ID. `register_change` at line 4919 emits that event. `kbd-apply.sh:570` checks the active phase's change map before registering. The issue's suspected cause is therefore not established for this checkout. Map keys and embedded IDs, legacy import, affected version and projection identity require investigation. |
| #194: task identity | PARTIAL | `kbd-apply.sh:514` and `task-identity.mjs` reuse registered task IDs by exact ID, normalized title or unique sequence, refusing an ambiguous mapping. This is existing task behavior, not a solution for duplicate change objects. |
| #194: duplicate diagnosis | PARTIAL | `reconcile.mjs:54` aborts with `invalid or duplicate progress change ID`, omitting the offending ID and competing identities. It reads canonical state before checking progress rows. Missing-start-receipt refusal exists in runtime replay at `lib.rs:2236`. Which refusal occurs first depends on the affected data; no reproduction was run. |
| #194: authorized repair | MISSING in inspected command/event path | The inspected change command/event paths expose registration and transition, not an explicit duplicate merge/tombstone repair. Do not delete journal events, edit generated progress or invent a merge command. Determine whether the defect is canonical identity, import or projection before selecting the minimum history-preserving repair. |
| Existing finite contract and closure | PRESENT foundation; runtime acceptance UNKNOWN | `references/schemas/adopted-contract.schema.json:74` has stable scenario IDs, owner task, production entry, oracle, negative control, fixtures and evidence; line 157 fixes `postImplementationOnly` and `singleRun` true. Spec/Plan already require finite contract adoption and bound handoff. `substrate/kbd-runtime/src/phase_closure.rs:775` distinguishes pass/refusal and line 797 refuses cancelled/deferred without an authorized path. Reuse these foundations delivered by #186/#192/#193. |
| #195: selected method propagation | PARTIAL | `kbd-execute/SKILL.md:80` already defers verification until all production changes finish. Bootstrap `references/AGENTS.base.md:34` still permits a blocking compiler exception; its position authority also needs alignment with canonical runtime. Team `runtime/src/adapters-codecs.mts:64` carries role/outcome/ownership/inputs/outputs/dependencies/skills, with no explicit selected-method/phase-authority reference. Existing fields are candidates for compact references, not justification for a new framework. |
| #195: recovery and supersession | PARTIAL | Bootstrap `assets/hooks/reanchor.sh:22` recovers waypoint fields and labels the stored command as intent, but does not reconstruct selected method, calibration or supersession. KBD resume validates plan revision; handoff already carries position and provenance. An explicit compact reference chain preserving contract, candidate, original evidence, consumed budgets, unresolved effects and current supersession remains absent from these inspected paths. |
| #196: relevance discovery | MISSING as requested reusable procedure | Assess already elicits useful operation, baseline, result, environment, exclusions and owner. Spec/Plan already bind finite scenarios. The inspected process/testing surface has no VLBV discovery procedure combining owner calibration, reason-now, likelihood-or-UNKNOWN, mandatory constraints, rejected alternatives and next decision. This is a bounded extension of those entry points. |
| #196: BDD author-only routing | PARTIAL | `bdd-cucumber-js/SKILL.md:79` and `bdd-cucumber-rs/SKILL.md` provide authoring, assertions, steps and runner configuration, but mix authoring guidance with execution commands. No explicit selected-policy author-only path was found in those entry points. Scenario definition and executable test authoring must remain distinct; the repository policy defers test authoring until production implementation is complete. |
| #196: phase-end execution semantics | PARTIAL, conflicting current guidance | `bdd-lifecycle-loop/SKILL.md:54–64` explicitly demands undefined-step runs and failing-test-first production slices. KBD `references/integrations/bdd-testing.md:13–32` schedules per-change checks, and lines 74–80 equate exit zero with PASS. The latter also points at the deprecated bdd-testing alias. These conflict with the requested complete-value-phase boundary and observation-based acceptance. |
| Distribution and selected installation | UNKNOWN behavioral parity | Source, Claude/Codex distributions and installed skills exist. Package and selected CLI both report 1.13.1, while #194 reports 1.11.3. Equal labels do not establish equal content or behavior. No installation or generated-payload qualification was performed. |

Paths in the table are relative to `skills/process/kbd-process-orchestrator/` unless a different root is stated; bootstrap, team and BDD paths belong respectively to `skills/process/prometheus-context-bootstrap/`, `skills/process/agent-team-creator/` and `skills/testing/`.

## Specification alignment

- `openspec/specs/kbd-phase-consistency/spec.md` already requires phase-local progress, typed mutations, real backend progress, completed production before integration evidence and explicit authorization. Preserve this specification while removing conflicting BDD instructions.
- `openspec/specs/kbd-control-plane-reconciliation/spec.md` requires confirmed canonical identity and preserved evidence before moving duplicate compatibility projections. It does not specify merging duplicate canonical changes; do not repurpose projection relocation as canonical repair authority.
- `openspec/specs/agent-team-management/spec.md`, existing handoff schemas and team role inputs are relevant integration surfaces. Reuse their identities and authority limits rather than add another authority ledger.
- The issues' linked VLBV paper was read as an external method source, using read-only HTTP retrieval after the web reader could not access the page. It supports pre-generation acceptance definition, owner-reviewed relevance calibration, mandatory constraints distinct from likelihood, no generated-code checks during generation, bounded phase-end observation, sensitivity challenges, preserved attempts and explicit supersession. It describes method/design; it is not proof of this pack's implementation or an operator authorization. Source: https://ipfs.prometheusags.ai/ipfs/bafybeihp44qigr3g5zoalkt3p6xvovaglcuzbsp3l4os5vba26glfz6zya/index.html .
- No new OpenSpec change or adopted-contract artifact was created during Assess. The issues provide requirements; design and adoption are subsequent work.

## Adopted-outcome inputs and remaining owner decisions

| Input | Recorded basis and limits |
| --- | --- |
| Actor and useful operation | A KBD operator and delegated coding role carry one bounded value phase from planning through fresh-session recovery to authorized real-path verification and honest closure; an operator with duplicated legacy change identities can diagnose and repair them without losing history. These are linked issue outcomes, not a claim that one runner scenario covers every issue. |
| Observed baseline | Current source has the foundations and gaps above. The affected #194 journal, exact command/version/content and key-versus-embedded-ID shape are UNKNOWN. The issue is an operator report, not a local reproduction. Current phase has no registered implementation. |
| Desired observable result | #194: no newly introduced duplicate identity, explicit duplicate diagnostics and a documented authorized repair usable by task boundaries/reconcile. #195: fresh and delegated contexts agree on method, contract, included/excluded value and authority. #196: calibrated discovery, author-only behavior and authorized phase-end execution bind scenario → assertion → evidence, reject a green process without the required observation, and preserve attempts and cleanup outcomes. |
| Environment | Source and generated Claude/Codex entry points, plus explicitly selected installed copies; JS and Rust BDD adapters. Current workstation is macOS. Additional host/platform qualification, exact selected installed paths and the finite conformance corpus must be named prospectively; Windows and every available harness are not implicitly required. |
| Safety and integrity | Preserve journal history, protected scenarios, original evidence, budget high-water marks, unresolved effects and release restrictions. Use actual boundaries, isolated reproduction resources and explicit repair authority; no generic shell/mutation fence. |
| Exclusions | No replacement runner/ledger/daemon, global prerequisite matrix, automatic repair loop, generic coverage target, automatic installation, global context rewrite, product implementation, publication or automatic successor. #184 owns broader exit semantics; #185 owns trusted admission only where advertised. Mini coordination is a dependency input, not permission to modify that repository. |
| Acceptance and risk | The three issues' finite acceptance sets are scope inputs. No owner-reviewed calibration examples, concrete oracle tolerances, chosen installed targets, attempt/resource budgets or final phase contract have yet been adopted here. These remain explicit inputs for Analyze/Spec/Plan, not invented approvals or reasons to stop fact-finding. |
| Decision owner | Requesting repository owner/operator. Agent proposals and recalled memory cannot adopt additional blocking requirements or release authority. |

Instructional VLBV propagation can be instruction-only on this host. No universal interception or technical enforcement is established by this assessment. Existing trusted runtime restrictions may enforce named mutations, but do not thereby intercept all shell tools or agent behavior.

## Build health and integration evidence

- Build/lint/typecheck/test status: UNKNOWN; deliberately not executed during Assess under the implementation-first policy. No inherited PASS is attributed to this phase.
- Existing integration starting points inspected: `shared/scripts/tests/test-kbd-apply-reconcile.sh` (real installed CLI/apply path), `scripts/tests/kbd-phase-consistency.integration.test.mjs`, `scripts/tests/kbd-phase-closure.conformance.integration.test.mjs`, and `shared/scripts/tests/test-kbd-readiness-gate.sh`. Their names alone do not prove coverage of the new requirements; inspect boundaries and substitutions when defining the eventual corpus.
- Coverage of requested #194–196 behavior: UNKNOWN. No numerical coverage claim or blanket suite expansion is warranted. Existing unit/module/mock tests do not establish acceptance.
- Commands actually run for assessment setup/inspection: managed OpenSpec `refresh --project <repo> --timeout-ms 120000` succeeded at 1.15.0 with latestVerified true and authoredPathsChanged empty; local model preflight completed; canonical KBD status/stage commands, CLI help/version, Git/source reads and memory recall. These are lifecycle/inspection observations, not product qualification.
- Model preflight reports `degraded`, `distinct_models: 0`, no provider keys detected and unresolved judge identities. Its printed “only one” explanation does not match the zero count; no distinct-model guarantee is claimed. Provider configuration and remote dispatch were not attempted.
- Constraints C-01/C-04 require deterministic regeneration and applicable drift checks at the eventual completed implementation boundary; selected installation is not automatically authorized. C-02/C-03/C-05 remain applicable to changed surfaces. No production compliance certification is made during Assess.

## Goal progress and dependencies

1. #194: PARTIAL — prevention exists in the inspected current runtime; duplicate-specific diagnostics and an evidence-preserving repair remain unresolved, with root cause unconfirmed.
2. #195: PARTIAL — whole-phase execution and canonical contract foundations exist; method/authority/supersession propagation and recovery are incomplete.
3. #196: PARTIAL — BDD adapters and contract fields exist; discovery/calibration, explicit author-only routing and observation-based boundary procedure are missing or conflict with current instructions. Depends on #195 semantics.
4. Finite adopted contract and whole-phase qualification: NOT MET — inputs recorded, concrete contract not yet adopted; no qualification executed.
5. Preservation and authority constraints: observed for this assessment's scope; no code repair, publication, installation or successor performed. Eventual production acceptance remains UNKNOWN.

Analyze should first distinguish #194's affected-state identity from current registration behavior and inspect actual installed content. It should then resolve the #195 policy conflict and identify the minimum reusable references and #196 discovery/authoring/execution integration. This is a direction for investigation, not an adopted implementation plan or new scope.

## Knowledge gaps recalled by the hook

The following are all entries from `prior-context.md`'s Knowledge gaps section. They are historical context, not newly adopted blockers. Optional learning suggestions are listed; none were executed.

- Claude/Codex memory and learning identity inventory had an unverified Codex runtime probe. Optional: `/learn-goal Claude and Codex hook identity and memory delivery boundaries`.
- The historical revised 19-PR team-learning plan depended on Codex corrective action. Optional: `/learn-goal qualifying cross-harness integration plans with unresolved platform evidence`.
- `shared/scripts/kbd-memory-recall.sh` historically recalled lifecycle metadata instead of lessons (PR B4). Optional: `/learn-goal KBD lesson recall versus lifecycle metadata`.
- `pk context` historically truncated at 43 entries and lacked a worker prompt snapshot (PR A1/A2). Optional: `/learn-goal knowledge recall truncation and retained prompt provenance`.
- Sampled surreal-memory records historically had null agent_id (PR A4/U6). Optional: `/learn-goal attributed memory identity and migration`.
- `scripts/install-binaries.sh` liter-llm block reportedly lacked `|| true`. Optional: `/learn-goal optional tool installation failure semantics`.
- Adversarial packet builder reportedly included all changes until a provisional spec handoff existed. Optional: `/learn-goal binding artifact review packets to the intended phase scope`.
- An orphaned team runtime phase and stale exactNextCommand were recalled. Optional: `/learn-goal canonical KBD position versus stored operator intent`.

Current task-specific unknowns: #194 incident journal/provenance and safe repair identity, owner calibration examples and finite selection thresholds, selected installed targets/content identity, precise phase budgets and retained evidence/cleanup formats. Resolve these before dependent generation or execution, while retaining issue exclusions.

## Review disposition and handoff limitations

Independent adversarial assessment review is PENDING. Repository policy keeps reviewer roles dormant until the completed production boundary; external action policy also does not grant remote-agent dispatch from this assessment request. No independent PASS, cross-model review or final certification is claimed. A local packet/pending receipt preserves the work for later authorized review. These warnings must accompany the next-stage handoff.

Assessment fact-finding is complete. Next suggested stage: `/kbd-analyze issues-194-195-196`. No implementation or automatic successor is selected by this assessment.
