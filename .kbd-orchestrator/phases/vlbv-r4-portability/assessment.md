# Assessment — vlbv-r4-portability

Stage: assess. Method: VLBV (pack default, no deviation). Method context is
`BLOCKED: adopted contract unavailable`. That is expected before adoption: this
assessment supplies the contract's inputs.

## Goals (from goals.md)

1. CR-31: protocol evals for AT 01–22 and the §18 fresh-session check, run per
   harness capability profile.
2. CR-31 exit: one report per harness with pass, fail and not-run counts. A
   capability becomes `supported` only with exercised evidence (FR-H7).
3. CR-33: the closure packet lists runner output artifacts dated before boundary
   authorization, or states that none were found. Report only.
4. Adopt a v2 contract before generation. Implement completely before one B4 run.

## Recalled lessons that shape this assessment

- *"The test sourced the helper directly and did not exercise installer
  dispatch"* (team-learning-hardening). Eval cases must drive each harness
  through its real CLI entry point and the installed pack. A case that calls a
  pack script directly measures the script, not the harness.
- *"Live and integration runs found defects that mocked and earlier gates had
  missed"* (team-aware-learning-memory-impl). Grading must read real transcripts
  and real file effects in the scratch project, not the agent's self-report.
- *"Treat One-Build Guard BLOCKED Phase Gates as Rerunnable"*. A harness case
  that cannot run (no auth, no isolation) is `BLOCKED` or `NOT_EXERCISED`, never
  PASS.
- *"KBD Task Closure Requires Ledger Synchronization"*. Close tasks with
  `begin-task`/`end-task`, not `kbd-apply mark-done` alone.
- From issue-195-closeout's closure report: Codex (SRF-001) and OpenCode
  (SRF-002) were dropped by amendment because a **scratch `CODEX_HOME` was not
  logged in** and **OpenCode isolation is unsupported**. These are the two
  known blockers for CR-31 and are carried below as risks R1 and R2.

## Current state (observed)

| Area | Observed |
|---|---|
| `evals/` | Holds only `skill-activation/` (a 30-case static corpus plus budgets). `scripts/check-skill-evals.js` validates its shape only. **No runner drives any harness.** |
| AT 01–22 | Defined verbatim in `skills/testing/vlbv/references/requirements.md` as challenge cases. No executable form exists. |
| §18 fresh-session check | Defined in `skills/testing/vlbv/references/supersession.template.md:53`: after a fresh-session handoff, the assistant identifies the current method, rejects rebuilding a retired harness, retains a relevant product failure, and preserves a mandatory approval gate. |
| Capability profiles | `shared/harnesses/capabilities.json`: 8 profiles × 7 status fields. One value is `supported` (codex `skillRoots`); the other 55 are `unknown`. The schema (FR-H7) already rejects `supported` without `verified.exercised`. |
| Harness CLIs installed | claude 2.1.325, codex-cli 0.166.0, opencode 2.3.0, kimi 2.5.3, qwen 0.27.1 (dated probe: `evidence/harness-baseline.txt`). zed is an editor CLI alias with no headless agent mode. minimax-code and mimo-code are not installed. **Headless invocation is unverified for every harness** (a probe is a model session); it is revalidated before adoption and recorded at B4. |
| CR-33 surface | `shared/scripts/kbd-phase-close-eval.sh` (275 lines) writes the closure packet through `closure_packet()` → `kbd-closure-packet.mjs`. `boundary-results.json` carries `attemptId` and `priorAttempts` with ISO timestamps. **No canonical "boundary authorized at" instant is recorded today**, and no contract field declares which paths are runner outputs. |

## Gaps

| # | Gap | Goal |
|---|---|---|
| G1 | No eval corpus for AT 01–22 / §18. | 1 |
| G2 | No runner that drives a harness headlessly in an isolated scratch project with the installed pack. | 1 |
| G3 | No grader. A non-producer judge with a declared stochastic profile (samples, grader, stopping rule) is required (VLBV 21). | 1, 2 |
| G4 | No per-harness report format, and no path from report evidence to `capabilities.json` `verified.exercised`. | 2 |
| G5 | Not every AT is an *agent-behaviour* case. AT 05/06/09/10/15 test `kbd-phase-close-eval.sh`/closure rules and AT 22 tests sanitization, all harness-independent. **All 22 ATs plus §18 stay in every harness report.** The six machinery cases execute once against the real scripts, and their immutable results are counted in each harness report, labelled `harness-independent`. No AT is dropped from any report. | 1 |
| G6 | No canonical boundary-authorization instant exists. **Resolution:** record an immutable authorization event (timestamp, contract digest, authorizer) when B4 is authorized, before any attempt. Attempt timestamps stay execution metadata only. Ordering invariant: contract adopted < implementation complete < B4 authorized ≤ first attempt. | 3 |
| G7 | No declaration of which paths count as runner output (test reports, `target/`, `playwright-report/`, coverage, junit/cucumber JSON). The contract's `boundaryPacket` must declare the scanned globs. | 3 |
| G8 | Goal 4 gates are not yet testable: (a) locate and adopt the v2 contract through `kbd-contract-adopt.sh` with `kbd-contract-lint.mjs` passing; (b) map every scenario to implementation and boundary evidence; (c) no generation before the binding is recorded; (d) exactly one B4 run after implementation is declared complete, with B5 only on owner authorization. This is the first live v2 adoption. | 4 |

