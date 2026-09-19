# change-rah-011-integration-evidence-and-docs

**Title:** Run the hardened pipeline end to end on a real query, record the evidence, and publish the documentation and version bumps
**Repository:** `prometheus-skill-pack`
**Phase:** research-agent-hardening
**Goal:** cross-cutting
**Depends on:** `change-rah-004-daemon-job-execution`, `change-rah-006-agent-duties-and-report-review`, `change-rah-009-learner-model-write-paths-and-fsrs`, `change-rah-010-source-scoring-and-graph`
**Backend:** native-kbd

## Why

Every earlier change proves its own contract with fixtures. The phase's goals are only met when a real research run produces a validated package with a provenance sidecar through the driver and through the daemon, and when the documentation site and skill versions describe what now exists. The dependency on change-rah-010 is conditional: if the plan's G5 deferral branch is taken and recorded in decision-log.md, this change proceeds without 010, writes the evidence document's scoring section as deferred to the named successor phase with G5 NOT MET, and omits `tests/scoring-graph.sh` from certification.

## What Changes

- Run `/deep-research --depth shallow` on a fixed query through the driver in a harness session; record the package path, the sidecar verdict, the adversarial-review findings, and the check-research-package result in `verification.md`; PASS WITH NOTES is acceptable, BLOCKED is recorded honestly.
- Start the same query through the daemon (`research_start`), confirm the headless child runs, and export the package with `research_export`.
- Documentation: update `site/docs/` research and learn pages, the deep-research and learn SKILL.md files' version fields, `CHANGELOG.md`, and `SKILLS.md`; regenerate the skills index; bump `metadata.version` on every touched skill.
- Final local certification: the driver, daemon, learner-model, scoring, learn-coherence, and adversarial-review suites; `check-research-package.sh`; `npm run validate:strict`; `npm run check:skills-index`; `openspec validate`.
- This change is the phase's constraint C-01 reconciliation change: it regenerates the skills index (the only generator that reads the SKILL.md files edited in this phase) and runs `npm run check:distribution`, `node scripts/check-harness-adapters.js`, and `npm run check:services-manifest` to prove no other generated surface drifted. No plist, unit, `skill-system.json`, or harness adapter source is edited in this phase, so those checks are expected to pass unchanged; a failure is recorded and repaired here, not deferred.

## Scope

Files this change may create, edit, or delete (tasks.json `files` is the per-task view):

- `skills/research/deep-research/SKILL.md`
- `skills/learn/feynman-loop/SKILL.md`
- `skills/learn/learn-grade/SKILL.md`
- `skills/learn/learn-retain/SKILL.md`
- `skills/learn/learn-certify/SKILL.md`
- `skills/learn/learn-practice/SKILL.md`
- `site/docs/research/deep-research.md`
- `site/docs/learn/feynman-loop.md`
- `CHANGELOG.md`
- `SKILLS.md`
- `SKILLS.md` (the generated skills index; there is no `skills-index.json` — the
  scope line naming one was wrong and is corrected here, adversarial round-1 S9)
- `docs/research-agent-hardening-evidence.md`
- `site/docs/substrate/prometheus-research.md` (scope amendment, round-1 F3: the
  daemon's headless execution contract is documented on the substrate page that
  exists, not on a research page that did not)
