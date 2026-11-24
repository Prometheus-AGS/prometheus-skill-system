# Plan — vlbv-r4-portability

The runtime bound the [v2 contract](adopted-contract.json) at plan revision 17, binding revision 1. Its ten scenarios remain the basis for acceptance.
Method: VLBV. Every task below is implemented before the single B4 run; nothing
executes generated code earlier (D1). LSP diagnostics are static reading (D5).

## Tasks

| Task | Deliverable | Scenarios |
|---|---|---|
| T1 | `evals/vlbv-protocol/cases/*.json`: 23 cases (AT 01–22 + §18). Each has an id, AT text reference, a classification (`agent` or `machinery`), the prompt/fixture, expected behaviour, and a deterministic check where one exists (file effects, forbidden commands in transcript) plus a grader rubric line for free-text judgement. A schema plus `scripts/check-skill-evals.js`-style shape check. | EVAL-* |
| T2 | `evals/vlbv-protocol/run.mjs` + `adapters/<harness>.mjs`. Per run: create a scratch project (`kbd-init`, installed pack generation), a scratch HOME and harness home, and copy the minimum auth file (claude: keychain is per-user so HOME relocation must be verified; codex: `auth.json`; opencode/kimi/qwen: their documented auth files). Hash real harness homes and the login-keychain item listing (attributes only) before and after. Capture transcripts and file effects under `evals/runs/<harness>/<case>/<sample>/`. Delete scratch, verify no auth leftovers. `--keep-scratch`/`--expect-leftover` for the negative control, and `--stub exit0` provisions a temporary no-output harness binary for neg-notrun-1. `evals/runs/` is gitignored local evidence. Bash-free Node so it runs on launchd/bash 3.2 hosts. | EVAL-CLAUDE..QWEN-1, AUTH-ISO-1 |
| T3 | `evals/vlbv-protocol/grade.mjs` + `validate-report.mjs`. Cross-model grading through liter-llm (`kbd_complete`-equivalent REST), family-aware grader choice, 3 fixed samples, all retained. Reports `evals/reports/<harness>.{json,md}` with per-case k-of-3, pass/fail/not-run counts, transcript digests, grader identity. Machinery verdicts merged as `harness-independent`. Uninstalled harnesses → the 17 agent cases NOT_EXERCISED with reason, plus the shared machinery verdicts. Declared k: capability-gating cases and AT 04 need 3-of-3, others 2-of-3; a sample is PASS only when its deterministic check passes and the grader says PASS. `validate-report.mjs` re-runs every deterministic check from the retained artifacts and rejects (a) a PASS citing a missing transcript and (b) a PASS whose transcript contains the forbidden command. `scan-secrets.mjs` covers `evals/runs/**` and `evals/reports/**` (AT 22). | EVAL-*, EVAL-NOTRUN-1 |
| T4 | `evals/vlbv-protocol/machinery.mjs`: drive the real `kbd-phase-close-eval.sh`, `kbd-closure-packet.mjs` and the KBD runtime in a scratch project for AT 05, 06, 09, 10, 15, 22, plus `--negative --at ATnn` for each of the six ATs: that AT's trigger is removed and the real script must accept. | EVAL-MACH-1 |
| T5 | `evals/vlbv-protocol/apply-capabilities.mjs`: move a `capabilities.json` field to `supported` only from report evidence whose case declares that capability field (field-to-case linkage in the corpus) (exercised, date, version, evidence path); leave every other field unchanged. Only a PASS case counts as evidence. Add the field-to-case linkage check to `scripts/check-harness-adapters.js`; `--negative` writes three copies (no evidence; a PASS case of another capability; a FAIL case of the right capability) and each must be rejected. Then `generate:harness-adapters` and `--check` for C-01 idempotence. | CAP-UPDATE-1 |
| T6 | CR-33. (a) Record an immutable B4 authorization event in the phase (`boundary-authorization.json`: timestamp, contract digest, authorizer), written when the boundary is authorized and before the first attempt; `value-scenario-verify`/`boundaries.md` say so. (b) `kbd-phase-close-eval.sh` scans `boundaryPacket.runnerOutputGlobs` and passes artifacts with mtime earlier than the event to `kbd-closure-packet.mjs`, which writes `preAuthorizationArtifacts[]` and a "Pre-authorization runner artifacts" section ("none found" when empty; "no authorization event recorded" when absent — never silent). (c) Schema: add `runnerOutputGlobs` to the v2 `boundaryPacket`. (d) `scan-fixture.mjs` builds the mixed and all-after fixtures. Report only; no guard. | SCAN-PRE-1 |

