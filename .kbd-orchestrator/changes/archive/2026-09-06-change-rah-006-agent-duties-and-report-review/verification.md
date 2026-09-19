# Verification — change-rah-006-agent-duties-and-report-review

Repository: `prometheus-skill-pack`
Depends on: `change-rah-003-stage-contract-driver`, `change-rah-005-claim-labels-and-provenance`

## Acceptance criteria

- All four agent files carry `tools:`; report-synthesizer's list contains no search or fetch tool.
- `driver-contract.sh --scenario review-critical`, `--scenario review-unavailable`, and `--scenario review-without-verify` pass against the real driver.
- The adversarial-review fixture suite passes with the new research-target test.

## Verify commands

Every acceptance criterion above maps to a command here; run from the repository root, locally, after the coherent edit batch. A command that cannot run (for example, another Cargo build is active) is recorded BLOCKED with the reason, never skipped silently.

```verify
set -a; . ~/.prometheus/kbd/secrets.env; set +a; export KBD_PRODUCER_MODEL="${KBD_PRODUCER_MODEL:-claude-fable-5-1}"
for f in skills/research/deep-research/agents/*.md; do grep -q '^tools:' "$f" || exit 1; done
bash skills/research/deep-research/tests/driver-contract.sh --scenario review-critical
bash skills/research/deep-research/tests/driver-contract.sh --scenario review-unavailable
bash skills/research/deep-research/tests/driver-contract.sh --scenario review-without-verify
bash skills/process/adversarial-review/tests/run-fixture-suite.sh
```

## Evidence

Run 2026-09-06 on the uncommitted working tree (branch `feat/cpc-001-002-integration-contract`, base `cfbc262`); no hosted CI.

| Gate | Command | Result |
|---|---|---|
| Agent allowlists | `grep '^tools:'` over the four agents; synthesizer list has no search/fetch | all four carry `tools:`; `report-synthesizer`: `Read, Write, Edit, Grep, Glob` |
| Driver contract, all scenarios | `tests/driver-contract.sh` under bash 5 and `/bin/bash` 3.2 | `128 passed, 0 failed` both (61 prior + 67 new across review-clean-verified, review-critical, review-warning, review-unavailable, review-without-verify, review-real-dispatch, frontmatter-body) |
| review-clean-verified | `--scenario review-clean-verified` | passing gate + PASS review ends `verified` and passes the drift check; the judge saw `partial` and a pending review; negative control (judge unavailable) ends `partial` |
| review-real-dispatch | `--scenario review-real-dispatch` | real packet builder + CLI-faithful dispatch stub: PASS recorded with the producer identity in packet and checkpoint; exit 3 → `judge unavailable`; exit 2 → `judge refused`; no producer → `judge failed (dispatch exited 4)`, report `partial` |
| frontmatter-body | `--scenario frontmatter-body` | a body that quotes the frontmatter keys is rejected at stage 09 (runner invoked); a proper report is accepted without the runner |
| review-critical | `--scenario review-critical` | 9/9: exit 0, checkpoint `review.verdict BLOCK` (1 CRITICAL, 1 WARNING), sidecar `Verification: BLOCKED` with `Blocked: adversarial review: 1 CRITICAL`, WARNING listed under `Review warnings`, report frontmatter and manifest `partial`, manifest verdict BLOCKED, exported package passes the drift check |
| review-warning | `--scenario review-warning` | 5/5: verdict PASS, warning text in the sidecar, frontmatter unchanged |
| review-unavailable | `--scenario review-unavailable` | 8/8: `blocked_review: blocked: judge unavailable (dispatch exited 3 ...)`, sidecar line present, PASS WITH NOTES, manifest partial, drift check passes, judge never called |
| review-without-verify | `--scenario review-without-verify` | 8/8: judge never dispatched, `blocked: review refused, stage 05 verification missing or invalid` in checkpoint, sidecar, and ledger; manifest partial; resume does not re-judge |
| Verifier before reviewer | fixture judge log records `stages_completed` at call time | full-run: exactly one call, after `09`; refused and unavailable runs: zero calls |
| Research packet target | `tests/test-research-target.sh` (bash 5 and 3.2) | 17/17: three files as separate fields, goals carry query and sub-questions, truncation recorded at default and forced (1000-byte) cap with inline markers, missing report/plan/sidecar → exit 2 and no packet, usage errors exit 1/2 |
| Adversarial-review fixture suite | `tests/run-fixture-suite.sh` (Group A live k3 judge, B, C, D) | `Passed: 28, Failed: 0, Judge calls: 4 / 6`; flawed → BLOCK, clean → PASS, `verified-distinct` on all four |
| Static fixtures unaffected | `check-research-package.sh --package` labelled, partial | exit 0 both (the `blocked_review` clause changes nothing for them) |
| Strict validation | `npm run validate:strict` deep-research, adversarial-review | PASS |
| Constraints | C-02 secrets grep over `files.txt` diff; C-05 `mapfile`/`declare -A` grep over touched scripts | 0 hits each |

Verify gate: `kbd-apply verify` PASSED at task close-out with the gateway key and producer exported, and FAILED once at archive time in a shell without them; the failing command was only `run-fixture-suite.sh` (Group A, live judge: `KBD_PRODUCER_MODEL is unset`), reproduced by running the block with and without credentials. The verify block now sources `~/.prometheus/kbd/secrets.env` and exports the producer first, which is the same precondition adversarial-review's own workflow states.

Two defects found by the new scenarios and fixed in the driver: the exit trap's jq expression concatenated a string inside an object literal without parentheses (a compile error on every non-block failure path, latent since rah-003), and a resumed run that re-ran a stale stage never restored `status: complete` when export was skipped as still valid.

QA: C-01 no generator input touched beyond SKILL.md files (skills-index reconciliation is rah-011); scope amendment in spec.md for write-provenance.sh, check-research-package.sh, okf-research-format.md, stage-runner.sh, the artifact mandate, the fixture suite, and the new fixture judge. Adversarial review: disposition in spec.md.

Verdict: PASS WITH NOTES (two review rounds disposed in spec.md, round-2 fixes covered by scenarios but not re-vetted; `tools:` is advisory on harnesses that ignore the key, as documented).
