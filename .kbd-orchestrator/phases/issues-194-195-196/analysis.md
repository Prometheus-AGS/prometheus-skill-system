# Analysis: issues-194-195-196

Date: 2026-10-27, America/Chicago. Source: `bfe572ff8a97a21c7610c44ea6b39d5f97a857b7` plus preserved preexisting dirty files. Inputs: `assessment.md`, its completed handoff, `goals.md`, `issue-baseline.md`, and refreshed `prior-context.md`. Mode: existing stack, Rust signed runtime/CLI plus Bash/Node/TypeScript skill integration. No stack replacement or new external dependency is recommended.

## Main conclusion

Adapt the existing signed KBD runtime/apply path for #194; retain the existing finite adopted contract and typed closure for #195–196; extend bootstrap/team/handoff references and the existing JS/Rust BDD skills. Add one small discovery procedure and explicit author-only versus phase-end execution routes. No external framework solves the pack's authority, repair-history or relevance-calibration requirements.

`library-candidates.json` records seven candidates: one adopt, four adapt, one reference and one reject. These are analysis recommendations, not contract adoption, execution permission or acceptance results.

## #194: distinguish three identity problems before choosing repair

Current registration already refuses an existing phase-map key (`substrate/kbd-runtime/src/lib.rs:2073`), and apply checks that key before registering (`kbd-apply.sh:570`). The issue's claim that ordinary current registration necessarily creates a second change remains unproven.

Two additional mechanisms are visible in source:

1. Legacy conversion (`lib.rs:6897`, `legacy_phase`) collects change rows into a `BTreeMap` keyed by embedded ID. Repeated array IDs collapse to one value; object rows retain a supplied embedded ID even if it differs from the object key. The subsequent collection can overwrite earlier rows. This is a static data-loss risk on a duplicate input, not a reproduced explanation for two rows in the issue's status output.
2. `LegacyStateImported` (`lib.rs:1962`) copies imported phase maps; the inspected arm does not itself validate every map key against each embedded ID. A malformed externally constructed imported map therefore requires explicit invariant inspection. Ordinary legacy conversion tends to collapse duplicate IDs, so these paths must not be conflated.

The error text has a separate explanation. CLI `guard_context` at `tools/prometheus-cli/crates/prometheus-cli/src/commands/kbd.rs:805` scopes resolution to the active phase. A task subject is first split into change/task map keys. If that lookup fails, it falls back to matching the entire subject against task ID/title values; zero matches and multiple matches both produce the same “not a unique canonical work item” finding at line 1151. Thus the finding does not establish that two changes exist. Change boundaries also match ID or title, allowing an ID/title collision to be ambiguous. A backend ordinal different from the registered task key can fail qualified lookup; the existing task mapper must remain in use.

The missing-start finding is produced by CLI obligation lookup as well as guarded by runtime receipt processing. Reconcile can encounter that refusal without having diagnosed the underlying identity. `reconcile.mjs:54` also emits a generic duplicate progress-row error without naming the repeated ID. These are distinct diagnostic layers.

Recommended design constraints for Spec:

- Inspect and report phase ID, map key, embedded change ID, task keys, backend identity, projection identity and outstanding receipt subjects before any duplicate repair. Distinguish duplicate embedded IDs, duplicate projection rows, key mismatch, missing task and ambiguous title.
- Preserve the existing same-key registration rejection or explicitly return the existing equivalent record. Never silently replace its tasks/status. Refuse conflicting metadata with the actual identity named.
- For confirmed canonical damage, use an explicit typed signed repair command/event in the current runtime, bound to phase, source revision and exact source/target identities. Preserve all source records, evidence, task outcomes and receipt lineage. Define collision/conflict handling and counters prospectively; do not infer completion or generate a passing start receipt from an implementation flag.
- For projection-only damage, use canonical projection repair with preserved diagnostic provenance; do not introduce a canonical merge when none is needed. For legacy duplicate inputs, preserve/refuse before lossy conversion rather than silently choosing the last row.
- No journal deletion, direct progress editing, generic history rewrite or automatic repair retry.

The affected `usefulness-update` journal, its 1.11.3 binary/content identity and exact commands are unavailable in the current phase inputs. No matching local phase was found in the scoped repository search. The incident-specific root cause and exact survivor mapping remain UNKNOWN. Spec can define safe finite behavior without asserting this cause; any live repair must wait for the affected evidence and mapping. No hypothetical merge CLI is presented as already available.

## #195: propagate references through existing surfaces

Candidate 003 uses existing role `inputs`, prompt instructions and selected source/evidence file provenance, plus KBD handoff `readInOrder`, `targetChecks`, `warnings` and `startPrompt`. `handoff-provenance.mts` already hashes selected files and explicitly states that observation does not transfer authority. Prefer a compact phase-policy reference tied to the adopted contract/decision and supporting identity-bound artifacts. Add structured fields only where existing references cannot preserve required semantics; do not create an independent policy authority.

