# change-rah-008-learn-artifact-and-corpus-coherence

**Title:** Unify the feynman artifact path across the learn skills and make the grounding corpus carry what learn-grade reads
**Repository:** `prometheus-skill-pack`
**Phase:** research-agent-hardening
**Goal:** G2
**Depends on:** `change-rah-007-eval-ground-truth-review`, `change-rah-005-claim-labels-and-provenance`
**Backend:** native-kbd

## Why

feynman-loop writes `artifacts/<artifact-id>.json`, learn-retain globs `artifacts/<concept-id>-*.json`, learn-certify reads `artifacts/<concept-id>/`; none can find the others' files. learn-grade expects `key_points[]` and `misconceptions[]` per source; `content-grounding-kb.sh` emits neither, so transfer-problem generation has no input. The script is triplicated byte for byte (assessment G2, verified; analysis D-07, D-08).

## What Changes

- Artifact path becomes `goals/<goal-id>/artifacts/<concept-id>/<artifact-id>.json`: `write-artifact.sh` writes it, learn-retain globs `artifacts/<concept-id>/*.json`, learn-certify reads the concept directory; all three SKILL.md files agree.
- `shared/scripts/content-grounding-kb.sh` emits `key_points[]` (sentences of `content_summary`) and `misconceptions[]` (entries flagged `is_misconception`) per source and keeps the existing fields; the two skill copies become thin wrappers that exec the shared script; learn-grade Step 1 documents the real shape; the eval `HARNESS.md` workaround is removed.
- Author `openspec/specs/learn-model-coherence/spec.md` covering artifact path, corpus schema, artifact provenance (from change-rah-005), learner-model write paths (from change-rah-009), and the eval baseline rule (from change-rah-007).
- Integration test `skills/learn/tests/learn-coherence.sh`: runs the real grounding script against a local fixture KB and validates the corpus with jq; writes an artifact with `write-artifact.sh`, then asserts learn-retain's documented glob and learn-certify's documented path both resolve it.
- Re-run the eval against the reviewed truth with the new corpus shape and record the post-change baseline.

## Scope

Files this change may create, edit, or delete (tasks.json `files` is the per-task view):

- `skills/learn/feynman-loop/SKILL.md`
- `skills/learn/feynman-loop/scripts/write-artifact.sh`
- `skills/learn/learn-retain/SKILL.md`
- `skills/learn/learn-certify/SKILL.md`
- `skills/learn/learn-grade/SKILL.md`
- `skills/learn/learn-grade/references/eval-dataset/HARNESS.md`
- `skills/learn/learn-grade/references/eval-dataset/baseline-snapshot.json`
- `skills/learn/learn-grade/references/eval-dataset/metrics-summary.json`
- `shared/scripts/content-grounding-kb.sh`
- `skills/learn/learn-goal/scripts/content-grounding-kb.sh`
- `skills/learn/learn-kb/scripts/content-grounding-kb.sh`
- `skills/learn/tests/learn-coherence.sh`
- `skills/learn/tests/fixtures/kb-local`
- `openspec/specs/learn-model-coherence/spec.md`

## Capabilities

- `learn-model-coherence (new)`

## ADDED Requirements

### Requirement: One artifact path
feynman-loop, learn-retain, and learn-certify SHALL resolve the same file for a given goal, concept, and artifact id.

#### Scenario: Round trip
- **WHEN** `write-artifact.sh` writes an artifact for concept `c1`
- **THEN** the glob documented in learn-retain and the path documented in learn-certify both resolve that file

### Requirement: The corpus carries key points and misconceptions
Every source emitted by `content-grounding-kb.sh` SHALL carry `key_points[]` and `misconceptions[]`.

#### Scenario: Local KB
- **WHEN** the script runs against the fixture KB with `--include-misconceptions`
- **THEN** `jq '.sources[] | has("key_points") and has("misconceptions")'` is true for every source

### Requirement: One grounding script
The two skill-local copies SHALL exec the shared script and contain no duplicated logic.

#### Scenario: Wrapper
- **WHEN** either wrapper is run
- **THEN** it produces byte-identical output to the shared script for the same arguments

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

- Whether `key_points[]` derivation by sentence split is good enough or the KB adapters should ask the backend for key points (default: sentence split now; adapter-native key points is a later phase).

## Dependency note

