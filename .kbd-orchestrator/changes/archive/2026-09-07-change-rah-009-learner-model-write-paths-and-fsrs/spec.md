# change-rah-009-learner-model-write-paths-and-fsrs

**Title:** Add gap, session, and certification write paths to learner-model and replace the FSRS stub
**Repository:** `prometheus-skill-pack`
**Phase:** research-agent-hardening
**Goal:** G2
**Depends on:** `change-rah-001-compile-baseline-and-timestamps`, `change-rah-008-learn-artifact-and-corpus-coherence`
**Backend:** native-kbd

## Why

`main.rs` dispatches exactly five methods; `GapRecord` and `SessionRecord` are typed but never written; there is no `certified_at`; learn-certify's session gate has no data (assessment G2, verified; goal G2). Separately, `fsrs.rs` stores `difficulty` and never reads it (assessment learn finding 26). The FSRS replacement is OPTIONAL within this change: no phase goal requires it, so tasks 1 and 3 may be skipped with a recorded reason and the phase certifies without them. Analysis D-09, D-10.

## What Changes

- Add `add_gap`, `add_session`, and `set_certified` JSON-RPC methods; add `certified_at: Option<String>` to `ConceptState`; extend the store fold so gaps and sessions survive load, save, and CRDT import; learn-grade, learn-practice, and learn-certify SKILL.md call the new methods.
- Run `cargo tree` for `rs-fsrs` and `fsrs-rs` and record the dependency weight; adopt `rs-fsrs` 1.2.1 for `next_review()` if its tree is scheduler-only, otherwise port the FSRS-6 stability and difficulty update formulas into the existing module; either way `difficulty` is read and updated on every review.
- Integration test `tests/rpc_roundtrip.rs` drives the built binary over stdin and stdout: seed from a survey, five observations, `add_gap`, `add_session`, `set_certified`, `review`; asserts mastery moved only from the fifth observation, gaps and sessions are returned by `get_concept`, `certified_at` is set, and `difficulty` changed after a review.

## Scope

Files this change may create, edit, or delete (tasks.json `files` is the per-task view):

- `substrate/learner-model/Cargo.toml`
- `substrate/learner-model/src/main.rs`
- `substrate/learner-model/src/store.rs`
- `substrate/learner-model/src/types.rs`
- `substrate/learner-model/src/fsrs.rs`
- `substrate/learner-model/tests/rpc_roundtrip.rs`
- `substrate/learner-model/README.md`
- `skills/learn/learn-grade/SKILL.md`
- `skills/learn/learn-practice/SKILL.md`
- `skills/learn/learn-certify/SKILL.md`
- `openspec/specs/learn-model-coherence/spec.md`

## Capabilities

- `learn-model-coherence (write-path requirements added)`

## ADDED Requirements

### Requirement: Gaps and sessions persist
WHEN `add_gap` or `add_session` is called, THEN the record is returned by `get_concept` after a fresh `load`.

#### Scenario: Round trip
- **WHEN** the binary is driven over stdin with add_gap, add_session, then load and get_concept
- **THEN** both records are present with their ids

### Requirement: Certification is recorded
WHEN `set_certified` is called, THEN `certified_at` is set on the concept and learn-certify can read it.

#### Scenario: Certify
- **WHEN** set_certified is called for `c1`
- **THEN** get_concept returns a non-null `certified_at`

### Requirement: Difficulty is live (optional)
WHEN the optional FSRS tasks are executed, THEN `next_review()` SHALL read and update `difficulty`; WHEN they are skipped, THEN the skip and its reason are recorded in verification.md and this requirement is marked deferred.

#### Scenario: Review
- **WHEN** the optional tasks were executed and a review with rating Hard follows one with rating Good
- **THEN** `difficulty` differs between the two returned cards

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

- If `rs-fsrs` proves heavy, whether the ported formulas should carry the FSRS-6 default parameters verbatim (default: yes, with a source citation).

## Dependency note

Declared dependency `change-rah-008-learn-artifact-and-corpus-coherence` was not
complete when this change ran (it is gated behind the operator's rah-007 review,
D-15). The write paths, `certified_at`, the FSRS adapter, and the rpc_roundtrip
gate do not depend on rah-008's artifact-path or corpus-schema work. The three
SKILL.md edits here touch different sections than rah-008 will (Step 6/8 write
calls and the learn-certify gate wording versus artifact paths and corpus
fields); rah-008 must merge with, not overwrite, these sections. rah-011's
skills-index regeneration covers all three files.

## Unresolved review findings

Adversarial review: k3 judge, producer `claude-fable-5-1`, diff mode, two
rounds (cap). Receipts under
`.kbd-orchestrator/phases/research-agent-hardening/review/change-rah-009-learner-model-write-paths-and-fsrs/`
(`round1/`, then `packet.json` + `findings.json` for round 2).

### Round 1 (BLOCK: 1 CRITICAL, 4 WARNING) — all accepted and fixed

| # | Sev | Finding | Disposition |
|---|---|---|---|
| 1 | CRITICAL | learn-practice keyed `add_session` by `$GOAL_ID` while learn-grade/learn-certify use `$LEARNER_ID`; Gate 2 could never see the sessions | Fixed: all learn-practice calls (including the pre-existing `add_observation`) use `$LEARNER_ID`; paragraph states it is the learner DID, not the goal id |
| 2 | WARNING | Closing `add_session` without `started_at` restamped the start to the close time (wholesale replace) | Fixed: `store::add_session(.., preserve_start)` keeps the existing record's `started_at` when the call omits it; probed by the test (`started_at` == opening value after a close without it) |
| 3 | WARNING | `gap_id` was to be "kept in the grade file" but the grade schema had no field | Fixed: `gap_id` added to the grade result schema (null when the write failed); Step 6 text says how `resolve_gap` uses it |
| 4 | WARNING | `parse_timestamp` treated any non-string (number, object) as absent → silently "now" | Fixed: `parse_optional_timestamp` returns an error for present non-string values; probed (`certified_at: 12345`, `ended_at: {..}` → error, nothing written) |
| 5 | WARNING | Open-session example piped through `jq -r .session_id` with no error check → literal `"null"` id creates a phantom session | Fixed: example captures the reply, extracts the id only when `.error` is null, skips the close call and warns when empty; degraded path documented |

### Round 2 (BLOCK: 1 CRITICAL, cap reached) — accepted and fixed after the cap

| # | Sev | Finding | Disposition |
|---|---|---|---|
| 1 | CRITICAL | `add_gap` read `label`/`severity` with `as_str()`, so a present non-string (`123`, `true`) was accepted as absent, contradicting the spec's refusal rule | Fixed after round 2 (no further judge round): `optional_enum` distinguishes absent/null/"" from a present non-string, which is refused; probed by the test (`label: 123`, `severity: true` → error; gap count unchanged after a fresh load). Recorded here rather than re-judged because the 2-round cap applies; the refuting probe is in `tests/rpc_roundtrip.rs` |

No finding was rejected. Both rounds passed the findings sycophancy gate
(score 0.0, strict).