The recovered view must identify selected method, complete included value, exclusions, current contract revision/digest, governing policy and allowed stage operations. It must also link current supersession, calibration, candidate/evidence freshness, consumed budgets and unresolved effects. Mutable files require content identity and re-resolution of the latest authorized supersession; a digest of a stale pointer is insufficient. Pause/resume and child/harness handoff preserve these facts without granting fresh execution authority.

Resolve policy conflicts explicitly. VLBV forbids generated-code verification during generation, including compiler checks disguised as unblockers. The repository owner’s immutable policy separately retains a narrow compiler exception. This analysis does not edit or repeal that policy. The integration must surface a selected-method/host conflict before dependent execution, and use the stricter compatible course here: no generated-code check during this analysis or subsequent generation. Generic BDD failing-test-first instructions cannot override the host. Scenario definitions may precede generation; executable test authoring follows the governing policy, which here requires completed production first.

Control claims remain bounded: compact instructions and recovery references are instruction-only; content hashing observes identity; signed KBD commands enforce their named runtime mutations. None establishes universal shell interception. #185 is not a prerequisite for this advisory integration. Publication cadence cannot shrink the adopted complete value phase or grant release authority.

## #196: adapt BDD capabilities and add only the missing procedure

The reusable discovery procedure should consume the actor/outcome/baseline inputs already elicited by Assess, then propose a finite frontier. Retain source requirement or explicit assumption, reason-now, likelihood or UNKNOWN, mandatory constraints, rejected/deferred alternatives and the next decision informed. Use a few owner-reviewed ordinary-positive, irrelevant/rejected and rare-critical examples. Calibrated examples are adoption inputs, not automatically blocking requirements. Do not invent probabilities or a weighted score. Meaningful model/context changes prompt review of unfamiliar candidates without opening an unlimited frontier.

The discovery output stays proposed until existing contract adoption. The current `scenarioMatrix` already owns scenario identity, task owner, production entry, oracle, negative control, fixtures and evidence; existing budgets/amendments/workerRoutes/boundaryPacket provide the remaining authority anchors. Any supplementary calibration/policy document affecting acceptance needs a contract-bound identity. Whether a small additive schema reference is necessary is a Spec decision, not grounds for another ledger.

