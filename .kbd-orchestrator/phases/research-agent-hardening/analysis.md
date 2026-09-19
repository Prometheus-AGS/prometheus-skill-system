# Analysis — research-agent-hardening

> Phase: `research-agent-hardening` · Stage: analyze · Date: 2026-09-03
> Inputs: `assessment.md` (all five goals NOT MET), `goals.md`, "Two Feynmans" gap analysis, Feynman CLI 0.3.47 source at `/Users/gqadonis/Projects/references/feynman` (MIT).
> Mode: stack specified. The stack is fixed by the pack: Rust crates under `substrate/`, bash and python3 scripts under `skills/` and `shared/scripts/`, markdown skills and agents executed by the harness, liter-llm as the OpenAI-compatible gateway. No stack discovery was run.
> Research budget: 5 Tier 1 queries, 2 Tier 2, 5 Tier 3, 0 Tier 4; roughly 15 minutes. No cap was reached.

## 1. Landscape

The assessment established that the research surface has no execution path and that the learn surface is internally inconsistent. Neither problem is a missing library. The work is overwhelmingly contract repair and pattern porting, and the analysis reflects that: most candidates are patterns or incumbent components, and `build_required` is long.

Three external facts shape every decision below.

- **Feynman CLI is MIT and its research workflow lives in plain markdown.** The 201-line `prompts/deepresearch.md`, the four agent prompts, and `SYSTEM.md` can be adapted into the pack's stage skills and agent definitions with attribution and no runtime dependency. Its execution model is "the harness runs the prompt", which is exactly how the pack's stages run today. It has no daemon.
- **The pack already has a working gateway path for script-side model calls.** `kbd_complete` in `shared/scripts/lib/kbd-model-resolve.sh` speaks OpenAI REST to the resolved gateway, builds bodies in python3, and fails loudly on empty replies. `adversarial-review` dispatches its judge through it. Any stage that needs a model call from a script has an incumbent.
- **Headless harness execution is already in use on this machine.** During this analyze a `claude -p --input-format stream-json` process was executing the companion phase, and CLAUDE.md records `codex exec --dangerously-bypass-hook-trust` as a vetted automation path. A daemon that needs to run a skill in the background has a proven way to do it that reuses the skill files verbatim.

Rust-native "deep research" projects surfaced by Tier 1 (`LightInn/deepsearch`, `programming-pupil/aos`, `usemanusai/free-deep-research`) are small, single-author, and Ollama- or web-specific. None offers the pack's package format, memory integration, or KB grounding, and none is a candidate for adoption. `tinyhumansai/openhuman` and `adolfousier/opencrabs` are general agents, not research pipelines. Landscape only.

## 2. Candidate evaluation

### 2.1 Execution model for G1 (open question 1) — verdict: adopt harness-driven stages with a contract-enforcing driver; adapt headless harness spawn for the daemon; reject a daemon-native pipeline this phase

Three options were scored on five criteria (fit to goals, reuse of existing skills, build cost, background execution, resumability), 5 points each.

| Option | Description | Fit | Reuse | Cost | Background | Resume | Total |
|---|---|---|---|---|---|---|---|
| A. Daemon-native | `prometheus-research` calls the gateway per stage from Rust, with search, scrape, memory, and KB adapters reimplemented in Rust | 4 | 1 | 1 | 5 | 4 | 15 |
| B. Harness-driven with a contract driver | The harness executes stage skills as today; `run-research.sh` becomes a stage-contract enforcer that verifies each stage's required artifacts, fires hooks, writes the checkpoint, honours `--resume`, and stamps blocked provenance when a stage fails | 5 | 5 | 5 | 1 | 5 | 21 |
| C. Daemon spawns a headless harness | `--daemon-job` is implemented as spawning `claude -p` or `codex exec` with the deep-research prompt and the job's output root; the daemon tracks the child and streams its checkpoint over SSE | 4 | 5 | 3 | 5 | 4 | 21 |

B and C tie, but they are not alternatives. B fixes the foreground path that every harness user hits and is the smallest change that makes G1 true. C gives the daemon a real job executor without a rewrite and reuses B's driver inside the child. A is rejected for this phase: it duplicates every integration the stage skills already declare (Tavily, Firecrawl, surreal-memory, sycophancy-correction, learn-grade) in a second language, and the daemon lacks even a date library today.