## Proposed scenario frontier (value-scenario-discovery; proposals until adopted)

| ID | Actor → useful outcome | Source | Reason now | Likelihood |
|---|---|---|---|---|
| EVAL-1 | Owner → a per-harness report of how claude-code honours VLBV agent-behaviour cases | CR-31, VLBV 19 | Primary harness; all 7 fields unknown | high |
| EVAL-2 | Owner → the same report for codex | CR-31 | Installed; SRF-001 open | medium (auth risk) |
| EVAL-3 | Owner → the same report for opencode, kimi, qwen-code | CR-31 | Installed | medium/UNKNOWN |
| EVAL-4 | Owner → machinery cases (AT 05, 06, 09, 10, 15, 22) run once against the real close-eval/closure scripts | CR-31, VLBV 05–10 | Harness-independent | high |
| EVAL-5 | Owner → the §18 fresh-session check per harness | CR-31, §18 | Proves recovery without hooks | high |
| CAP-1 | Owner → `capabilities.json` updated only from exercised report evidence; unexercised fields stay unknown | FR-H7 | Exit criterion | high |
| SCAN-1 | Reviewer → closure packet lists pre-authorization runner artifacts | CR-33, VLBV 04/10 | Detects early execution without a guard | high |
| SCAN-2 | Reviewer → closure packet states "none found" when every runner artifact postdates authorization | CR-33 | Negative control for SCAN-1 | high |
| SCAN-3 | Reviewer → with a mixed fixture (artifacts both before and after the authorization event), the packet lists **exactly** the earlier ones | CR-33 | Date-discrimination oracle: an implementation that lists every artifact fails | high |

Deferred with reason: zed, minimax-code and mimo-code are reported `NOT_EXERCISED`
(no headless CLI or not installed). The rare-critical example: a harness that
*runs* a test before B4 during an eval must surface as FAIL on AT 04, never be
averaged away.

## Adopted-outcome contract inputs

| Input | Value |
|---|---|
| User operation | Run `evals` per harness; close the phase and read the closure packet. |
| Observed baseline | 55/56 capability values unknown; no runner; no pre-authorization scan. |
| Desired observable result | One report per harness with pass/fail/not-run counts; capabilities updated only with evidence; closure packet scan section. |
| Supported environments | This Mac (darwin arm64), the five installed harness CLIs. |
| Required safety checks | Scratch `HOME`/`CODEX_HOME`/`PROMETHEUS_PLUGIN_ROOT` per run (CLAUDE.md hook-testing rule); no writes to real harness state; no secrets in reports (AT 22). |
| Exclusions | zed/minimax-code/mimo-code execution; CI; per-edit tests; tool guards (CF-36). |
| Risk acceptance | **UNKNOWN — owner**: model spend and trial count (see open decisions). |
| Decision owner | Repository owner. |

## Risks

- **R1 (auth in isolation).** A scratch `CODEX_HOME` (and likely the other
  harness homes) has no login. Real-home execution is excluded by the safety
  checks. The remaining path is copying the minimum auth material into a scratch
  home outside the tracked tree, deleted after each run, with a check that no
  real harness state changed. Copying credentials is the owner's decision. Where
  it is declined, that harness reports `BLOCKED`.
- **R2 (OpenCode isolation).** Recorded as unsupported in issue-195. It may need
  `XDG_*` relocation; this is UNKNOWN until tried at B4.
- **R3 (cost/time).** About 17 agent-behaviour cases × 5 harnesses × N samples are
  real model sessions.
- **R4 (judge independence).** The grader must not share a family with the
  harness under test where avoidable; the producer for claude-code is Claude.
- **R5 (dirty working tree).** KBD re-projected other phases' `progress.json` and
  `tasks.md` at phase start. These are unrelated to R4 and must stay out of R4
  commits.

## Open owner decisions

1. **Auth material in scratch homes (R1/R2).** May the runner copy each
   harness's minimum auth file into a temporary scratch home outside the repo,
   deleted after the run? If not, that harness reports `BLOCKED`.
2. **Samples per case** (stochastic profile): 1 (behaviour shown to occur),
   3 (minimum reliability signal), or 5?
3. **Grader model:** cross-model via liter-llm (MiniMax-M3 / k3.1-pro), as in
   the 1.15.0 reviews?

## Review record

Round 1 (judge MiniMax-M3; k3.1-pro returned HTTP 524) found 3 CRITICAL and
3 WARNING. All six were accepted and folded in: G5, G6, G8, SCAN-3, R1, and the
dated baseline. Receipt: `review/assess/findings.json`.

## Knowledge gaps (from prior-context; `/learn-goal` offered, not run)

- `memory-index-partition.py` keeps file order rather than ranking entries.
- Codex memory generation is set by hand, not checked by the installer (since
  addressed by `codex-memories-config.sh` in 1.15.0; verify).
- `.prometheus/cadence/procedures/refresh-skill-pack.sh` is unversioned.
- `memory-envelope.integration.mjs` publishes to the live `:23001` service.
- Cortex mirror tested only against a stub.
- SubagentStart delivery skipped under extreme load.
- `kbd-apply mark-done` leaves the canonical ledger stale.

## Unresolved review findings

Round 2 (same judge) returned unusable output (`[judge] ERROR: unusable judge
output`). The round-1 findings are addressed above, but **no independent judge
has confirmed the revisions**. The plan-stage review of the contract must
re-check G5, G6/SCAN-3 and G8 explicitly.