For author-only invocation, extend `bdd-cucumber-js` and `bdd-cucumber-rs` to consume approved IDs and emit features, actual assertions, step definitions and configuration. Explicitly prohibit execution, test-bearing imports, dependency installation and protected-scenario edits. Cucumber’s documented dry-run still loads support code, skips steps and can exit successfully with undefined steps; it is neither no-execution authoring nor acceptance evidence. [Pinned Cucumber.js documentation](https://github.com/cucumber/cucumber-js/blob/edaa0bb17a0b9a6004d6179741597e37db87465e/docs/dry_run.md).

At the complete authorized value-phase boundary, a thin shared procedure resolves contract/candidate identity, selected scenarios, runner availability, permissions, limits and resource ownership. Invoke existing runners; retain scenario/assertion mapping, provenance, observations, every material attempt, actual/substituted boundaries and restoration. Use a safe sensitivity challenge against material oracle families; a green process without the required observation/assertion cannot establish acceptance. Distinguish product failure from fixture/permission/prerequisite/evidence failure. Preserve uncertain effects and repair consumption; finish the agreed repair before targeted reverification. Send supported results to the actual existing canonical writer.

Existing Rust Cucumber output already supports Cucumber JSON, JUnit and libtest; there is no reason to build a new runner or force a browser/video stack. [Pinned cucumber-rs output documentation](https://github.com/cucumber-rs/cucumber/blob/d95872541ee2a43d71089c64bd3fa8848e24c922/book/src/output/index.md). Do not assume a JSON formatter alone proves the assertions.

## Build-versus-adopt and readiness mapping

| Candidate | Recommendation | Contract sections supported | New work needed before execution readiness |
| --- | --- | --- | --- |
| 001 signed runtime/apply/reconcile | Adapt | scenarioMatrix, evidenceInvalidation, boundaryPacket | Targeted diagnosis, loss-preserving import behavior and signed repair semantics; cannot fulfill #194 solely with docs. |
| 002 adopted contract/decisions/readiness/closure | Adopt | identity, adoptionSource, scenarioMatrix, budgets, workerRoutes, boundaryPacket | Existing authority is reusable. Do not reopen #186/#192/#193; extensions only for identified missing references. |
| 003 bootstrap/team/handoff provenance | Adapt | workerRoutes, amendmentPolicy, evidenceInvalidation, identity, budgets | Compact method/authority/supersession reference integration and governing-policy conflict handling. |
| 004 Cucumber.js adapter | Adapt | scenarioMatrix, boundaryPacket | Explicit author-only and phase-end routes; retain oracle observations and provenance. |
| 005 cucumber-rs adapter | Adapt | scenarioMatrix, boundaryPacket | Same separation with targeted Rust production entry; no new general runner. |
| 006 independent VLBV method | Reference | functionalScope, exclusions, scenarioMatrix, amendmentPolicy | Small discovery/calibration procedure and contract-bound supporting references. |
| 007 replacement runner/orchestrator/ledger | Reject | Existing authority already covers these needs | Adds excluded scope and duplicate authority. |

The eventual implementation phase cannot reach all requested outcomes without some production runtime/CLI work for #194. #195–196 predominantly need integration/procedure changes; technical enforcement beyond named runtime commands remains outside scope. READY to spec does not mean READY to execute or verified product behavior.

## Research record and installed identity

Bounded research: at most 8 queries per tier and 20 minutes. Tier 1 used 5 requests: two GitHub MCP searches failed authentication; CLI fallback had one empty query and two successful official-repository searches. Tier 2 used 6 requests: Context7 resolve/query, two official web opens (Rust book unavailable), and two pinned raw-document opens. Tier 3 used 2 read-only registry requests. Tier 4 was unnecessary. No budget cap was reached; search was stopped when existing runners and local authority answered the build/adopt question. Local source reads are not external search queries. VLBV paper evidence was inherited from Assess, not counted as newly fetched research.

Registry observations: npm reports Cucumber.js 13.3.0, MIT, metadata modified 2026-10-05; crates.io reports cucumber 0.23.0, MIT OR Apache-2.0, updated 2026-04-23. These are maintenance signals, not compatibility certification or an upgrade instruction. Preserve consumer pins. No downloads/issue counts are used as quality scores.

`installed-identity.json` records absolute source/installed paths and SHA-256 for seven selected KBD files under `/Users/gqadonis/.codex/skills/kbd-process-orchestrator`; all seven currently match their source bytes. This narrows the earlier installation unknown only for this exact subset. It does not qualify generated distributions, team/bootstrap/BDD copies, binary/source equivalence or behavior, and does not establish the incident’s 1.11.3 environment. No global copy was changed.

## Lessons, open decisions and finite qualification direction

Recalled lesson: “issue #194 incident root cause remains unconfirmed without affected journal/version and map-key versus embedded-ID inspection.” This analysis retains that uncertainty while identifying concrete source paths. Recalled lesson: “Reuse existing finite contract and typed closure.” Candidate 002 implements that reuse recommendation. Historical memory about mark-done synchronization is context, not permission to replay old repair scripts. Unrelated mobile/MCP/team-learning work remains out of scope.

Before final contract adoption, name the selected installed/generated entry points, owner-reviewed calibration examples, concrete budgets and exact observable oracles. A bounded proposed qualification set can cover: duplicate registration/import/repair preserving history; fresh/delegated/resumed policy agreement and conflict refusal; ordinary/rejected/rare-mandatory scenario selection; discovery/author-only invocation with no generated-code execution; one real value operation plus safe oracle challenge; and wrong/stale candidate, missing assertion, cleanup failure and uncertain-effect dispositions. These are candidate families for owner calibration, not newly adopted scenarios or permission to author executable tests now. Preserve required failures, attempted repairs and no-successor behavior without expanding into a blanket matrix.

Knowledge gaps recalled by the hook remain historical and unverified here. Each optional suggestion is informational, not executed:

- Codex/Claude memory identity inventory had an unverified Codex probe: `/learn-goal Claude and Codex hook identity and memory delivery boundaries`.
- Historical cross-harness team-learning plan depended on Codex corrective action: `/learn-goal qualifying cross-harness integration plans with unresolved platform evidence`.
- Recall reportedly mixed lifecycle metadata and lessons: `/learn-goal KBD lesson recall versus lifecycle metadata`.
- pk truncation and absent worker prompt snapshot: `/learn-goal knowledge recall truncation and retained prompt provenance`.
- Null agent_id migration: `/learn-goal attributed memory identity and migration`.
- Optional liter-llm installer failure semantics: `/learn-goal optional tool installation failure semantics`.
- Review packet scope depended on provisional handoff: `/learn-goal binding artifact review packets to the intended phase scope`.
- Orphaned phase and stale exactNextCommand: `/learn-goal canonical KBD position versus stored operator intent`.

## Status and recommendation

Analysis artifacts are complete; generated-code health, integration acceptance, protected-test certification and behavioral distribution parity remain UNKNOWN. No build, test, lint, doctor, generated-code import or runner probe was performed. Lifecycle preflight refreshed OpenSpec 1.15.0 and reported no authored-path changes. Independent artifact review remains pending under the repository completed-production reviewer boundary; the prior degraded model preflight supplies no distinct-model assurance. No remote agent was dispatched.

READY to spec. The design is bounded and the reusable foundations are identified. Spec must preserve the incident-specific UNKNOWNs, resolve the policy-reference representation and define finite acceptance/adoption inputs before generation. Live #194 incident repair remains blocked on confirmed affected-state evidence and exact mapping; this does not block specifying safe diagnostic/repair behavior. Recommended next command: `/kbd-spec issues-194-195-196`.