Decision: B first, C second, A rejected. The daemon's `--daemon-job` becomes a real clap argument whose handler spawns the headless harness with the job's query, depth, and output root, and the child runs the same driver as a foreground session. Which harness binary to spawn is resolved at runtime from `PATH` with `claude` preferred and `codex` as fallback; absence of both is a `blocked` job, not a crash. Hook-trust handling for `codex exec` is a spec-stage detail.

Risk: a headless child inherits the operator's harness configuration, including hooks. The spec must state which hooks a research child may fire, and the driver must never fire KBD lifecycle hooks from inside a research job.

### 2.2 Contract of record (open question 2) — verdict: author OpenSpec capabilities at spec stage; promote `research-package-spec.md` to the single normative reference; add a JSON Schema and validate it in both shell and Rust

The pack has 33 OpenSpec capabilities and none covers research or learn. Every prior phase since `phase-codex-plugin-distribution-and-ci` has expressed its contract as OpenSpec, and the certification gate runs strict OpenSpec verification. Writing the contract anywhere else would exempt this phase from the gate. Two capabilities are recommended for spec: `research-pipeline-execution` (driver contract, package layout, provenance, labels, resume) and `learn-model-coherence` (artifact path, corpus schema, learner-model RPCs).

For the package itself, `references/research-package-spec.md` becomes normative. No generator is introduced: the manifest template is deleted (the schema is the template), the SKILL.md example is a hand-maintained literal copy, and `export-package.sh` is hand-edited to emit the schema's fields. Drift is prevented by a check script, `scripts/check-research-package.sh`, that validates the SKILL.md example and a fresh `export-package.sh` output against the schema and fails on any mismatch; the plan wires it into the phase's local certification gate. This keeps constraint C-01 out of scope, since C-01 applies only to generated artifacts, and avoids adding a generator whose idempotency would need its own C-04 proof. A machine schema, `references/schemas/research-manifest.schema.json`, is the enforcement point. Validation runs in two places: the daemon through the `jsonschema` crate 0.53 (adopt, cand-006), and the shell driver through python3 `jsonschema` when importable (4.26 is present on this machine) with a jq required-keys fallback when it is not (cand-005). `ajv-cli` was rejected: it would add a Node runtime requirement to bash hooks that already require python3.

### 2.3 Output root (open question 3) — verdict: `~/.prometheus/research/<job_id>/`

Every other pack surface lives under `~/.prometheus/` (`learn`, `kbd`, `research` in the skill and scripts). The daemon and all four hooks are the outliers at `~/.research-jobs/`. Moving two Rust constants and four hook defaults is cheaper than moving the documented user-facing path, and `RESEARCH_OUTPUT_DIR` stays as the override. The `<job_id>.research/` suffix in the package spec is dropped; the directory is the package.

State of the old root, checked 2026-09-03: `~/.research-jobs/` exists and holds 28 `job-<epoch>-<uuid8>` directories created by the daemon's checkpoint writer; `~/.prometheus/research/` does not exist. Because no daemon job ever executed a stage, those directories hold checkpoint state only and no research output (inferred from the assessment's spawn finding, not from listing every directory). Decision: no migration. The new root starts empty, the old directories are left in place and never read, and the plan includes a one-line note in the daemon's status output naming the legacy path so an operator can delete it by hand.

### 2.4 Job identity and slugs (G3) — verdict: adapt Feynman CLI's slug rule; keep an opaque job id alongside

Feynman CLI derives a five-word lowercase slug per run and bans generic file names so concurrent runs cannot collide. The pack already has `subject_to_slug()` in `content-grounding-kb.sh`. Decision: move that function to `shared/scripts/lib/slug.sh`, derive `<slug>` from the query at job start, and name the package directory `<slug>-<yyyymmdd>-<4hex>` (the same shape `learn-goal` already uses for goal ids). The daemon's `job-<epoch>-<uuid8>` stays as the internal job id and is recorded in the manifest.

### 2.5 Verification vocabulary, provenance sidecar, plan ledger (G3) — verdict: adapt from Feynman CLI; no library

Ported as patterns, with attribution in the reference file:

- Claim labels `verified | unverified | blocked | inferred` on every claim in `graph.json` and every row of the report's evidence table. Package label stays `verified | partial | unverified` for OKF continuity and is derived from the claim labels.
- `<slug>.provenance.md` beside `report.md` with date, rounds, sources consulted, accepted, rejected, verification verdict `PASS | PASS WITH NOTES | BLOCKED`, plan path, and stage files. Written by the driver on every exit path, including failure.
- `plan.md` gains three maintained sections, task ledger, verification log, and decision log, updated by the driver at each stage boundary. `report-synthesizer.md` is corrected to read `plan.md`.
- `--resume` reads `checkpoint.json` and skips stages whose required artifacts exist and validate.
- The feynman-loop artifact gains `verification: {label, evidence}` on each transfer score and a `provenance` block naming the grade file and corpus, so the learn side speaks the same vocabulary.

