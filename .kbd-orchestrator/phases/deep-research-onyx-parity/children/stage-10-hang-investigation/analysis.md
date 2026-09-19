# Analysis — stage-10-hang-investigation

Date: 2026-09-11
Phase: deep-research-onyx-parity
Child: stage-10-hang-investigation
Stage: analyze
Producer: k3 (opencode, kimi-for-coding/k3)
Revised: 2026-09-12 by glm-5.3 (opencode) — resumed after the 2026-09-11 session
died during adversarial vet; revised per review round 1 (see Adversarial review).

## Inputs

- `assessment.md` (2026-09-10) — goals, evidence, gap analysis.
- `handoff-in.md` (2026-09-11) — handoff from prior Claude Code session, retained as a **corroborating input only** (it is not part of the formal handoff chain — review round-1 WARNING). Its claim that `driver-contract.sh:309/:326` reach the live gateway is disproved by citations verified directly against the working tree this session: `skills/research/deep-research/tests/driver-contract.sh:288` writes a heredoc stub `dispatch-judge.sh` into `$ADV_STUB/scripts/`; lines `:309` and `:326` set `RESEARCH_ADV_DIR="$ADV_STUB"` for both scenarios; `skills/research/deep-research/scripts/run-research.sh:372-375` prefers `RESEARCH_JUDGE_CMD`, then the `RESEARCH_ADV_DIR` dispatch. No scenario in the suite can reach the live gateway.
- `review/assess/findings.json` — adversarial PASS (MiniMax-M3 vs k3, verified-distinct); two WARNINGs carried forward:
  1. Packet declared `prior_handoffs: null` while assessment cites handoff claims — handoff-derived claims must cite their source.
  2. Many file:line citations were absent from the packet `file_tree` (uncommitted branch) — citations must be verifiable.
- `goals.md` — 4 goals (exact blocking command; pre-existing vs introduced; fix proven by driver-contract full count; adversarial review of diagnosis before any fix).
- `prior-context.md` — ABSENT (step-3 memory recall skipped by design).
- `evolver-bridge.json` — ABSENT (step-4 evolver bridge not required).

## Problem restatement

`driver-contract.sh --scenario full-run` hangs intermittently (~1 run in 3) with zero assertions printed (timeout 90 → exit 124); identical retry passes 12/12; standalone full deep run through the same fixture passes in <60 s. The stage-10 call chain is known by reading:

`run-research.sh:478` → `review_report` :340 → `sync_report_status` :364 → `write-provenance.sh` :365 → `build-review-packet.sh` :367 → judge dispatch :372-377.

No blocking command has been isolated. The defect is **intermittent**, which invalidates the previous session's serial one-shot hypothesis-testing method (six theories disproved without finding the cause).

## Tiered research record

Budget: `max_queries_per_tier` 8, `max_minutes` 20. Queries used: 2 (Tier 4 only).

### Tier 1–3 — empty by design (FINDING A)

Both open gaps are methodology, not library adoption:
1. Diagnosing an intermittent hang — no package decides this; it is a test-method choice.
2. The heredoc-extraction fix — a repo-internal pattern already proven in sibling scripts on this host (see Appendix A); adopting an external library would be unjustified churn.

Negative results — off-the-shelf categories considered and rejected (recorded so Plan need not re-scan):
- **bash repeat/flaky runners (bats-core et al.)** — the driver suite is plain bash + env fixtures, not bats; a repeat loop with tracing is ~20 lines and leaves the acceptance command unchanged. Adopting a framework would rewrite green suites for no diagnostic gain.
- **process-tracing wrappers (strace/dtruss)** — used *inside* the cand-2 harness as a tool, not adopted as a framework; macOS SIP restricts dtruss on system binaries, so the harness offers `set -x` capture as the portable default with `sample`/`spindump` for live hang capture.
- **sandboxed/containerized re-runners** — the suspected mechanism is OS pipe-buffer state on this host; sandboxing changes the very conditions under which the hang reproduces (~1/3), destroying the signal.

Therefore Tiers 1 (gh search), 2 (Context7/docfork), 3 (npm/cargo/PyPI registries) were intentionally skipped. Recorded here per pipeline doc's partial-findings rule.

### Tier 4 — firecrawl_search (2 queries)

**Q1: flaky/intermittent test fix proof standards and root-cause frameworks**
- Race conditions / timing issues are ranked root cause #1 for flaky tests across independent frameworks (Autonoma "Five Root Causes"; Semaphore flaky-test elimination guide; ScienceDirect multivocal review S0164121223002327; Sauce Labs CI flakiness reduction).
- Recommended remediation pattern: re-run N times and use a statistical pass criterion rather than one-shot pass/fail.