Applied on operator instruction (`/kbd-apply change-rah-008…`, 2026-09-07)
before its declared dependency `change-rah-007` finished: rah-007 task 2 (the
operator's review of the 24 ground-truth items) has not landed, so every item
is still `review_status: draft`. Everything in this change that does not depend
on reviewed truth was delivered and gated; the live eval re-run (task 5) is
recorded BLOCKED with the reason, not performed against draft labels (D-15).
`change-rah-009` (learner-model write paths) was archived on 2026-09-07 before
this change; the spec authored here states its requirements as current
behaviour because they are, and names the owning change on each.

## Unresolved review findings

Adversarial review: k3 judge, producer `claude-fable-5-1`, diff mode, two
rounds (cap). Receipts under
`.kbd-orchestrator/phases/research-agent-hardening/review/change-rah-008-learn-artifact-and-corpus-coherence/`
(`round1/`, then `packet.json` + `findings.json` for round 2).

### Round 1 (BLOCK: 3 CRITICAL, 5 WARNING, 1 SUGGESTION)

| # | Sev | Finding | Disposition |
|---|---|---|---|
| 1 | CRITICAL | Task 5 marked complete while its live re-run is BLOCKED on draft truth; the regression "pass" compares unchanged results against an unchanged baseline | **Accepted.** The change verdict is recorded **BLOCKED** on the eval re-baseline criterion (verification.md). The task ledger stays closed because the task's deliverable (the recorded post-change baseline entry) exists and says BLOCKED; the live re-run is owed after rah-007 task 2 and is listed in EVAL-RESULTS.md with the exact steps |
| 2 | CRITICAL | `--goal-id ../../outside` escaped the learn home (only concept_id/artifact_id were guarded) | **Accepted, fixed.** Probe confirmed `/tmp/escape` was created. All three ids now pass one single-path-component guard (no `/`, `\`, `.`, `..`); gate asserts a traversal goal_id and a `..` artifact_id are refused and nothing lands outside the learn home |
| 3 | CRITICAL | learn-grade Step 1 told the grader to derive the fields by hand, contradicting the spec's `--normalize`, never-a-second-derivation rule | **Accepted, fixed.** Step 1 now runs `--normalize` and grades the normalized file; gate asserts Step 1 names `--normalize` and no longer says "derive them the same way" |
| 4 | WARNING | Unquoted `--include=*.sh` inside `bash -c` glob-expands in the cwd, silently narrowing the scan | **Rejected as a defect, applied as hygiene.** Probe: `bash -c 'echo --include=*.sh'` from the repo root prints the literal; a glob only expands when a file matching the *whole* word (`--include=…sh`) exists, and `install.sh` does not. The pattern is quoted anyway and a positive control now proves `write-artifact.sh` is scanned |
| 5 | WARNING | Wrapper `exec bash` from PATH, so the `/bin/bash 3.2` parity run never ran the shared script under 3.2 | **Accepted, fixed.** Wrappers exec `"${BASH:-bash}"` (the invoking interpreter); the gate also runs the shared script directly under `/bin/bash` and compares bytes |
| 6 | WARNING | Test asserts `concept_id` in the success JSON but the script only prints `ok`+`path` | **Rejected.** `write-artifact.sh` has emitted `artifact_id` and `concept_id` in its success object since before this change (unchanged tail, line 126); the gate passed against it. The header comment was stale and now lists the full object |
| 7 | WARNING | "Authored key_points kept" check was positional and vacuous at N=0 | **Accepted, fixed.** The check joins on `source_ref`, sorts, and first asserts the authored count is > 0 (it is 7) |
| 8 | WARNING | Spec states rah-009-owned behaviour and learn-practice's `add_session` while rah-009 is not a declared dependency; verification said "both new capabilities" | **Accepted in part.** rah-009 is archived (2026-09-07) and its behaviour is current; the Dependency note above records the order. The stale "both new capabilities" wording in verification.md is corrected to the one capability. The evidence table records the grep proving learn-practice names `add_session` |
| 9 | SUGGESTION | Two `subject_to_slug` implementations (lib vs inline fallback) can drift; installed generations ship the script without `lib/` | **Accepted.** The fallback is byte-identical to `lib/slug.sh` (the judge's "different sed expression" claim is wrong: the pre-change inline already used `\{1,\}`), and the gate now copies the script to a directory without `lib/` and asserts byte-identical output, which pins `corpus_id` parity across both layouts |

### Round 2 (BLOCK: 1 CRITICAL, 3 WARNING — cap reached)

The round-2 dispatch first failed with the judge's "unavailable (exit 3)" after
its default timeout on a 137 KB packet; the gateway answered HTTP 200 on
`/v1/models` throughout. Re-dispatched with `ADV_JUDGE_TIMEOUT=900` and it
completed. That is a transport limit, not a review outcome.

| # | Sev | Finding | Disposition |
|---|---|---|---|
| 1 | CRITICAL | Task 5 marked complete while the eval re-run is BLOCKED on draft truth; the regression pass is vacuous | **Already accepted and recorded** (same as round-1 finding 1). The change verdict is **BLOCKED** on that one criterion in verification.md; the vacuity of the regression pass is stated in the evidence table and in EVAL-RESULTS.md. No new information |
| 2 | WARNING | HARNESS.md claimed all three eval corpora carry no `key_points`/`misconceptions`, contradicting the test and EVAL-RESULTS.md | **Accepted, fixed.** Counted directly: `cellular-respiration-corpus.json` has 7 authored `key_points` of 12 sources; the two meta-corpora have 0 of 18 and 0 of 16; none carries `misconceptions`. HARNESS.md now states those counts and that normalization keeps authored lists verbatim. Gate asserts the false sentence is gone |
| 3 | WARNING | `run_normalize` rebuilt each source from eight fields, silently dropping per-source extras such as a KB's own `concept_id` | **Accepted, fixed.** Normalization now preserves every unowned per-source key and folds it under the rebuilt object (rebuilt fields win on conflict, order preserved). Probed with a corpus carrying per-source `concept_id`, `tags`, and `custom`; three gate assertions added |
| 4 | WARNING | learn-certify and learn-retain hardcoded `~/.prometheus/learn`, so under `PROMETHEUS_LEARN_HOME` they would read a different store than `write-artifact.sh` writes | **Accepted, fixed.** Both now resolve `<learn-home>` as `${PROMETHEUS_LEARN_HOME:-~/.prometheus/learn}`, matching feynman-loop and the spec. Gate asserts both files name the variable |

Three of four round-2 findings were real and are fixed; the CRITICAL restates
the already-recorded blocked criterion. Both rounds passed the findings
sycophancy gate (round 1 score 0.018, round 2 score 0.0, strict).
