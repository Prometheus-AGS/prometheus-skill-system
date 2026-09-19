# Assessment — deep-research-onyx-parity

**Phase:** `deep-research-onyx-parity`
**Date:** 2026-09-08
**Repository:** `prometheus-skill-pack`, branch `feat/cpc-001-002-integration-contract`, base `cfbc262`, uncommitted
**Predecessor:** `research-agent-hardening`, closed 2026-09-08, 5/5 goals MET, **11/11 changes archived**
(`ls .kbd-orchestrator/changes/archive/ | grep -c rah` → 11; no active rah change remains).
`goals.md` originally read "10 archived" because it was written before rah-007
was descoped and archived later the same day; both documents now report 11/11.
**Source analysis:** `docs/deep-research/deep-research-skill-vs-onyx-report.md` (2026-09-07, revised 2026-09-08) — **input, not a settled plan** (decision D-22/D-23)

**Where "UAR" is, for every citation below.** Universal Agent Runtime is an
**external sibling checkout**, not part of this repository and not a submodule of
it: `/Users/gqadonis/Projects/prometheus/universal-agent-runtime`. Every
`universal-agent-runtime/...` path with a line number in section 0 is relative to
that absolute root and was read there. Nothing under this repo resolves those
paths, and the pack has no dependency on UAR (see §2). Analyze and plan must read
them at that root or re-verify at whatever root the operator's machine uses.

**Do not confuse it with the in-repo lookalike.** This repository contains
`substrate/exec-tier-w/versions.toml`, which is a *different file* and carries no
liter-llm pin. V1's evidence is the UAR checkout's own `versions.toml`, read at
the external root above. V1–V3 are therefore **external-checkout evidence**:
reproducible on this machine, but analyze should re-verify at the UAR root rather
than assume an in-repo path resolves them.

---

## 0. The four open verifications (D-23) — resolved by command

The report could not complete these because local MCP servers hung. They are
resolved here before any goal is assessed, so nothing downstream inherits an
assumption.

| # | Question | Verdict | Evidence |
|---|---|---|---|
| V1 | liter-llm Rust API availability | **AVAILABLE** | `universal-agent-runtime/versions.toml:20` pins `liter_llm = "1.18.2"` at commit `c5c6caa`; `Cargo.toml:504` declares `liter-llm = { path = "vendor/git/liter-llm/crates/liter-llm", features = ["full"] }`; the submodule is checked out and its `Cargo.toml` exists. It appears in the workspace `exclude` list, which means only "not a workspace member" — a path dependency is still usable |
| V2 | UAR-REM-002 status for three correctness defects | **UNRESOLVED — the identifier does not exist** | `grep -rl "UAR-REM"` over the entire UAR tree returns nothing. No phase under `universal-agent-runtime/.kbd-orchestrator/phases/` matches. The report's D-1 gates the native runner on "verified UAR-REM correctness fixes" that cannot be located. **Treated as unverified; the native-runner path must not be planned against it** |
| V3 | Nature of the nested `crates/prometheus-skill-system` | **A GENUINE SUBMODULE** | `universal-agent-runtime/.gitmodules` declares it with `url = https://github.com/Prometheus-AGS/prometheus-skill-system.git`, `branch = main`; `git ls-files -s` shows mode `160000` pinned at `ad5c82c6`. Not a worktree, not a drifted copy |
| V4 | Onyx benchmark standing (RACE ~54, "reported as #1 at points in 2026") | **UNVERIFIED** | Not measured here and not measurable without `tests/bench/`, which does not exist. Recorded as an `unverified` claim per the phase's own label vocabulary. It must not enter the plan as established fact |

**One correction to the report's premise.** The report's revision proposes a
`NativeRunner` on "UAR kernel + liter-llm". UAR has exactly **one** crate —
`crates/prometheus-skill-system`, which is this pack's own submodule — and its
workspace members are `[".", "tools/uar-jwt-proxy", "tools/mcp-server-fetch"]`.
There is no separate kernel crate to build against. liter-llm is real and
usable; the "UAR kernel" half of D-1 is not established.

---

## 1. Goal-by-goal gap assessment

### G1 Thread execution — **NOT MET**

