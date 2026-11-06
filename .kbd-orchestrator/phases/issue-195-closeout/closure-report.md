# Closure report: issue-195-closeout

**Canonical disposition:** CLOSED, verdict PASS, recorded 2026-11-06 by
`shared/scripts/kbd-phase-close-eval.sh --phase issue-195-closeout`. A retry was
a no-op (ALREADY-CLOSED). No successor phase was created (17 phases before and
after).

| Field | Value |
|---|---|
| Contract | `36e11d1bbb7a07a8d465f0efafb7bebfe6638b460fe941175f23bbf5ca2ea235` (amendment-2, binding revision 3) |
| Evidence | `boundary-results.json`, sha256 `0dd78362b601c202c0c9441b8d4d34db51b529e6ff0a43f503be5d7278d70dfe` |
| Final attempt | `evidence/attempts/2026-11-06T06-55-26.060Z-0bcf865a-8347-45da-a3a9-54e3958edab4` |
| Candidate | main `5d575cdc` (PR #209 on #208); `prometheus` 1.14.0 built from that commit, sha256 `66c9d2f7…` |
| Run | `sovereign-sync-service-reliability-20260829` |

## Six claims, reported separately

1. **Authored implementation: complete.** PR #209 (merged as 5d575cdc):
   - Unacknowledged policy conflicts set `CONFLICT`, and `--require-resolved` exits 2. A hash-bound `KBD-POLICY-ACK` decision clears it.
   - The 11 stage skills carry one identical selected-method core, enforced by `scripts/check-selected-method-core.mjs`.
   - VER-001 and VER-003 bodies, the CON, REF and PKG scenarios, and driver isolation.
   - Repair 2 changed the product instructions in `value-scenario-verify` and in the conflict bullet of the core.
2. **Measured contract acceptance: 10 of 10 bound scenarios PASS** on merged main:
   - CON-001..003, POL-001..003, REF-001, PKG-001, VER-001, VER-003.
   - The core check and the distribution check both exit 0.
3. **Demonstrated runtime value: within scope.**
   - Claude Code sessions recover the selected method and contract.
   - They stop before dependent execution on an unacknowledged conflict, and proceed after a matching acknowledgement.
   - n=1 sample per scenario: this shows the behaviour occurs, not that it is reliable.
4. **Unresolved obligations:**
   - Codex (SRF-001) and OpenCode (SRF-002) were removed from scope by signed amendment-2, and no claim is made for them. Reasons: the scratch `CODEX_HOME` was not logged in, and OpenCode isolation is unsupported.
   - Review warnings W2–W4 and S1–S2 (spec wording vs. code; no effect on results) are carried as known gaps.
   - The #194 portion of `issues-194-195-196` is outside this phase.
5. **Release eligibility: not established.** No release certification was run for this phase.
6. **Release authority: not granted.** Nothing was released, tagged or globally installed. The main-built binary lives in `tools/prometheus-cli/target/release` only.

Cross-model review (glm-5.3 via a per-run override) found 0 CRITICAL issues and 4 warnings. One warning was disproved; the other three are listed above. The review is labelled `unverified-producer-unknown` because the producer identity is not configured.

## Attempt and amendment lineage

All attempts are retained under `evidence/attempts/`; none was overwritten.

| Attempt | Contract | Non-passing (excluding NOT_EXERCISED) |
|---|---|---|
| 2026-10-31T17-00 / T17-13 | 3ab50017 | session scenarios BLOCKED (no scratch login) |
| 2026-11-05T23-18 | d546e3a6 (amendment-1) | REF-001, PKG-001 BLOCKED; VER-003 FAIL |
| repair 1: regenerated hashes; `noProgress` reported by fixture | | |
| 2026-11-05T23-29 | d546e3a6 | POL-002, VER-001, VER-003 FAIL |
| repair 2: product instructions | | |
| 2026-11-05T23-54 | d546e3a6 | none (SRF-001/002 NOT_EXERCISED) |
| 2026-11-06T06-55 | 36e11d1b (amendment-2) | none |

- **Amendment-1** (signed 2026-11-05): one fresh boundary for the candidate rebased onto #207.
- **Amendment-2** (signed 2026-11-06): removed SRF-001/002 and authorised one confirming run on merged main, with zero repairs and unchanged ceilings. It was needed for two reasons:
  - PR #208 changed `substrate/kbd-runtime/src/change_identity.rs` after the amendment-1 boundary.
  - The evaluator never counts NOT_EXERCISED as PASS.

## Budget

These figures are cumulative since the amendment-1 meter start (2026-11-05T23:18:22Z) and are self-reported.

| Dimension | Value | Class | Ceiling |
|---|---|---|---|
| timeMinutes | 54 (boundary elapsed: 45 + 9) | measured | 180 |
| toolCalls | 125 | estimate: transcript count of parent-session calls, excluding nested sessions | 240 |
| repairAttempts | 2 | measured | 2 (at ceiling: `budget-exhausted` note) |
| noProgress | 1 (attempt 2 did not reduce the failing count) | measured | 2 |
| tokens | unknown | unknown | none |

Wall-clock time is longer than measured boundary time; it was mostly idle, waiting on the operator merge and signature.

## Posting outcome

- **#195 and #196:** closed automatically when PR #209 merged. The PR body carries the acceptance-to-evidence table. No separate evidence comment was posted.
- **#201** (zeespec replay): commented with the #207 regression and the #208 re-fix; it was already closed.
- **`issues-194-195-196`:** partial disposition recorded as the canonical decision `issues-194-195-196-partial-disposition`, with a note in `../issues-194-195-196/DISPOSITION.md`.

## Operational notes

- **Checkout:** fast-forwarded to 5d575cdc so closure could hash the merged sources. The local pre-rebase drafts are kept in `git stash` ("pre-ff-to-5d575cdc …"), and the live `.kbd-orchestrator` state was restored over the fast-forward.
- **Installed binary:** `~/.local/bin/prometheus` is still 1.13.2. Run `scripts/install-binaries.sh` to bring it in line with main.
