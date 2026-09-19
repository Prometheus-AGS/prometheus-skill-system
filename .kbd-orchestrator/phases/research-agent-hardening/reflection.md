# Reflection — research-agent-hardening

**Phase:** `research-agent-hardening`
**Opened:** 2026-09-03 (runtime revision 613) · **Closed:** 2026-09-08
**Changes:** 11 of 11 implemented, **11 of 11 archived**
**Repository:** `prometheus-skill-pack`, branch `feat/cpc-001-002-integration-contract`, base `cfbc262`, uncommitted

---

## Goal outcomes

| Goal | Verdict | Evidence |
|---|---|---|
| **G1 Execution** — daemon and driver actually execute every stage and fire hooks; `--daemon-job`, timestamps, one output root, one manifest schema | **MET** | rah-001 (chrono timestamps, compile baseline), rah-002 (one output root, schema-validated manifest, package id shape), rah-003 (stage-contract driver: boundary validation, checkpoint on every exit, `--resume`, two execution modes), rah-004 (`--daemon-job` defined and running a real headless child). Proven end to end by the rah-011 evidence run: seven stage boundaries through the driver, and a daemon job that drove all seven autonomously and exported |
| **G2 Learn coherence** — one artifact path, one corpus schema, learner-model write paths and `certified_at` | **MET** | rah-009 (`add_gap`, `resolve_gap`, `add_session`, `set_certified`, `certified_at`, rs-fsrs adapter so difficulty is live; 1-test round-trip over the built binary), rah-008 (one artifact path across feynman-loop/learn-retain/learn-certify, `key_points[]` + `misconceptions[]` on every source, two 504-line duplicate scripts reduced to 16-line wrappers, `--normalize` for older corpora; 58-assertion suite under bash 3.2 and 5) |
| **G3 Provenance** — four-label vocabulary, provenance sidecar, plan-as-ledger, `--resume` reads the checkpoint | **MET** | rah-005 (label enum, derivation rule, sidecar on every exit path), rah-003 (plan.md carries task ledger, verification log, decision log; `--resume` re-runs any stage whose artifacts stopped validating). The evidence run shows all of it on a real package |
| **G4 Duties** — tool allowlists, verifier before reviewer, scale gate, adversarial review of the report | **MET** | rah-006. Demonstrated live in the evidence run: the report review fired for real (132 s, k3 judge) and returned four warnings that were genuine defects in the producer's own report |
| **G5 Scoring** — renormalised credibility with a sensitivity artifact, content-addressed claim ids, contradicts edges | **MET** | rah-010. The deferral branch (D-17) was **not** taken; `scoring-graph.sh` passes 45 assertions and the scorer refuses an empty registry (exit 2) rather than synthesising scores |

**All five goals MET.** No goal was closed on inference; each has a suite or a
recorded run behind it. **All 11 changes are archived**; rah-007's remaining
tasks were descoped on 2026-09-08 with the reason recorded in its verification.md.

## Certification

Recorded in `docs/research-agent-hardening-evidence.md`, all local, no hosted CI:

- Four bash integration suites: driver-contract 128, scoring-graph 45, learn-coherence 58, adversarial-review 28 — 0 failed.
- Six cargo suites across `prometheus-research` and `learner-model` — 0 failed.
- Seven validators including all three C-01 reconciliation checks — all PASS.
- Both generators proven idempotent by hash across two runs.
- Two research packages validated: driver run PASS, daemon run PASS WITH NOTES.

## What the phase found that it did not set out to find

Three defects surfaced only because the work was exercised for real rather than
asserted:

1. **The launchd plist for the research daemon sets no `PATH`** (D-A). The
   service could not find `claude` or `codex` although both are installed.
   Sibling templates carry a `__PROMETHEUS_PATH__` placeholder; this one does
   not, and the installer substitutes only `__HOME__`. **Open, carried forward.**
2. **The installed plugin generation ships a pre-phase stub driver** (D-B). The
   daemon's `~/.claude` driver candidate resolved to a 1920-byte script with
   none of the stage contract, against a 30718-byte repo driver. **Open, carried
   forward.** Interim override: `RESEARCH_DRIVER`.
3. **rah-011's own task-4 verify string could not pass.** It chained four suites
   in one shell, two of which require opposite `KBD_PRODUCER_MODEL` states. The
   string was corrected in the archived ledger with a note; every gate passes
   individually.

In all three cases the daemon or the driver refused to report a success it had
not achieved. That refusal is the phase's most valuable output.

## Where the process worked

- **The headless child refused to fake a run.** Given a stub driver, it
  diagnosed the mismatch, ruled out a stale-copy fluke by comparing all five
  copies on the machine, and stopped: "Running it positionally would have printed
  twelve lines of fake stage-completion JSON and exited 0 — a green result with
  an empty package. That is worse than reporting the block." Anti-sycophancy
  held inside an autonomous child with no observer.