### 2.6 Agent duties (G4) — verdict: adopt harness-native `tools:` frontmatter; build the scale gate and ordering into the driver; adopt adversarial-review with a new artifact target

Claude Code agent definitions accept a `tools:` allowlist in frontmatter, which is the same convention Feynman CLI uses on Pi. The pack's four research agents carry `metadata.model_tier` and `metadata.stage` only. Decision: add `tools:` to each, with the report synthesizer given no search or fetch tools, the verifier given fetch tools, and the planner given read-only tools. On harnesses that ignore the key, the allowlist is advisory and the agent prompt restates it; that degradation is documented, not hidden.

Verifier-before-reviewer ordering and the scale gate are driver logic, not agent logic: the driver refuses to start stage 06 until stage 05's artifact validates, and a `--scale direct` mode (chosen when the planner emits fewer than three sub-questions) runs stages 01, 02, 03, 05, 09, 10 with the lead session and no subagents.

For the final report, `adversarial-review` is adopted as is, with one build item: `build-review-packet.sh --mode artifact` gains a `research` target that packs `report.md`, `<slug>.provenance.md`, and `plan.md`. The producer model is the session model, as for every other artifact review.

### 2.7 Learn coherence (G2) — verdict: nest artifacts per concept; extend the grounding script; add three RPCs; adopt rs-fsrs for scheduling

- Artifact path: `goals/<goal-id>/artifacts/<concept-id>/<artifact-id>.json`. Two of the three readers already key by concept (learn-certify reads the concept directory, learn-retain globs by concept prefix). feynman-loop's writer changes one path segment; learn-retain's glob becomes `artifacts/<concept-id>/*.json`.
- Corpus schema: `content-grounding-kb.sh` is extended, not replaced, to emit `key_points[]` and `misconceptions[]` per source, deriving `misconceptions[]` from entries flagged `is_misconception` and `key_points[]` from `content_summary` sentences. The eval harness's workaround is then removed. The triplicated copies become one file under `shared/scripts/` with the two skill copies replaced by thin wrappers, which is the pattern the pack uses for `ui-surface`.
- learner-model: `add_gap`, `add_session`, and `set_certified` RPCs, plus `certified_at: Option<String>` on `ConceptState`. `GapRecord` and `SessionRecord` are already typed; only the dispatch arms and the store fold are missing.
- FSRS: the stub scheduler never reads `difficulty`. Tier 2 confirms `fsrs-rs` 6.6.2 exposes `FSRS::next_states(Option<MemoryState>, desired_retention, days_elapsed)` returning stability, difficulty, and interval per rating, which maps directly onto `FSRSCard`. `fsrs-rs` also ships the optimizer, which pulls a machine-learning backend the learner-model binary does not need under the pack's build-speed policy; `rs-fsrs` 1.2.1 from the same organisation is scheduler-only. Decision: adopt `rs-fsrs` for `next_review()`, reference `fsrs-rs` for a later optimizer phase. The dependency weight of both must be confirmed with `cargo tree` at spec stage; this verdict is inferred from crate descriptions, not from a build.

### 2.8 Scoring and graph (G5) — verdict: adapt Feynman CLI's algorithms as python3 scripts; no library

Available-signal weight renormalisation and rank sensitivity are a few dozen lines each and need no numeric library. They replace the body of `verify-sources.sh` with a `score-sources.py` that reads the registry, scores the five documented rubric dimensions where evidence is present, drops absent signals from the denominator, records `appliedWeights` per source, and writes a `sensitivity.json` classifying each rank `stable | sensitive | volatile` under four alternate weight vectors. Content-addressed claim ids (`claim:sha256(scope:normalised_text)[:16]`) and `contradicts` edges go into `build-graph.sh`, which then emits the spec's `{topics, claims, relations}` shape. Semantic contradiction detection, which the assessment found absent, routes through `kbd_complete` with the critic role, since it is a model call and the incumbent gateway helper exists for exactly that.

G5 is ordered last and does not gate certification of G1 to G4.