Stages 02–04 are strictly sequential in one context. Evidence:

- `run-research.sh` has exactly **one** stage loop; there is no fan-out primitive.
- `skill.toml:38` declares `threaded = true`, which the pipeline docs describe as
  future intent. The flag is aspirational and currently false in behaviour.
- No `research-director` or `research-worker` agent exists; `agents/` holds four
  files (planner, source-verifier, contradiction-resolver, report-synthesizer).
- No `threads/` artifact tree, no brief schema, no merge script.

Consequence on long jobs: one context carries every sub-question, every search
result, and every chunk. At `exhaustive` depth that is precisely the context
blow-up the two-level topology exists to avoid.

### G2 Lossless handoff — **NOT MET**

`agents/report-synthesizer.md` declares its inputs as `plan.md`, `graph.json`,
`citations.json`, `contradictions.json`. It **never reads prose evidence**. The
path is chunk → registry → graph claim → report, so the synthesizer sees only
compressed claims. There is no dossier artifact and claims carry no verbatim
quote span, so "read before you label" cannot be checked mechanically.

### G3 Multi-pass report — **NOT MET**

`stage-09-report/SKILL.md` step 5 writes `report.md` in one pass. There is no
outline artifact, no section files, no assembler, no coherence editor, and no
claim-set invariant. For a long report this risks output truncation, section
drift, and repetition.

### G4 Budgets and failure semantics — **NOT MET**

`grep -nE "timeout|budget|max_cycles|minutes"` over `run-research.sh` returns
**nothing**. The only failure control is a retry policy. There is no
force-complete path, so a stalled stage has no bounded exit. `checkpoint.json`
carries no `budgets` object.

### G5 Measurement — **NOT MET**

`tests/bench/` does not exist. The playbook's targets are internal and no
RACE/FACT run has ever been performed. "On par with Onyx" is currently
**unfalsifiable in both directions** — the pack cannot show it matches, and the
report cannot show Onyx's number either (V4).

---

## 2. What is already in place (do not rebuild)

The predecessor phase built the contract this work must preserve:

| Asset | Where | Why it matters here |
|---|---|---|
| Stage contracts + boundary validation + checkpoint + `--resume` | `run-research.sh`, `references/stage-contracts.md` | Threading changes *how* 02–04 are produced, not the contract. Validators must keep passing on merged output |
| Four-label claim vocabulary; derived `verification_status` | `okf-research-format.md`, `check-research-package.sh --derive` | The differentiator Onyx has no equivalent for. Verified-claim ratio is the metric to report in G5 |
| Provenance sidecar on every exit path | `write-provenance.sh` | Must gain per-thread rows without changing its exit-path guarantee |
| `tools:` allowlists on four agents | `agents/*.md` | The director's no-search rule uses the same mechanism; advisory on some harnesses |
| Headless harness spawn, per-job logs, token-gated event ingest, cancel, exit mapping | `job/spawn.rs`, `job/daemon.rs` (`Command::new`, `spawn_harness`) | Process-level workers are an extension of code that already works, not new infrastructure |
| Content-addressed claim ids | `build-graph.sh`, `detect-contradictions.sh` | The merge step's dedupe key already exists |
| Fixture-driven suites | `tests/driver-contract.sh` (128), `tests/scoring-graph.sh` (45) | The regression net for any refactor of 02–04 |

**Dependency direction is clean.** No `substrate/` manifest or `package.json`
references `universal-agent-runtime`. Integration-contract rule 1 holds today and
must survive this phase.

---

## 3. Carried-in defects (predecessor D-A, D-B)

Both were found by running the pipeline for real, both are install-surface, and
both directly affect this phase because it will exercise the daemon heavily.

| Id | Defect | Impact here |
|---|---|---|
| **D-A** | `com.prometheus.research.plist` sets no `PATH`; `install-binaries.sh:582` substitutes only `__HOME__` while sibling templates carry `__PROMETHEUS_PATH__` | The launchd daemon cannot resolve any harness binary, so **no process-level worker can ever spawn under the installed service**. This blocks G1's strategy B until fixed |
| **D-B** | The installed plugin generation ships a 1920-byte stub driver (repo: 30718) with zero `--resume`/`checkpoint`/`next_stage`/`RESEARCH_STAGE_RUNNER` | `resolve_driver()`'s `~/.claude` candidate resolves to a script with none of the stage contract. Interim override: `RESEARCH_DRIVER` |

