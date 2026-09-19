# change-rah-006-agent-duties-and-report-review

**Title:** Restrict each research agent's tools, and route the final report through adversarial-review before delivery
**Repository:** `prometheus-skill-pack`
**Phase:** research-agent-hardening
**Goal:** G4
**Depends on:** `change-rah-003-stage-contract-driver`, `change-rah-005-claim-labels-and-provenance`
**Backend:** native-kbd

## Why

The four research agents have no `tools:` key; nothing orders verifier before reviewer; adversarial-review is wired into nothing under research or learn (assessment G4, verified). Analysis D-06, cand-008.

## What Changes

- Add `tools:` frontmatter to `research-planner` (read-only), `source-verifier` (read plus fetch and search), `contradiction-resolver` (read-only), and `report-synthesizer` (read and write, no search or fetch); each prompt restates its allowlist so harnesses that ignore the key still see it; SKILL.md documents the advisory degradation.
- Add a `research` target to `build-review-packet.sh --mode artifact` that packs `report.md`, `<slug>.provenance.md`, `plan.md`, and the goals of the run; document it in the adversarial-review SKILL and output contract.
- The driver runs adversarial-review after stage 09 and before stage 10, and only after the stage 05 verification artifact (`sources/credibility.json`) exists and validates; without it the review is refused, the sidecar records `blocked: review refused, stage 05 verification missing or invalid`, and the package label cannot be `verified`. A CRITICAL finding sets the sidecar verdict to BLOCKED and the report frontmatter to `partial`, WARNINGs are appended to the sidecar, and a gateway absence is recorded as `blocked: judge unavailable` rather than skipped silently. This is the verifier-before-reviewer rule: verification (stage 05 plus the verifier agent) always precedes review (adversarial-review), and the two never run in one dispatch.
- Fixture test in adversarial-review's suite: packet build for the research target on the labelled fixture package, asserting the three files are present and truncation is recorded.

## Scope

Files this change may create, edit, or delete (tasks.json `files` is the per-task view):

- `skills/research/deep-research/agents/research-planner.md`
- `skills/research/deep-research/agents/source-verifier.md`
- `skills/research/deep-research/agents/contradiction-resolver.md`
- `skills/research/deep-research/agents/report-synthesizer.md`
- `skills/research/deep-research/SKILL.md`
- `skills/research/deep-research/scripts/run-research.sh`
- `skills/process/adversarial-review/scripts/build-review-packet.sh`
- `skills/process/adversarial-review/SKILL.md`
- `skills/process/adversarial-review/references/output-contract.md`
- `skills/process/adversarial-review/tests/test-research-target.sh`
- `skills/research/deep-research/tests/driver-contract.sh`

Scope amendment (recorded during apply): files owned by earlier changes that
the review step required, edited minimally and listed so the packet is honest
about what moved:

- `skills/research/deep-research/scripts/write-provenance.sh` (rah-003): a
  review verdict of BLOCK makes the sidecar verdict BLOCKED; WARNING claims
  are listed under `Review warnings`; the interim wording no longer assumes
  checkpoint mode.
- `skills/research/deep-research/scripts/check-research-package.sh` and
  `references/okf-research-format.md` (rah-005): one clause added to the
  derivation rule, `blocked_review != null` derives `partial`, so a refused or
  unavailable review can never leave a package `verified` at any scale.
- `skills/research/deep-research/tests/fixtures/stage-runner.sh` (rah-003):
  `FIXTURE_DROP_05_AT_09=1` knob for the review-without-verify scenario.
- `skills/research/deep-research/tests/fixtures/judge.sh` (new): the fixture
  judge behind `RESEARCH_JUDGE_CMD`; it exits 7 unless the packet it receives
  carries `research_report`, `research_provenance`, and `research_plan`.
- `skills/process/adversarial-review/assets/reviewer-mandate-artifact.md`:
  a `target = research` section and the `file` enum for report findings.