### 2.9 Daemon hygiene (G1) — verdict: adopt `chrono` 0.4

The daemon has no date library, which is why `chrono_now()` fabricates timestamps. `learner-model` already pins `chrono = "0.4"` with serde. Adopt the same pin in `prometheus-research` and delete both hand-rolled formatters.

## 3. Build-vs-adopt summary

| Gap | Verdict | Candidate |
|---|---|---|
| Stage execution model | adopt pattern | harness-driven stages + contract driver (cand-003, cand-004) |
| Daemon job execution | adapt | headless harness spawn (cand-004) |
| Daemon-native pipeline | reject | async-openai rewrite (cand-007) |
| Contract of record | adopt | OpenSpec capabilities at spec; package spec normative |
| Manifest validation | adopt | jsonschema crate (cand-006), python jsonschema with jq fallback (cand-005); ajv-cli rejected (cand-009) |
| Output root | decide | `~/.prometheus/research/` |
| Slug and job id | adapt | Feynman slug rule + existing `subject_to_slug` (cand-003) |
| Labels, sidecar, ledger, resume | adapt | Feynman CLI patterns (cand-003) |
| Agent allowlists | adopt | harness `tools:` frontmatter (cand-003) |
| Report review | adopt + build | adversarial-review with new `research` target (cand-008) |
| Learn artifact path, corpus schema, RPCs | build | no candidate; contract repair |
| FSRS scheduling | adopt | rs-fsrs 1.2.1 (cand-001); fsrs-rs reference (cand-002) |
| Scoring, sensitivity, claim ids, contradicts | adapt | Feynman CLI algorithms as python3 (cand-003) |
| Script-side model calls | adopt | `kbd_complete` (cand-010) |
| Timestamps | adopt | chrono 0.4 (cand-011) |

## 4. Open questions carried to spec and plan

1. **Headless harness policy.** Which hooks may a research child fire, how is `codex exec` hook trust handled, and what does the daemon record when neither harness binary is on `PATH`.
2. **Dependency weight of rs-fsrs and fsrs-rs.** Confirm with `cargo tree` before the learner-model change is authored; if rs-fsrs is also heavy, the fallback is porting the FSRS-6 formulas into the existing stub.
3. **Compile health precondition still open, and every Rust adopt verdict is conditional on it.** `cargo check` on `prometheus-research` and `learner-model` was blocked at both assess attempts and at two analyze attempts by a `cargo test -p prometheus-substrate --features sovereign` process (pid 57647) that had been running for more than eleven hours when analyze closed. The adopt verdicts for `jsonschema` 0.53, `rs-fsrs` 1.2.1, and `chrono` 0.4 add dependencies to those two crates and are therefore conditional: if either crate does not compile today, the first change in the plan is a compile repair, and the dependency changes wait behind it. The option scoring in section 2.1 does not depend on compile state, since option B touches no Rust. Plan must schedule the two checks as the first task of the phase, before any G1 or G2 change is authored.
4. **Eval ground-truth review comes before G2, then a re-baseline after.** The assessment asked for the human review of the 24 draft ground-truth items before the corpus schema changes, and that ordering stands: a re-baseline against unreviewed truth would measure the grader against labels nobody has checked. Plan sequence is review the ground truth, land the G2 corpus change, then re-run the eval and record the new baseline.
5. **Runtime position contention.** A concurrent headless session is executing `control-plane-to-companion`. Reactivating this phase hijacks that session's position. The operator must choose: pause the companion executor, or run this phase's remaining stages when it is idle. Analyze artifacts were written without reactivating the phase for that reason.

## Adversarial review

Judge k3 over the liter-llm gateway, cross-model check verified-distinct, producer claude-fable-5-1. Verdict PASS: 0 CRITICAL, 3 WARNING, 2 SUGGESTION. Findings at `review/analyze/findings.json`; anti-theater gate PASS.

- W1 (eval review ordering reversed): resolved; open question 4 now schedules the ground-truth review before G2.
- W2 ("regenerated" with no generator): resolved; section 2.2 now states no generator, hand-synced files, and a drift-check script added to `build_required`.
- W3 (Rust adopt verdicts on an unverified compile baseline): resolved by stating the verdicts as conditional in open question 3; the check itself is still blocked by the same external build.
- S1 (cand-006 maintenance evidence): resolved; repo activity added to `library-candidates.json`.
- S2 (old output root state): resolved; section 2.3 records 28 legacy job directories and the no-migration decision.
