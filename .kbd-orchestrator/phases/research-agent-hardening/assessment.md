ASSESSMENT: research-agent-hardening
Project: prometheus-skill-pack (Prometheus skill system 1.8.0)
Date: 2026-09-03
Codebase baseline: main at cfbc262 plus branch feat/cpc-001-002-integration-contract; the deep-research skill, prometheus-research daemon, and learn domain are all present on disk but the research pipeline has no working execution path and the learn skills disagree with each other on paths and schemas.
Cross-tool progress: none (phase created this session at runtime revision 613; progress.json lists zero changes)

Evidence labels used below: **verified** = read or grepped in this session; **survey** = subagent inventory not independently re-read. Source analysis: "Two Feynmans" (https://claude.ai/code/artifact/874b2865-dae8-45c0-8e68-3734d3b520e0), comparing against Feynman CLI 0.3.47 at /Users/gqadonis/Projects/references/feynman.

IMPLEMENTATION STATUS

Research execution (G1)
- prometheus-research daemon job execution: [STUB] — `substrate/prometheus-research/src/job/spawn.rs:48` passes `--daemon-job`; the clap `Cli` in `src/main.rs` defines `--mode`, `--port`, and one other long flag, none named `daemon-job`. The child exits on parse; the parent still writes `status: running`. Grep confirms the string appears exactly once in the crate. **verified**
- Checkpoint timestamps: [STUB] — `src/job/checkpoint.rs:60` formats every timestamp as `1970-01-01T00:00:{secs % 60}Z`. `src/job/spawn.rs` hardcodes `2026-07-08T00:00:00Z`. **verified** (spawn.rs literal: survey)
- Shell driver: [STUB] — `skills/research/deep-research/scripts/run-research.sh` lines 51 to 60 loop the stage list, emit `started` then `completed` per stage with a comment where execution should be, and report `complete`. No stage runs, no hook fires, no artifact is written. `--check-tools`, which SKILL.md tells users to run, is not handled. **verified**
- MCP export: [STUB] — `src/mcp_server/mod.rs:147` returns `"Export generation will be implemented in change-prb-007"`. **verified**
- Output root: [PARTIAL] — hooks (all four) and the Rust crate (`config.rs:31`, `checkpoint.rs:27`, `checkpoint.rs:88`) use `~/.research-jobs/`; `run-research.sh:32` and `export-package.sh:8` use `~/.prometheus/research/`. **verified**
- Manifest schema: [PARTIAL] — `references/research-package-spec.md:34` has `stages_completed: 10` (int); `templates/research-package-manifest.json:8` has `stages_completed: []` (array); `export-package.sh` emits neither field and hardcodes `feynman_grade: null`, `contradictions_resolved: 0`, `graph_nodes: 0`. Three shapes are verified. goals.md carried a survey count of four; the fourth was attributed to a SKILL.md stage-10 example, but SKILL.md contains no manifest.json example (only `report.md` frontmatter), so the verified count is three and the survey count was wrong. **verified** (export-script hardcodes: survey)
- HTTP server, SSE stream, AG-UI events, A2UI registry, MCP tool registration: [DONE] — real code with tests in `tests/job_lifecycle.rs`, `tests/mcp_tools.rs`, `tests/sse_stream.rs`. **survey**
- Stage scripts: [PARTIAL] — `verify-sources.sh` scores by domain string only (1 of 5 documented rubric dimensions); `detect-contradictions.sh` is numeric-regex only; `build-graph.sh` emits `cites` edges only and a `{nodes, edges}` shape that does not match the spec's `{topics, claims, relations}`. **survey**
- Resumability: [MISSING] — `hooks/post-stage.sh` writes `checkpoint.json` with `last_completed_stage`; grep finds no reader and no `--resume` anywhere under `skills/research/deep-research`. **verified**

Learn coherence (G2)
- Feynman artifact path: [PARTIAL] — feynman-loop writes `goals/<goal-id>/artifacts/<artifact-id>.json`; learn-retain globs `artifacts/<concept-id>-*.json` (SKILL.md:47); learn-certify reads `artifacts/<concept-id>/` (SKILL.md:145). Three conventions, mutually unreadable. **verified**
- Corpus schema: [PARTIAL] — learn-grade Step 1 expects `sources[].key_points[]` and `sources[].misconceptions[]`; `shared/scripts/content-grounding-kb.sh:61` emits `source_ref, source_type, confidence, is_misconception, content_summary` only. learn-grade Step 7 (transfer problems from `key_points`) has no input. **verified**
- learner-model write paths: [MISSING] — `src/main.rs` dispatches exactly `load`, `seed_from_survey`, `get_concept`, `add_observation`, `review`. `store.rs` touches `gaps` and `sessions` only at construction (lines 366 to 367, 421 to 422). No `add_gap`, no `add_session`, no `certified_at` field. **verified**
- FSRS difficulty: [STUB] — `src/fsrs.rs` references `difficulty` once, at initialisation (line 94); `next_review()` never reads or updates it. **verified**
- learn-grade eval dataset: [DONE] — 24 items, regression script, baseline snapshot; ground truth is 24/24 draft, 0 reviewed, and the harness documents that learn-grade is prose-executed. **survey**
- Loop control in feynman-loop (depth cap, recursion floor, escalation, three-criterion closure): [STUB] — `scripts/write-artifact.sh` is a JSON writer; all control logic is prose. **survey**
- Grounding script duplication: [PARTIAL] — `content-grounding-kb.sh` exists byte-identical in `shared/scripts/`, `learn-goal/scripts/`, `learn-kb/scripts/`. **survey**

Provenance and vocabulary (G3)
- Verification labels: [PARTIAL] — pack uses `verified | partial | unverified` at package level only; grep of `references/*.md` for `inferred` or `blocked` returns nothing. No claim-level label exists. **verified**
- Provenance sidecar: [MISSING] — no `<slug>.provenance.md` or equivalent per-deliverable sidecar; manifest.json is the intended carrier and has three conflicting shapes. **verified**
- Plan as ledger: [MISSING] — `stage-01` writes `<job_id>/plan.md` once; nothing updates it with task, verification, or decision state. `agents/report-synthesizer.md` expects `plan.json`, which stage-01 never writes. **verified** (stage-01 path; report-synthesizer mismatch: survey)
- Slug naming: [MISSING] in research — job IDs are `research-<YYYYmmdd-HHMMSS>` (bash) and `job-<epoch>-<uuid8>` (Rust); the `.research` example in SKILL.md shows a topic slug no code produces. **survey**

Agent duties (G4)
- Tool allowlists: [MISSING] — `agents/source-verifier.md` frontmatter carries `model_tier` and `stage` only; no `tools:` key on any of the four research agents. **verified**
- Verifier-before-reviewer ordering: [MISSING] — no ordering constraint in SKILL.md or stage skills. **survey**
- Scale gate: [MISSING] — `depth` changes per-question result counts (10/25/50) and which stages run; nothing forbids multi-stage execution for a narrow question. **verified** (SKILL.md depth table)
- adversarial-review on the report: [MISSING] — grep for `adversarial-review` under `skills/research` and `skills/learn` returns no files. The only gates are the stage-05 sycophancy check and the stage-09 learn-grade gate. **verified**

Scoring and graph (G5)
- Weight renormalization, rank sensitivity: [MISSING] — `verify-sources.sh` produces a 0 to 100 score from domain lists and does not sort. **survey**
- Content-addressed claim IDs: [MISSING] — `build-graph.sh` uses `node-<md5(name)[:8]>` for nodes; claims are not deduplicated across artifacts. **survey**
- `contradicts` edges: [MISSING] — `build-graph.sh` emits `cites` only. **survey**

Documentation drift (cross-cutting; serves G1 under constraint C-03, docs updated with surface changes)
- `skill.toml:31` still says the MCP server binary is deferred; the binary exists. **verified** — G1 (C-03)
- `SKILL.md:199` links `references/a2ui-components.md`; the file does not exist. **verified** — G1 (C-03)
- `SKILL.md:107` says the daemon is v1.6.0; `Cargo.toml` and the `/health` example both say 0.1.0. **verified** (SKILL.md text; Cargo.toml: survey) — G1 (C-03)
- SKILL.md lists A2UI components `confidence-meter` and `export-card`; the registry has `media_card` and `markdown_viewer` instead. **survey** — G1 (C-03)

Findings outside G1 to G5 (deferred, out of phase scope unless analyze adds a goal)
- feynman-loop loop control (depth cap, recursion floor, escalation, three-criterion closure) is prose-only. Not required by G2, which covers paths, schema, and write paths. **survey** — deferred
- `content-grounding-kb.sh` byte-identical triplication. Consolidation is hygiene, not a goal. **survey** — deferred
- learn-grade eval ground truth unreviewed. Raised as open question 5, not a goal. **survey** — deferred

CROSS-TOOL PROGRESS
- NONE — no cross-tool activity recorded. At creation (revision 613) progress.json showed 0 of 0 changes. The phase's `completion.evidence`, `completion.certification`, and `completion.publication` blocks carry inherited runtime-level summaries from a prior run (commit a9f7d62); they do not describe this phase.
- RUNTIME PROJECTION DEFECT (observed during this assess, revision 623): after `prometheus kbd stage enter --phase research-agent-hardening --id assessment`, the runtime (a) projected the companion phase's `change-cpc-008-sync-skills-plugin` into this phase's `progress.json` `changes[]` as IN_PROGRESS, (b) raised the project-wide implementation total from 22 to 23, (c) rewrote `current-waypoint.json` with `phase: control-plane-to-companion` while `activePhaseId` still says `research-agent-hardening`, and (d) set `exactNextCommand` to `/kbd-apply change-cpc-008-sync-skills-plugin`. `prometheus kbd status --json` shows both phases `in_progress` and `activePath` unset. No runtime-owned file was hand-edited. This must be reconciled through the CLI (`prometheus kbd phase activate`) before analyze, and the projection bug itself belongs to the KBD runtime, not this phase. **verified**

SPEC GAP SUMMARY
- No OpenSpec spec covers deep-research, prometheus-research, or the learn domain. `openspec/specs/` has 33 capabilities; grep for `deep-research`, `prometheus-research`, `learn-grade`, or `feynman` returns none. The pack-local references (`research-package-spec.md`, `okf-research-format.md`, `feynman-quality-gate.md`) are the only written contracts and they disagree with each other and with the scripts. Analyze must decide whether this phase authors OpenSpec capabilities for these surfaces or keeps the pack-local references as the contract.
- Package spec says `<job_id>.research/`; every script and the daemon use a bare `<job_id>/` directory. **verified**
- Package spec lists `plan.json`; stage-01 writes `plan.md`. **verified**
- The `--ingest-palace` flag documented in the package spec is not accepted by `export-package.sh`. **survey**
- No prior `reflection.md` exists for `control-plane-to-companion` or `kbd-control-plane-recovery`; there is no predecessor recommendation to reconcile against.

BUILD HEALTH
- build check: UNKNOWN — a `cargo`/`rustc` process from another workspace was active at assessment time; the single-build rule in CLAUDE.md forbids starting a competing check. `cargo check -p prometheus-research` was not run. Re-run it at analyze or plan when the machine is idle.
- known violations: compile health of `prometheus-research` and `learner-model` is UNKNOWN. A second attempt at assess close found cargo pid 57647 still active in another workspace, so neither crate was checked. Existence of test files is not evidence of compilation. The spawn flag bug is a runtime defect no compiler will catch regardless. The learner-model crate's store tests use a `json-stub` CRDT engine that does no merging, so CRDT convergence is asserted only against a fake. **survey**
- BLOCKING PRECONDITION for analyze: run `cargo check -p prometheus-research` (from `substrate/prometheus-research`) and `cargo check -p learner-model` (from `substrate/learner-model`) when no other cargo process is running, and record the result in the analyze handoff before any G1 or G2 change is authored.
- test coverage: PARTIAL — prometheus-research has three integration test files covering lifecycle, MCP tools, and SSE; none exercises job execution end to end, which is why the spawn bug survives. learner-model has inline unit tests only (allowed as legacy, not acceptance evidence). deep-research shell scripts have no tests. learn skills have the learn-grade eval dataset and nothing else.

CONSTRAINT CHECK
- AGENTS.md violations: NONE found. AGENTS.md has no "Never Do" section; the operative rules live in CLAUDE.md. No violation of the implementation-first policy, the single-build rule, or the local-only validation rule was observed in the surfaces inspected.
- constraints.md violations: NONE currently, with two forward risks. C-01: if any change in this phase touches `shared/launchagents/com.prometheus.research.plist`, `shared/services.manifest.json` must be regenerated in the same change. C-05: every deep-research hook runs under `bash` and must stay bash 3.2 compatible; none currently uses `mapfile` or `declare -A` (spot check, not exhaustive).
- Policy risk for G1: making `run-research.sh` execute stages means the driver must invoke LLM-backed stage skills from a shell script. The harness cannot call skills from bash. Analyze must choose an execution model (daemon-driven via liter-llm, harness-driven via a skill loop, or hybrid) before any change is written.

GOAL PROGRESS
- G1 Execution: NOT MET — the daemon cannot run a job, the driver runs nothing, timestamps are wrong, two output roots, three manifest shapes.
- G2 Learn coherence: NOT MET — three artifact path conventions, corpus schema mismatch, no gap or session write path, no `certified_at`.
- G3 Provenance: NOT MET — three package-level labels, no sidecar, no ledger, no resume, no slugs.
- G4 Duties: NOT MET — no tool allowlists, no ordering, no scale gate, adversarial-review not wired.
- G5 Scoring: NOT MET — domain-only scoring, no sensitivity artifact, no content-addressed claims, no contradicts edges.

OPEN QUESTIONS FOR ANALYZE
1. Execution model for G1: who runs the stages when `run-research.sh` is invoked, given that stage skills are LLM-executed and bash cannot call them.
2. Contract of record: author OpenSpec capabilities for deep-research and learn, or promote `research-package-spec.md` to the single source and delete the template and SKILL variants.
3. Output root: `~/.prometheus/research/` (matches the rest of the pack's `~/.prometheus/` convention) or `~/.research-jobs/` (what the daemon and hooks already use).
4. Sequencing of G5 within this phase. G5 is in scope and NOT MET; it is the only goal whose gaps are missing capability rather than broken existing behaviour, so it should be ordered last, after G1 through G4, and analyze should confirm it does not block phase certification if G1 to G4 land first.
5. Whether the learn-grade eval ground truth should be human-reviewed before G2 changes the corpus schema, since the F1 0.96 baseline will need re-baselining.

ADVERSARIAL REVIEW
- Judge k3 via rest-gateway, cross_model_check verified-distinct, producer claude-fable-5-1. Verdict PASS: 0 CRITICAL, 3 WARNING, 1 SUGGESTION. Findings at `review/assess/findings.json`; anti-theater gate PASS (score 0.0).
- W1 (manifest count four vs three): resolved above; verified count is three, survey count was wrong.
- W2 (G5 descoping): resolved; open question 4 reframed as sequencing only.
- W3 (compile claim unsupported): resolved by downgrading to UNKNOWN and adding the blocking precondition above. The cargo check itself remains outstanding because another build held the machine at both attempts.
- S1 (orphan findings): resolved by tagging documentation drift to G1 under C-03 and listing out-of-scope findings as deferred.
- Sycophancy self-check: score 0.02, one low S-07 length note, no correction. Receipt at `sycophancy/assess-2026-09-03T23-47-29Z.json`.

ASSESSMENT COMPLETE