- `skills/process/adversarial-review/tests/run-fixture-suite.sh`: Group D
  (no judge calls) runs `test-research-target.sh`.

## Capabilities

- `research-pipeline-execution (review requirement added)`

## ADDED Requirements

### Requirement: The synthesizer cannot invent a source
The report-synthesizer agent SHALL declare no search or fetch tool in its allowlist.

#### Scenario: Frontmatter
- **WHEN** the four agent files are parsed
- **THEN** each has a `tools:` list and report-synthesizer's contains no search or fetch tool

### Requirement: The report is judged before delivery
WHEN stage 09 completes, THEN the driver dispatches adversarial-review on the research target before stage 10 runs.

#### Scenario: Critical finding
- **WHEN** the fixture judge returns one CRITICAL
- **THEN** the sidecar verdict is BLOCKED and stage 10 still exports the package with `verification_status: partial`

### Requirement: Judge absence is visible
WHEN no gateway is reachable, THEN the sidecar records `blocked: judge unavailable` and the package label is not `verified`.

#### Scenario: No gateway
- **WHEN** the driver runs with the gateway unreachable
- **THEN** the sidecar contains the blocked line

### Requirement: Verification precedes review
The review step SHALL be refused unless the stage 05 verification artifact exists and validates; verification and review never run in one dispatch.

#### Scenario: Review without verification
- **WHEN** a package reaches stage 09 with `sources/credibility.json` absent or invalid
- **THEN** the driver does not dispatch the judge, the sidecar records `blocked: review refused, stage 05 verification missing or invalid`, and stage 10 exports with `verification_status: partial`

## Constraints