Neither is a defect in daemon logic: in both cases it refused to report a
success it had not achieved.

### Constraint impact of fixing them (must reach the plan)

| Fix | Constraint triggered | Same-change requirement |
|---|---|---|
| D-A: add `__PROMETHEUS_PATH__` to the research plist and the substitution in `scripts/install-binaries.sh` | **C-01** — `shared/services.manifest.json` is generated from `shared/launchagents/*.plist` and `shared/systemd/*` | Regenerate the manifest (`npm run generate:services-manifest`) and pass `npm run check:services-manifest` in the same change |
| D-B: republish the plugin generation so the installed driver is current | **C-01** — plugin surfaces are generated | Pass `npm run check:distribution` and `npm run validate:codex` in the same change |
| Any bash fan-out in `run-research.sh` (open question 2) | **C-05** — the driver is reachable from a launchd-invoked path, and macOS `/bin/bash` is 3.2 | No `mapfile`, no `declare -A`; test under `/bin/bash`, not only bash 5 |
| Any Rust scheduler work | one Cargo build machine-wide | `pgrep -x cargo` before every cargo command; never dispatch a competing build |

---

## 4. Open questions for analyze

1. **Native runner viability.** With V2 unresolved and no UAR kernel crate, is
   the `NativeRunner` in scope at all this phase, or is `HarnessRunner` the only
   runner? Default: harness-only; native deferred with the reason recorded.
2. **Where does the scheduler live?** Bash driver fan-out, a new Rust subcommand
   (`prometheus-research threads run`), or in-session subagents? The report's
   revised D-3 proposes the Rust subcommand with the bash driver still owning the
   contract. Cost, testability, and the one-Cargo-build constraint all bear on this.
3. **Workspace split.** The report's revision puts `drt-000` (split
   `prometheus-research` into four crates) first. Is that justified before any
   threading behaviour exists, or is it speculative restructuring? The crate is
   currently a single `[lib]`+`[[bin]]` with no workspace.
4. **Token cost ceiling.** Three parallel workers × 8 cycles × long dossiers is
   3–6× today's linear 02–04. What is the acceptable ceiling, and is
   `--max-parallel 1` a required deliverable rather than a nicety?
5. **Advisory-harness enforcement.** On harnesses that ignore `tools:`, the
   director's no-search rule is unenforceable. Is the merge-script gate (a source
   in a dossier that is not in that thread's `sources.json` is CRITICAL) accepted
   as the real control?
6. **Do D-A and D-B belong in this phase?** They block G1 strategy B. They are
   install-surface, which no change in the predecessor phase owned. Fixing them
   here is defensible; deferring them means G1 cannot be demonstrated under the
   installed service.
7. **Bench scope.** DeepResearch Bench is 100 PhD-level tasks; the report
   proposes a 10-task English subset. Judge model, cost per run, and whether
   Onyx is run locally for a real head-to-head or cited from published numbers.
8. **Is director → worker a hard depth ceiling?** The report is emphatic that a
   worker must never spawn sub-workers, because each hop re-summarises and
   distorts; the director dispatches another thread instead. Proposed default:
   two levels, enforced — the merge step treats a source in a dossier that is not
   in that thread's own `sources.json` as CRITICAL, which also catches a worker
   that sub-dispatched. Analyze should confirm the ceiling and where it is
   enforced rather than merely documented.