**Q2: subprocess pipe-buffer deadlock (heredoc hangs)**
- `subprocess.Popen` with PIPE deadlocks when more than the OS pipe buffer (~64 KB) crosses stdin/stdout/stderr without concurrent draining; Python docs direct users to `communicate()`; multiple independent reports of hangs "indefinitely if stdout > ~65000 chars".
- This is a direct mechanistic match for an intermittent `python3 - <<'EOF'` heredoc hang: output volume near the pipe-buffer threshold makes the deadlock timing-dependent, fitting the observed ~1-in-3 intermittency.

## Candidate decisions

### Candidate 1 — Proof standard: N consecutive clean runs (ADOPT, build)

Goal 3's gate ("driver-contract passing its full count") is currently green 128/128 ×4 on 2026-09-09, yet the defect persists intermittently — so a single passing run certifies nothing for this defect class.

- Decision: restate Goal 3's proof standard as **N consecutive clean full-run passes** (suggested N ≥ 10, informed by observed ~1/3 failure rate: P(10 consecutive passes | still broken) ≈ (2/3)^10 ≈ 1.7% **under a run-independence assumption**).
- Independence caveat (review round-1 WARNING): an intermittent multi-process hang plausibly correlates consecutive runs (orphaned children, lock residue, pipe state, shared caches), which inflates the false-negative probability above the bound. The harness MUST run a pre-call cleanup (kill the leftover driver process tree, reset scratch dirs) before the statistical gate applies, and N MUST be re-sized from the failure rate cand-2 actually measures — 1.7% is indicative, not a guarantee.
- Evidence: Tier-4 flaky-test frameworks (see Q1).
- Tier: 4. Confidence: high (method), medium (the specific N).

### Candidate 2 — Diagnosis method: instrumented repeat-run reproduction harness (ADOPT, build)

Goal 1 (exact blocking command) was NOT met by reading alone; the chain is known but the blocking call is not isolated. For an intermittent defect, single-shot instrumentation is insufficient.

- Decision: build a repeat-run harness that loops `--scenario full-run` up to K times (or until hang), with per-call tracing (e.g. `set -x` capture or strace/dtruss on the child) so the first hanging run yields the exact blocked command and its stack.
- Evidence: race/timing ranked #1 flaky-test cause (Q1); previous session's serial one-shot method failed six times (assessment).
- Tier: 4 + assessment. Confidence: high.

### Candidate 3 — Heredoc extraction for `build-review-packet.sh` (BUILD-REQUIRED, conditional on Candidate 2 outcome)