- **Adversarial review earned its cost.** Across the phase it caught a
  laundered `verified` label, a path-traversal hole in `write-artifact.sh`
  (`--goal-id ../../outside` escaped the learn home), a JSON-RPC boundary that
  read a present-but-wrong-type value as absent, unrun Cargo suites presented
  under a "certification passes" heading, and a site page the spec named that had
  never been written. Round 2 of rah-011 found no defect in the shipped work —
  only inaccuracies in the evidence document itself.
- **Derived-never-chosen verification status.** `--derive` independently agreed
  with the manifest on both evidence packages, including the honest `unverified`
  on a run with no search tier.

## Where the process cost time

- **Two review rounds are the floor, not the ceiling.** Several changes needed
  fixes after the cap; those are recorded as post-cap with the refuting probe
  rather than re-judged.
- **A judge timeout reads exactly like a clean review.** rah-008's round-2
  dispatch exited 3 ("unavailable") on a 137 KB packet while the gateway was
  answering HTTP 200. Treating that as an all-clear would have skipped four
  findings. `ADV_JUDGE_TIMEOUT=900` is the fix; the failure mode is worth
  remembering.
- **Generated-surface drift is invisible until certification.** The distribution
  output went stale across the phase and only surfaced at the C-01 check.
- **Test placement is load-bearing.** A suite one directory too high failed
  `validate:strict` as a skill with no `SKILL.md`.

## Debt carried out of this phase

| Item | Owner |
|---|---|
| ~~rah-007 tasks 2–3~~ — **descoped 2026-09-08**, not deferred. Nothing consumes the eval baseline: no build target, no gate, no CI. The D-15 control had already done its job by stopping rah-008 from re-baselining against draft truth; producing a reviewed baseline was never load-bearing. learn-grade's F1 claim is downgraded from empirical to provisional so the docs no longer overstate it | closed |
| ~~The eval re-baseline rah-008 owes~~ — moot with the above; EVAL-RESULTS.md records that every number is provisional against draft truth | closed |
| Should learn-grade's accuracy ever be cited externally or become safety-critical, do the 24-item review then. `REVIEW-SHEET.md` and the dataset are unchanged on disk and `index.json` still reports `draft: 24, reviewed: 0`, so no future reader can mistake the state | future, if needed |
| D-A launchd `PATH`; D-B stale installed generation | install surface; both outside every change's file scope in this phase |
| `content-grounding.sh` (public-web variant) still emits neither `key_points[]` nor `misconceptions[]` | covered for now by learn-grade's `--normalize` route |
| Nothing in this phase is committed (~300 files changed) | operator decision |

## Recommended Next Phase

**`deep-research-onyx-parity`** — bring deep-research to functional parity with
the Onyx project's execution topology while keeping the evidence contract this
phase built.

Decided by the operator on 2026-09-08 as a **subsequent top-level phase**, not a
child and not an amendment (decision D-22). The rationale holds up: this phase's
five goals are met and none depend on threading, and the Onyx design explicitly
*depends on* what this phase produced — stage contracts, claim labels,
checkpoint, verifier ordering, derived verification status — and instructs
"resist renumbering."

Source: `docs/deep-research/deep-research-skill-vs-onyx-report.md`.

Candidate goals (input to assess, not a settled plan):

- **Thread execution** — replace the sequential internals of stages 02–04 with a constrained director plus isolated workers and a deterministic merge, leaving stage contracts, numbers, and validators unchanged.
- **Lossless handoff** — persist per-thread fact dossiers with inline content-addressed citations and claim quotes, so synthesis reads prose evidence rather than only compressed graph claims.
- **Multi-pass report** — split stage 09 into outline, parallel section writers, deterministic assembly, and a coherence editor, bound by a claim-set invariant.
- **Budgets and failure semantics** — cycle and wall-clock budgets at job, director, and thread level with force-complete paths, and a ledger row for every dispatch including failures.
- **Measurement** — a benchmark reporting RACE, effective citations, citation accuracy, and verified-claim ratio, so "on par with Onyx" becomes falsifiable.

**Assess must resolve first** (decision D-23), by command, not by inheriting the
report's assumptions:

1. liter-llm Rust API availability from UAR `Cargo.toml` / `versions.toml`.
2. UAR-REM-002 status for the three correctness defects.
3. The nature of the nested `universal-agent-runtime/crates/prometheus-skill-system` checkout — submodule, worktree, or drifted copy.
4. The Onyx benchmark standing (RACE ~54, "reported as #1 at points in 2026") is recorded **unverified** until `tests/bench/` actually runs.

Carry D-A and D-B into that phase's assess as known install-surface defects.