## Order

0. **Adopt the v2 contract** through `kbd-contract-adopt.sh` (digest per `digestNormalization`, planRevision pinned, phase bound). The recorded adoption satisfied this prerequisite for T1–T6.

Then T1 → T4 → T2 → T3 → T5 → T6, then documentation (vlbv skill references, CHANGELOG
Unreleased), then B4 authorization (owner) → the boundary packet once.

## Boundary packet

As declared in the contract: 28 commands, `singleRun`, machinery cases first (their verdicts feed every report), stop on AUTH-ISO-1 FAIL before any further harness; negative controls write only under `evals/reports/_negative/`. Expected spend: 255 agent harness sessions (17 × 3 × 5) plus grading; the 6 machinery cases run once (owner decision D-R4-03).

## Exit with partial verification

T1–T6 are recorded complete. The phase remains open. The original acceptance route requires verified behavior. For an operator who wants to end this phase with incomplete verification, use the existing signed `supersede-exit` route. This plan edit proposes that decision. The operator makes it separately.

| Evidence to preserve | Treatment at exit |
|---|---|
| `EVAL-MACH-1` | Retain the machinery PASS and its six negative controls. |
| `EVAL-CLAUDE-1`, `AUTH-ISO-1` | Retain both recorded B4 failures. Claude could not authenticate in the scratch environment. Interrupting the runner left scratch credentials that required manual cleanup. |
| Remaining seven scenario IDs | The B4 packet records them as `NOT_EXERCISED`. Preserve the partial B5 evidence, including the blocked Kimi sessions. Withdraw these seven from R4 with a proposed destination of `vlbv-portability-verification-follow-up`. |

[Proposed exit details](proposed-exit-details.json) maps each scenario to its evidence or destination. It also routes all six functional scope items to that proposed follow-up because the retained evidence does not qualify the full scope. B4 recorded 22 minutes. Total spending across repairs is unmetered. The follow-up is a backlog destination and requires separate authorization to start.

1. The operator reviews the evidence, limits and destination. If they choose this exit, record their exact words and decision-log reference in a working copy of the details file under `operatorDecision.text` and `operatorDecision.reference`.
2. From the registered checkout, generate and validate the unsigned manifest using the [operator exit procedure](../../../docs/guide/15-hooks-and-lifecycle.md#operator-decided-phase-exit), phase `vlbv-r4-portability` and that completed details file. The template checks the bound contract, evidence hashes and coverage against live state.
3. The operator signs the validated manifest. Submit that manifest and signature with `kbd-phase-close-eval.sh --superseded` as documented. Confirm `prometheus kbd phase disposition --id vlbv-r4-portability --json` returns `CLOSED_SUPERSEDED` with the three exercised verdicts and all withdrawals. Reflect can then report the signed disposition. Until the runtime records it, R4 remains open.

This exit grants neither release approval nor a promotion of any harness capability. CPC-010, CPC-011 and CPC-013 belong to the separate Companion plan and have no bearing on R4 closure.

## Risks carried

R1 auth-in-isolation per harness (Claude Code's keychain login may not follow a
relocated HOME), R2 OpenCode isolation, R3 cost, R4 grader independence, R5
unrelated dirty `.kbd-orchestrator` projections kept out of commits.

## Review record

- Round 1 (glm-5.3; MiniMax-M3 returned unusable output): 3 CRITICAL, 4 WARNING, all fixed.
- Round 2 (glm-5.3): 3 CRITICAL, 4 WARNING, 1 SUGGESTION — all fixed (deterministic re-verification, declared k, PASS-only capability evidence, provisioned stub, linkage check, runs secret scan + gitignore, adapter regeneration, baseline path).
- The two-round limit is reached. **Round-2 fixes are not independently re-verified**; CR-30 therefore puts adoption to the owner.