9. **The eval surface G5 will touch is already carrying provisional numbers.**
   rah-007's 24-item ground-truth review was **descoped** on 2026-09-08 (nothing
   consumed the baseline: no build target, no gate, no CI), and with it the
   re-baseline rah-008 owed. Every figure in
   `skills/learn/learn-grade/references/eval-dataset/EVAL-RESULTS.md` is now
   explicitly provisional against draft truth, and `index.json` still reports
   `draft: 24, reviewed: 0`. G5's harness is a **separate** measurement surface
   (RACE/FACT over research reports, not learn-grade's misconception eval), so
   there is no double-booking — but if G5 reuses any of that dataset or its
   metrics script, the draft-truth caveat travels with it.

---

## 5. Assessment verdict

**5 of 5 goals NOT MET.** This is expected for a phase-opening assessment: every
goal describes capability that does not exist yet, and each gap is confirmed by
a command, not inferred.

The work is **additive to a healthy contract**, not a rescue. The predecessor
left stage contracts, labels, provenance, and a working headless spawn path; the
gap is entirely in execution topology (G1, G2), output structure (G3), bounded
failure (G4), and measurement (G5).

**Two things must not be carried into planning as fact:** the UAR-REM-002
remediation status (V2, identifier does not exist) and Onyx's benchmark standing
(V4, unmeasured). Any change that depends on either must state the dependency and
its unverified status.

---

## Unresolved review findings

Adversarial review: k3 judge, producer `claude-opus-5`, artifact mode, two rounds.
Receipts under `.kbd-orchestrator/phases/deep-research-onyx-parity/review/assess/`
(`round1/`, then `packet.json` + `findings.json` for round 2). Both rounds
returned **verdict PASS**; both passed the findings sycophancy gate at 0.0 strict.

### Round 1 (PASS: 3 WARNING, 1 SUGGESTION) — all accepted

| # | Finding | Disposition |
|---|---|---|
| 1 | UAR citations were unreproducible: the packet contains no `universal-agent-runtime` tree | **Accepted.** Added a preamble naming the external sibling checkout as the root for every section-0 citation |
| 2 | Header said 11/11 archived while goals.md said 10 | **Accepted, inverted.** The judge assumed the header was wrong; in fact `ls .kbd-orchestrator/changes/archive/ \| grep -c rah` returns 11 and no active rah change remains. `goals.md` was stale because rah-007 was descoped and archived after it was written. Both documents now read 11/11 |
| 3 | Fixing D-A and D-B triggers constraints the assessment never mentioned | **Accepted.** Added a constraint-impact table naming C-01's same-change regeneration duties, C-05 bash 3.2 for any launchd-reachable fan-out, and the one-Cargo-build rule |
| 4 | rah-007/rah-008 eval work not surfaced although G5 touches that surface | **Accepted.** Added as open question 9, recording that both were descoped and that the draft-truth caveat travels with the dataset if G5 reuses it |

### Round 2 (PASS: 2 WARNING, 3 SUGGESTION) — three accepted, one rejected, one deferred

| # | Finding | Disposition |
|---|---|---|
| 1 | The preamble's "nothing under this repo resolves those paths" could be misread, since `substrate/exec-tier-w/versions.toml` exists in-repo | **Accepted.** Named the lookalike explicitly and labelled V1–V3 external-checkout evidence pending re-verification at analyze |
| 2 | The constraint table prescribes npm commands that "appear nowhere" and may not exist | **REJECTED — the claim is false.** All four exist in `package.json`: `generate:services-manifest` → `node scripts/generate-service-manifest.mjs`; `check:services-manifest` → the same with `--check`; `check:distribution` and `validate:codex` → `generate-skill-system-distribution.js`. Three of them were executed during the predecessor's certification and are recorded passing in `docs/research-agent-hardening-evidence.md`. The judge inferred absence from the packet's contents rather than from `package.json` |
| 3 | The stale-count correction contradicts a goals.md that already reads 11/11 | **Accepted.** True *after* round 1's fix, which corrected goals.md. Reworded to say both now agree |
| 4 | The two-level depth ceiling is flagged in the goals but absent from the open questions | **Accepted.** Added as open question 8, with a proposed default and the enforcement point |
| 5 | D-B's row lists only C-01; republishing the plugin surface may also trigger a docs constraint | **Deferred to analyze, not dismissed.** The pack's constraint set as written names C-01 for generated surfaces; whether a driver republish alters the *documented* plugin surface depends on whether D-B is fixed by regeneration alone or by a version bump, which analyze decides. Recorded here so the plan cannot skip the question |

One finding across both rounds was rejected, with the refuting command recorded.