- `site/sidebars.js` (wires the new research page into the site)
- `skills/learn/feynman-loop/tests/learn-coherence.sh` and its `tests/fixtures/kb-local/`
  (scope amendment, round-1 F3/F5: relocated from `skills/learn/tests/`, which sat at
  the category level where `validate:strict` enumerates skills and therefore failed
  certification with `tests: SKILL.md is required but not found`. The relocation is
  this change's repair of a certification failure it found)

## Capabilities

- `research-pipeline-execution`
- `learn-model-coherence`

## ADDED Requirements

### Requirement: A real run produces a validated package
WHEN the driver runs a shallow research query in a harness session, THEN a package exists under the output root, validates, and has a provenance sidecar with a verdict.

#### Scenario: Shallow run
- **WHEN** the fixed query runs end to end
- **THEN** `check-research-package.sh --package <dir>` exits 0 and the sidecar verdict is PASS or PASS WITH NOTES, or BLOCKED with the blocked stage named

### Requirement: The daemon path matches the driver path
WHEN the same query is started through `research_start`, THEN the exported package validates identically.

#### Scenario: Daemon run
- **WHEN** the job completes
- **THEN** `research_export` returns the path and `verification_status`

### Requirement: Docs describe what exists
Every skill touched in this phase SHALL carry a bumped version and the docs site SHALL describe the driver, labels, sidecar, and daemon execution.

#### Scenario: Index
- **WHEN** `npm run check:skills-index` runs
- **THEN** it exits 0 and the touched skills show their new versions

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

- Which fixed query to use for the evidence run (default: a pack-internal question answerable from the docs site so the run does not depend on paid search).

## Unresolved review findings

Adversarial review: k3 judge, producer `claude-opus-5`, diff mode.
Receipts under `.kbd-orchestrator/phases/research-agent-hardening/review/change-rah-011-integration-evidence-and-docs/`.

### Round 1 (BLOCK: 5 CRITICAL, 3 WARNING, 1 SUGGESTION) — all accepted, none rejected

| # | Sev | Finding | Disposition |
|---|---|---|---|
| 1 | CRITICAL | The Cargo suites named in verification.md were neither run nor recorded BLOCKED, while certification was declared | **Accepted, fixed.** Ran them: learner-model `rpc_roundtrip` 1 passed; prometheus-research `job_execution` 1, `job_lifecycle` 4, `mcp_tools` 3, `sse_stream` 2, lib 3 — all 0 failed. Recorded in evidence §3. The judge was right that "certification passes" was asserted over a gap |
| 2 | CRITICAL | C-01 requires running the generator twice and proving byte-identical output; only one run was recorded | **Accepted, fixed.** Both generators run twice with sha256 before/after; SKILLS.md identical across three hashes, dist tree identical. Table added to evidence §3 |
| 3 | CRITICAL | `site/docs/substrate/prometheus-research.md` and the relocated test were edited outside the declared scope | **Accepted, fixed.** Scope amended above with the rationale for each, and `files.txt` rewritten to include the relocated fixtures the reviewer needs |
| 4 | CRITICAL | Requirement "docs describe what exists" unmet: the in-scope `site/docs/research/deep-research.md` was never written, so no site page describes the driver, labels, or sidecar | **Accepted, fixed.** The page did not exist at all — the spec named a path that had never been created. Written now covering the stage-contract driver, both execution modes, the suffixless package layout, the four claim labels, the derived `verification_status`, the provenance sidecar, agent duties, and scoring; wired into `site/sidebars.js` under a new Research category |
| 5 | CRITICAL | The relocation is not reproducible from the diff: fixtures absent, old-path deletion absent | **Accepted in part, fixed.** The move itself was real (`skills/learn/tests/` no longer exists; the suite runs 58 passed from its new home). The defect was in `files.txt`, which listed only the `.sh` and so gave the judge an incomplete diff. Fixtures now listed |
| 6 | WARNING | The daemon scenario names `research_start`, but the run used `POST /api/v1/jobs` with an asserted equivalence | **Accepted, fixed.** `research_start` invoked over MCP stdio and its reply recorded; equivalence also shown structurally — both call the same `spawn_job` with the same arguments |
| 7 | WARNING | Version identity inconsistent: SKILLS.md 1.8.0, site page "v1.8.0", crate 0.1.0, CHANGELOG all under Unreleased | **Accepted in part, fixed.** The site page claiming v1.8.0 for a 0.1.0 crate was wrong and now states the crate version `/health` actually reports. SKILLS.md 1.8.0 matches `package.json`; entries stay under `[Unreleased]` because this phase does not cut a release — that is a release-time decision, not this change's |
| 8 | WARNING | The regenerated index carries `sync-*` removals and a new `kbd-bottleneck-detector` with no CHANGELOG entry | **Accepted, fixed.** Both recorded in CHANGELOG with an explicit note that this phase regenerates the index carrying them but does not own the moves |
| 9 | SUGGESTION | Scope names `skills-index.json`, which does not exist; the index is SKILLS.md | **Accepted, fixed.** Scope corrected |

Every round-1 finding was legitimate. Two — the unrun Cargo suites and the
never-written site page — were gaps between what the evidence claimed and what
had been done, which is exactly the failure mode adversarial review exists to
catch on a change whose own subject is certification evidence.

### Round 2 (BLOCK: 1 CRITICAL, 2 WARNING — cap reached) — all accepted

| # | Sev | Finding | Disposition |
|---|---|---|---|
| 1 | CRITICAL | The idempotence table showed `—` for the distribution generator's second run while the round-1 disposition claimed both generators ran twice | **Accepted, fixed.** The disposition overstated what the table recorded. Ran `build:distribution` a second time; the dist-tree hash is identical across both runs and the table now carries three hashes. The evidence names the discrepancy rather than quietly filling it in |
| 2 | WARNING | The relocation still has no deletion hunk, so "the old copy is gone" cannot be verified from the packet | **Accepted, fixed.** Verified `git ls-tree HEAD -- skills/learn/tests/` is empty: that path was created by rah-008 in this same uncommitted tree and never reached a commit, so no deletion hunk can exist. Stale `git add -N` entries for it were still in the index and are now cleared with `git rm --cached`; the index matches the disk. Explanation added to the evidence |
| 3 | WARNING | `validate:strict` "PASS (145 skills)" against SKILLS.md's 161 is an unreconciled discrepancy in a certification document | **Accepted, fixed.** The count was also stale (144, not 145). Both numbers are correct and measure different sets — source-tree skills carrying a `SKILL.md` versus skills published to the plugin surfaces. The row now states which, with a short section explaining the difference |

Round 2 found no defect in the shipped work; all three findings were
inaccuracies in the evidence document itself, which is the right thing to catch
on a change whose deliverable *is* evidence. Both rounds passed the findings
sycophancy gate (round 1 score 0.018, round 2 score 0.0, strict). The two-round
cap is reached; the fixes above were applied after it, with the corrections
verifiable by command.
