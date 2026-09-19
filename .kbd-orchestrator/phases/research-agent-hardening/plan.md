# PLAN: research-agent-hardening

Project: prometheus-skill-pack (Prometheus skill system 1.8.0)
Date: 2026-09-04
OpenSpec available: YES (directory present) — change backend: **native-kbd**, per the spec handoff and the control-plane-to-companion precedent (the review packet and kbd-apply read `.kbd-orchestrator/changes`); the two OpenSpec capabilities decided at analyze are authored inside changes 002 and 008 as `openspec/specs/*/spec.md` and validated with `openspec validate`.
Changes to implement: 11 (47 tasks), all already emitted at spec stage under `.kbd-orchestrator/changes/change-rah-*/`.
Inputs: `assessment.md` (5 of 5 goals NOT MET), `analysis.md` and `library-candidates.json` (11 candidates, 13 build items), `handoffs/spec.handoff.json`, `review/spec/disposition.md`.

## CHANGE LIST (ordered)

1. change-rah-001-compile-baseline-and-timestamps: record the compile baseline for both crates and replace fabricated timestamps
   - Scope: rust (prometheus-research) | tests
   - Depends on: NONE
   - Recommended agent: Codex (focused implementation)
   - Est. complexity: S
   - Complexity score: Low
   - Model class: small
   - Customer value: MEDIUM (unblocks every Rust verdict; visible only as correct timestamps)
   - library: cand-011 (chrono 0.4, same pin as learner-model)
   - Details: Task 1 is the phase's first gate: both `cargo check` commands run only when `pgrep -x cargo` is empty, and their full output is saved to `.kbd-orchestrator/changes/change-rah-001-compile-baseline-and-timestamps/evidence/compile-baseline.txt`, which is the persisted execution evidence the gate is judged on. Then adopt chrono, delete the 1970 formatter and the hardcoded 2026-07-08 literal, and extend `tests/job_lifecycle.rs` with the RFC 3339 assertion. If `learner-model` fails to compile, repair it here (spec open question, default here).

2. change-rah-002-research-package-contract: one normative package spec with a JSON Schema, one output root, and a drift check
   - Scope: docs | scripts | hooks | rust (two constants) | openspec
   - Depends on: change-rah-001
   - Recommended agent: Codex
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH (every later change reads this contract)
   - library: cand-005 (python3 jsonschema, jq fallback), cand-006 (jsonschema crate, consumed by 004)
   - Details: Write `research-manifest.schema.json`, delete the template, rewrite `export-package.sh` to emit real state, move the four deep-research hook scripts (`skills/research/deep-research/hooks/{pre-research,post-stage,on-contradiction,post-export}.sh`) and the daemon to `~/.prometheus/research/`, fix the four documentation drifts, add `check-research-package.sh`, author the `research-pipeline-execution` capability. No generator is introduced (D-13). The top-level `hooks/hooks.json` and `hooks/codex-hooks.json` are not touched by this change or any change in the phase; "hooks" in every scope line below means deep-research's own scripts.