- Implementation-first, integration-only evidence (CLAUDE.md highest-precedence policy): finish the coherent edit batch, then run the smallest full-integration gate named in `verification.md`. No unit tests, mocks, or snapshots count as delivery evidence.
- One Cargo build machine-wide at a time. Check `pgrep -x cargo` before any `cargo` command; if another build is active, wait or record BLOCKED, never start a competing build. `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Local-only validation: no GitHub Actions run is evidence.
- Verification labels are `verified | unverified | blocked | inferred` on claims and `PASS | PASS WITH NOTES | BLOCKED` on provenance. A gate that could not run is recorded BLOCKED with the reason, never described as passed.
- Scripts that launchd may invoke stay bash 3.2 compatible (constraint C-05): no `mapfile`, no `declare -A`.
- Every script touched keeps `set -euo pipefail` semantics and non-zero exit on failure; no silent `|| true` on a gate.
- The pack never depends on the Companion or any extension (integration contract rule 1); capability is discovered, never assumed (rule 2).
- Constraint C-01 (generated artifacts): `SKILL.md` and `skill.toml` files edited in this phase are inputs only to `generate:skills-index`; `skill-system.json`, the harness adapters, and the service manifest are not edited by any change in this phase. `change-rah-011-integration-evidence-and-docs` is the named reconciliation change: it regenerates the skills index and runs `check:distribution`, `check-harness-adapters.js`, and `check:services-manifest` at certification. No earlier change claims distribution certification.

## Open Questions

- Whether a CRITICAL finding should also stop export entirely (default: no; export a partial package so the run is auditable).

## Unresolved review findings

Adversarial review (k3 judge via gateway, producer `claude-fable-5-1`, `--mode diff` on `files.txt`), two rounds, receipts in `.kbd-orchestrator/phases/research-agent-hardening/review/change-rah-006-agent-duties-and-report-review/` (`round1/` and the round-2 `findings.json`).

### Round 1 — BLOCK (2 CRITICAL, 2 WARNING, 2 SUGGESTION)

| # | Finding | Disposition |
|---|---|---|
| C1 | The rule table says `verified` needs "no claim is blocked or unverified" but the jq rule and both checker implementations only demote for critical non-verified or any blocked claim; a non-critical `unverified` claim matched no row | **Accepted, fixed.** `blocked or unverified` on any claim derives `partial` in the doc rule, the python derivation, and the jq derivation; the `partial` row says so. Probe: labelled copy with claim-002 `unverified` derives `partial` and fails against declared `verified` in both paths. |
| C2 | A clean full run self-blocks at stage 10: stage 09 can only write `partial` (review not yet run), nothing raises it after a PASS review, and the drift check rejects the report/manifest disagreement | **Accepted, fixed.** `check-research-package.sh --derive <dir>` prints the derived status (the jq rule, now one function used by the checker and the driver); after every review outcome the driver rewrites the report frontmatter to it, raising or lowering, and stage 10 copies it into the manifest. New scenario `review-clean-verified` (passing gate + PASS review → `verified`, drift check passes) with a negative control (same gate, judge unavailable → `partial`). |
| W1 | Goals extraction for the research packet swallowed all failures (`|| true`), leaving an empty `goals` field | **Accepted, fixed.** Fails closed with exit 2 and no packet; probe with an unreadable plan.md. |
| W2 | The lowering path was never exercised: the fixture report always said `partial` | **Accepted, fixed.** `FIXTURE_REPORT_GATE=pass` makes stage 09 write `verified` provisionally with a passing gate; review-critical, review-unavailable, and review-without-verify now assert the drop from `verified` to `partial`. |
| S1 | A recorded refusal or judge-unavailable block was permanent across resumes | **Accepted, fixed.** A recorded verdict is final; a blocked review is retried on resume after stale stages are re-run (stage 10 is re-exported so manifest and sidecar copy the retry). Scenarios: unavailable then resume → one judge call, `verified`; refused then resume → stage 05 re-runs before the single judge call; refused and still invalid → blocks at 05, no judge call. Documented in SKILL.md. |
| S2 | Dead `SLUG=` recomputation in the driver | Deleted. |

### Round 2 — BLOCK (1 CRITICAL, 2 WARNING, 1 SUGGESTION); cap reached, fixed and not re-vetted

| # | Finding | Disposition |
|---|---|---|
| C1 | The judge would always see a provisional `verified` frontmatter beside a sidecar saying `Adversarial review: not run`, which the mandate lists as a report-honesty defect, so no mandate-following judge could ever let a clean run end `verified` | **Accepted, fixed.** Before the packet is built the driver rewrites the frontmatter to the pre-review derivation (a full run derives `partial` there) and the interim sidecar says `Adversarial review: pending`; the mandate now states that a pending review is expected and never a finding. The fixture judge records what it saw; `review-clean-verified` asserts `saw=partial | pending` and a final `verified`. |
| W1 | Every scenario injected the fixture judge, so the real dispatch branch and its CLI contract were never exercised, and the driver passed no producer identity | **Accepted, fixed.** The driver records `producer_model` in the checkpoint at creation (from `KBD_PRODUCER_MODEL`, else `RESEARCH_PRODUCER_MODEL`, else the resumed checkpoint) and passes it to the packet builder and dispatcher; dispatch exit codes are mapped to distinct blocked reasons (3 unavailable, 2 refused, other failed). New scenario `review-real-dispatch` runs the real packet builder and a CLI-faithful `dispatch-judge.sh` stub via `RESEARCH_ADV_DIR`, covering pass, gateway-down, garbage output, and a no-producer negative control. |
| W2 | Stage 09 contract greps the whole report for frontmatter keys, so a body that quotes them passes | **Accepted, fixed.** The checks run against the extracted frontmatter block only. New scenario `frontmatter-body` plants such a report and proves it is rejected (the runner is invoked) while a proper report is accepted without the runner. |
| S1 | The sidecar filename re-derives the slug from the id format, duplicating `slug.sh` | **Accepted, fixed.** The driver records `slug` in the checkpoint at creation; `write-provenance.sh` reads it and falls back to suffix stripping only for older checkpoints. |

Round-2 fixes are covered by the scenarios named above but were not seen by the judge.