`build-review-packet.sh` (919 lines, verified by `wc -l` this session; call 3 in the chain) embeds ~10 `python3` heredoc programs — the same construct that hung indefinitely in other scripts on this host and was fixed by extraction (the assessment's "six hung scripts" is tier-0 host history; the independently verifiable in-tree precedent is the `.sh`/`.py` companion list in Appendix A); never extracted here.

- Decision: **if** the Candidate-2 harness isolates the hang inside `build-review-packet.sh`'s python3 heredocs, extract them to standalone files following the repo-internal extraction pattern (Appendix A). Do NOT extract pre-emptively — Goal 4 requires an adversarially reviewed diagnosis before any fix is written, and a full deep run completed through this script in <60 s, so the heredocs are a lead, not a confirmed cause.
- Mechanism support: pipe-buffer deadlock (~64 KB threshold, Q2) explains intermittency.
- Tier: repo-internal pattern + Tier-4 mechanism. Confidence: medium (mechanism fits; site unconfirmed).

### Contested-choice check

No adoption choice at stake (all candidates are build/method decisions; no external library candidates). Score-gap < 15% escalation is not applicable. No `pmpo-elicit` escalation required.

## Goal-gap mapping

| Goal | Assessment status | Analyze disposition |
|---|---|---|
| 1. Exact blocking command with reproducible trace | NOT MET (chain enumerated; blocking command not isolated — status aligned with the prose per review round-1 WARNING) | Candidate 2 (instrumented repeat-run harness) is the path; reading exhausted |
| 2. Pre-existing vs introduced, first appearance | NOT MET | Harness output (Cand. 2) + git-bisect — **prerequisite**: the 551-file tree is uncommitted and bisect needs committed history, so Plan must either schedule a commit pass first (work item) or date via blame of the suspect heredoc block against the branch merge-base (limitations to be documented); undatable today |
| 3. Fix proven by driver-contract full count | NOT STARTED | Candidate 1 restates the gate as N ≥ 10 consecutive clean runs (independence caveat + pre-run cleanup + measured-rate re-sizing); fix follows diagnosis (Cand. 3 conditional) |
| 4. Adversarial review of diagnosis before fix | NOT STARTED | Satisfied by this stage's artifact-mode review of the diagnosis in Spec/Plan; no fix written in Analyze |

## Findings

- **FINDING A**: Tier 1–3 empty by design (no library adoption at stake) — see above.
- **FINDING B (CORRECTED 2026-09-12 — the original claim was factually false)**: the 2026-09-11 session asserted that `references/schemas/library-candidates.schema.json` (referenced by `kbd-analyze/SKILL.md`) "does not exist anywhere under `kbd-process-orchestrator/` (verified by glob)". It exists at `skills/process/kbd-process-orchestrator/references/schemas/library-candidates.schema.json` — the glob searched the wrong root. Adversarial review round 1 flagged this as CRITICAL; `library-candidates.json` has since been rewritten against the real schema and passes `jsonschema` (draft 2020-12) validation. Its shape changed accordingly: `verdict` enum (adopt/adapt/reference/reject), evidence restricted to tiers 1–4 (assessment-derived claims moved into `decision_rationale`), `build_required` items as `need`/`why_no_candidate`, and no undeclared top-level keys.
- **FINDING C** (carried from assess WARNING #1): handoff-derived claims in this analysis cite `handoff-in.md` explicitly; the disproved live-gateway claim is noted in Inputs.
- **FINDING D** (carried from assess WARNING #2): all file:line citations herein are on the uncommitted branch `feat/cpc-001-002-integration-contract`; verifiable locally but not from a fresh-clone packet.

## Adversarial review

- Round 1 (dispatched 2026-09-12 on resume; judge MiniMax-M3 via rest-gateway `localhost:4000`, producer k3, `cross_model_check: verified-distinct`, sycophancy screen pass): verdict **BLOCK** — 1 CRITICAL (FINDING B false: the library-candidates schema exists; the artifact was shaped against an inferred schema), 5 WARNINGs (Goal-1 status inconsistency; bisect-on-uncommitted; handoff-in not in the formal chain; independence assumption; Tier 1–3 justification thin), 1 SUGGESTION (unverifiable tier-0 counts). Full record: `review/analyze/findings.json` until round 2 overwrites it.
- Corrections applied in this revision: all of the above — FINDING B corrected and the artifact schema-validated; Goal 1 → NOT MET; Goal 2 bisect prerequisite surfaced as a Plan work item; handoff-in disproof re-anchored to directly verified file:line citations; Candidate 1 independence caveat + mandatory pre-run cleanup + measured-rate re-sizing; Tier 1–3 negative results enumerated; Appendix A added.
- Resume defect (process note): the dead 2026-09-11 session's review packet had been built with `--phase deep-research-onyx-parity` (the **parent**), embedding the parent's 2026-09-08 Onyx-parity analysis instead of this child's, and with `producer_model: "unknown"`. A round-1 dispatch on that packet would have reviewed the wrong artifact. Rebuilt with the child phase path and `KBD_PRODUCER_MODEL=k3` before dispatch.
- Round 2: dispatched after this revision; outcome recorded in `review/analyze/findings.json` and summarized in the stage handoff.

## Open questions

1. Observed intermittency (~1/3) is from a small sample; the harness (Cand. 2) should first establish a measured failure rate to size N in Candidate 1 rigorously.
2. Whether the ~64 KB pipe-buffer threshold is actually crossed by `build-review-packet.sh` heredoc output at stage-10 input sizes is unmeasured — the harness must capture output volume at the hang site.
3. If the hang isolates outside `build-review-packet.sh` (e.g. in `write-provenance.sh` or judge dispatch), Candidate 3 is void and a new fix candidate must be analyzed.

## Skip protocol

Not invoked.

## Appendix A — verifiable extraction precedent (tier-0 facts, checked 2026-09-12)

- `skills/process/adversarial-review/scripts/build-review-packet.sh` — **919 lines** (`wc -l`), embeds ~10 `python3 - <<'EOF'` heredoc programs; NOT extracted.
- Extraction pattern in tree (`skills/research/deep-research/scripts/`, `.sh` driver + `.py` companion pairs): `assemble-report`, `build-graph`, `check-graph-claims`, `check-manifest-files`, `check-manifest-schema`, `detect-contradictions`, `export-package`, `merge-threads`, `score-sources` — nine extracted companions as of this revision.
- The assessment's "six scripts hung and were fixed by extraction" is host history carried from `assessment.md` (tier-0, not independently verifiable from the tree); the list above is what Plan can verify.