3. change-rah-003-stage-contract-driver: replace the no-op driver with a stage-contract enforcer
   - Scope: scripts | hooks | references | tests | shared lib | openspec
   - Depends on: change-rah-002
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH (this is the change that makes G1 true for every harness user)
   - library: cand-003 (Feynman CLI patterns: slug rule, sidecar, plan ledger, scale gate)
   - Details: `stage-contracts.md` is written first and is the table the driver enforces. The driver creates the per-run `plan.md` with task ledger, verification log, and decision log sections and updates the ledger and verification log at every stage boundary (G3's plan-as-ledger requirement; spec task 2), runs stages through `RESEARCH_STAGE_RUNNER` or stops in checkpoint mode, validates every stage's artifacts, fires all four deep-research hook scripts at their defined points, honours `--resume`, `--check-tools`, `--scale`, gates 06 on 05, and writes the provenance sidecar from an exit trap. Six integration scenarios in `tests/driver-contract.sh`, including bash 3.2; the full-run scenario asserts the ledger holds one row per executed stage and the verification log one entry per hook exit code.

4. change-rah-004-daemon-job-execution: implement `--daemon-job` through a headless harness and a validated export
   - Scope: rust (prometheus-research) | tests | references
   - Depends on: change-rah-001, change-rah-003
   - Recommended agent: Claude Code
   - Est. complexity: L
   - Complexity score: High
   - Model class: frontier
   - Customer value: HIGH (background research from the MCP tools and the SSE UI)
   - library: cand-004 (headless `claude -p` or `codex exec`), cand-006 (jsonschema crate)
   - Details: Real clap argument, harness resolution from `PATH` with `blocked` on absence, checkpoint mirroring into SSE, `research_export` validating the manifest. Hook policy for the daemon path (resolves the analyze open question): the daemon fires no hooks itself; the headless child runs the real driver from 003, which fires the four deep-research hook scripts exactly as in the foreground, and the child never fires KBD lifecycle hooks (the prompt sets `KBD_HOOKS_DISABLED=1` and `references/headless-execution.md` records the rule). `tests/job_execution.rs` uses a fake harness that runs the real driver with the fixture runner from 003 and asserts the four hook markers under `RESEARCH_HOOK_LOG` after the job completes. Cargo-gated: run tests only when no other build is active.

5. change-rah-005-claim-labels-and-provenance: four-label vocabulary on every claim and on feynman-loop artifacts
   - Scope: references | stage skills | agents | scripts | learn skills | fixtures
   - Depends on: change-rah-003
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH (this is what makes a report's confidence legible)
   - library: cand-003 (Feynman CLI label vocabulary and verifier rules, attributed)
   - Details: Label enum and `research-graph.schema.json`, derivation rule for the package label, stage 05 to 09 and three agents updated, `check-research-package.sh` validates labels with a labelled fixture and an unlabelled negative fixture, `write-artifact.sh` refuses artifacts without verification and provenance blocks.

6. change-rah-006-agent-duties-and-report-review: tool allowlists, verification before review, adversarial-review on the report
   - Scope: agents | scripts (driver, adversarial-review packet builder) | tests
   - Depends on: change-rah-003, change-rah-005
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH (the producer never grades its own report)
   - library: cand-008 (adversarial-review skill, new `research` target), cand-003 (writer network isolation pattern)
   - Details: `tools:` frontmatter on the four agents with restated allowlists, `research` packet target, driver runs the review between 09 and 10 only after stage 05 validates, with BLOCKED, WARNING, judge-unavailable, and review-refused handling proven by three driver scenarios and the adversarial-review fixture suite.

7. change-rah-007-eval-ground-truth-review: human review of the 24 eval items and the pre-change baseline
   - Scope: learn-grade eval dataset
   - Depends on: NONE (may start in Round 1 and run alongside 001 to 006)
   - Recommended agent: Manual (operator) with Claude Code preparing the sheet and re-running metrics
   - Est. complexity: S for the agent; operator time is the real cost
   - Complexity score: Low
   - Model class: small
   - Customer value: MEDIUM (protects the grader's measured accuracy from being re-baselined against unchecked labels)
   - Details: Generate `REVIEW-SHEET.md`, wait for the operator to mark all 24 items reviewed in `index.json`, then regenerate `baseline-snapshot.json` and `metrics-summary.json`. This is an operator gate: 008 must not start until `reviewed == 24`.

8. change-rah-008-learn-artifact-and-corpus-coherence: one artifact path and a corpus that carries what learn-grade reads
   - Scope: learn skills | shared script | tests | openspec | eval baseline
   - Depends on: change-rah-007, change-rah-005
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH (the learn loop cannot close today because its skills cannot find each other's files)
   - Details: `artifacts/<concept-id>/<artifact-id>.json` across three skills, `key_points[]` and `misconceptions[]` from the shared grounding script with the two copies reduced to wrappers, `learn-coherence.sh` proving the round trip, the `learn-model-coherence` capability, and the post-change eval re-baseline recorded rather than tuned.

9. change-rah-009-learner-model-write-paths-and-fsrs: gap, session, and certification RPCs and a live FSRS scheduler
   - Scope: rust (learner-model) | tests | learn skills | openspec
   - Depends on: change-rah-001, change-rah-008
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: High (CRDT fold semantics and a dependency decision)
   - Model class: frontier
   - Customer value: MEDIUM (learn-certify's session gate finally has data)
   - library: cand-001 (rs-fsrs 1.2.1, adopt if `cargo tree` confirms it is scheduler-only), cand-002 (fsrs-rs, reference)
   - Details: The three RPCs, `certified_at`, the store fold, and `tests/rpc_roundtrip.rs` driving the built binary over stdin and stdout are the G2 work. The FSRS replacement (tasks 1 and 3: `cargo tree` decision and scheduler swap) is **optional within this change**: it fixes assessment learn finding 26 (`difficulty` stored but never read) but no goal requires it, so the phase certifies without it. If it is skipped, tasks 1 and 3 are marked skipped with the reason and the rpc_roundtrip difficulty-delta assertion is omitted; the plan's "nothing outside the five goals" claim is accurate for the required tasks only. Cargo-gated.

10. change-rah-010-source-scoring-and-graph: rubric scoring with renormalisation and sensitivity, content-addressed claims, contradicts edges
   - Scope: scripts | stage skills | references | fixtures
   - Depends on: change-rah-005
   - Recommended agent: Codex
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: MEDIUM (G5; ordered last among goal work by analysis decision)
   - library: cand-003 (Feynman CLI `combineSignals`, sensitivity profiles, claim ids, ported to python3), cand-010 (`kbd_complete` for semantic contradictions)
   - Details: `score-sources.py` with `appliedWeights` and `sensitivity.json`, `build-graph.sh` in the spec shape with `claim:sha256[:16]` ids and `contradicts` relations, semantic contradiction path labelled `inferred` or `blocked`, five fixture scenarios in `tests/scoring-graph.sh`.

11. change-rah-011-integration-evidence-and-docs: real end-to-end runs, documentation, version bumps, C-01 reconciliation
   - Scope: docs | site | skill versions | skills index | evidence document
   - Depends on: change-rah-004, change-rah-006, change-rah-009, and change-rah-010 only if 010 is not deferred (see SCOPE CUTS)
   - Recommended agent: Claude Code
   - Est. complexity: M
   - Complexity score: Medium
   - Model class: medium
   - Customer value: HIGH (the only change that proves the goals on a real query)
   - Details: A shallow research query through the driver in a harness session and the same query through the daemon, both recorded with package paths, sidecar verdicts, and check results in `docs/research-agent-hardening-evidence.md`. Docs site, SKILL.md versions, CHANGELOG, SKILLS.md, skills index. Final local certification runs every suite from 003 to 010 plus `validate:strict`, `openspec validate`, `validate:codex` (defensive: no change edits the codex surface, so it must pass unchanged), `check:distribution`, `check-harness-adapters.js`, and `check:services-manifest`. This change is the phase's C-01 reconciliation change. When 010 is deferred, the evidence document's "Source scoring and graph" section is written as `deferred to <successor phase>` with G5 marked NOT MET, `tests/scoring-graph.sh` is omitted from certification, and the run still exports a package because 010's scripts are replaced, not required.

## EXECUTION ROUND ORDER

Round 1 (parallel): change-rah-001, change-rah-007
Round 2: change-rah-002
Round 3: change-rah-003
Round 4 (parallel): change-rah-004, change-rah-005
Round 5 (parallel): change-rah-006, change-rah-010, change-rah-008 (008 also needs 007's operator gate)
Round 6: change-rah-009
Round 7: change-rah-011

Rationale: 001 is a pure precondition and 007 is operator time, so both start immediately. Everything on the research side flows through the package contract (002) and then the driver (003), because 004, 005, and 006 all test against the driver's fixture runner. The learn side is gated by the operator review (007) and by the label vocabulary (005) that the artifact writer enforces. 009 waits for 008 so the learn skills that call the new RPCs already agree on paths. 011 is last by definition.

Parallel work must respect the single-build rule: 001, 004, and 009 each run Cargo and may not overlap with each other or with any other build on the machine. At plan time a `cargo test -p prometheus-substrate --features sovereign` process (pid 57647) had been running for more than eleven hours; until it ends, every cargo-gated task records BLOCKED and the executor moves to a non-Cargo task.

## COMMANDS TO RUN

All change structures exist. Execute in order with the KBD apply driver:

```
/kbd-apply change-rah-001-compile-baseline-and-timestamps
/kbd-apply change-rah-007-eval-ground-truth-review        # prepares the review sheet, then waits on the operator
/kbd-apply change-rah-002-research-package-contract
/kbd-apply change-rah-003-stage-contract-driver
/kbd-apply change-rah-004-daemon-job-execution
/kbd-apply change-rah-005-claim-labels-and-provenance
/kbd-apply change-rah-006-agent-duties-and-report-review
/kbd-apply change-rah-010-source-scoring-and-graph
/kbd-apply change-rah-008-learn-artifact-and-corpus-coherence   # only after index.json shows reviewed == 24
/kbd-apply change-rah-009-learner-model-write-paths-and-fsrs
/kbd-apply change-rah-011-integration-evidence-and-docs
```

## SCOPE CUTS AND TRADE-OFFS

- **A daemon-native Rust pipeline is not built.** Option A scored 15 of 25 against 21 for the harness-driven options; the daemon runs a headless harness instead (D-01). If neither harness binary is on `PATH`, daemon jobs are `blocked` by design and only the foreground driver works.
- **No migration of `~/.research-jobs/`.** Its 28 directories are daemon checkpoints from jobs that never ran; they are left in place and named in status output (D-14).
- **G5 may slip.** It is the only goal with no broken behaviour behind it. If 001 to 009 consume the phase, 010 moves to a successor and the phase still certifies against G1 to G4 with G5 recorded NOT MET; the phase does not certify with 010 half-done. In that branch 011's dependency on 010 is dropped by the deferral decision recorded in `decision-log.md`, and 011's evidence document names the successor phase.
- **The eval ground truth gets one reviewer.** Independent second review is deferred to a later phase; the review is still required before 008 (D-15).
- **FSRS optimizer is deferred.** Only the scheduler is adopted or ported; parameter training is a later phase (D-10).
- **Checkpoint-mode harness adapters are deferred.** The driver emits a `next_stage` JSON line; no harness adapter consumes it yet.
- **Feasibility against the assessment.** The assessment found the research surface non-functional and the learn surface internally inconsistent; this plan repairs contracts before adding capability and adds nothing outside the five goals. The 47 tasks are sized for one agent session each with two exceptions, the driver (003) and the daemon executor (004), which are marked L and may each need two sessions.

## ADVERSARIAL REVIEW

Judge k3 over the liter-llm gateway, cross-model check verified-distinct, producer claude-fable-5-1. Verdict PASS: 0 CRITICAL, 5 WARNING, 1 SUGGESTION (`review/plan/findings.json`, anti-theater gate PASS). Sycophancy self-check on the draft: score 0.0, no patterns.

| Finding | Disposition |
|---|---|
| W1 011 depends on 010 while 010 may be deferred | 011's dependency made conditional; the deferral branch names what 011 skips and records G5 NOT MET |
| W2 G3 plan ledger not in any Details | 003's Details now state the ledger creation and per-stage updates (already spec task 2) and the full-run assertion |
| W3 headless hook policy unresolved | 004's Details state the policy: daemon fires nothing, the child driver fires deep-research hooks, KBD hooks disabled; test asserts the markers; spec regenerated to match |
| W4 FSRS replacement outside the five goals | marked optional within 009, tied to assessment finding 26, phase certifies without it; spec regenerated to match |
| W5 hooks/hooks.json possibly touched | stated that only deep-research's four hook scripts are touched; `validate:codex` added to 011's certification defensively |
| S1 baseline artifact unnamed | 001 records both cargo outputs at `changes/change-rah-001-.../evidence/compile-baseline.txt`; spec regenerated to match |

## OPERATOR GATES

1. **Ground-truth review (change-rah-007 task 2).** The operator marks all 24 items reviewed before 008 starts.
2. **Cargo idleness.** Cargo-gated tasks in 001, 004, and 009 run only when `pgrep -x cargo` is empty; the executor never starts a competing build.
3. **Runtime position.** A concurrent headless session executing `control-plane-to-companion` re-took the KBD position repeatedly during assess, analyze, spec, and plan. The operator chose to wait rather than contest it. Until this phase is reactivated with `prometheus kbd phase activate`, `/kbd-apply` for any `change-rah-*` will be refused by the canonical-state gate. The reactivation command is in the spec-stage report and the phase memory.

## PLAN COMPLETE
