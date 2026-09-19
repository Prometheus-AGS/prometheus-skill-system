# change-drt-001-dispatch-smoke-and-thread-contracts

**Title:** Prove concurrent subagent dispatch, then freeze the thread artifact contracts
**Repository:** `prometheus-skill-pack`
**Phase:** deep-research-onyx-parity
**Goal:** G1, G2
**Depends on:** none — this is the phase's first change
**Backend:** native-kbd

## Why

Analyze decision **D-03a** is explicit: sequential Agent-tool dispatch is citable
in this repository, but **parallel** in-session subagent dispatch is not
documented anywhere in it. The adversarial review caught that claim resting on
harness context rather than evidence. Every later change in this phase assumes
workers run concurrently, so that assumption is tested **first**, before any
scheduler is built against it.

The same change freezes the artifact contracts the scheduler and merge will
write, so drt-002 and drt-004 build against a fixed shape rather than inventing
one each.

## What Changes

- `tests/dispatch-smoke.sh`: dispatches two workers concurrently against a
  fixture task, asserts both complete, asserts each ran in an isolated context
  (neither sees the other's scratch file), and records wall-clock overlap as the
  proof of concurrency rather than sequential execution.
- `references/thread-contracts.md`: normative shapes for `brief.json`,
  `dossier.md`, `reflections.md`, `sources.json`, `claims.json` (with the
  verbatim `quote` span that makes stage 05's "read before you label" rule
  mechanically checkable), and `threads/index.json`.
- `references/schemas/thread-brief.schema.json` and `thread-index.schema.json`.
- `templates/thread-brief.json` and `templates/dossier.md`.
- The dossier rule is written from the mechanism, not from Onyx's prose (D-08).

## Scope

- `skills/research/deep-research/tests/dispatch-smoke.sh`
- `skills/research/deep-research/tests/fixtures/thread-smoke/`
- `skills/research/deep-research/references/thread-contracts.md`
- `skills/research/deep-research/references/schemas/thread-brief.schema.json`
- `skills/research/deep-research/references/schemas/thread-index.schema.json`
- `skills/research/deep-research/templates/thread-brief.json`
- `skills/research/deep-research/templates/dossier.md`
- `skills/research/deep-research/tests/thread-contracts.sh`

## Capabilities

- `research-pipeline-execution (thread contracts added)`

## ADDED Requirements

### Requirement: Concurrent dispatch is proven, not assumed
WHEN two workers are dispatched, THEN both SHALL complete and their executions SHALL overlap in wall-clock time, and neither SHALL observe the other's scratch state.

#### Scenario: Two workers, one dispatch
- **WHEN** `tests/dispatch-smoke.sh` dispatches two fixture workers
- **THEN** both write their own output, the union of their start/end windows overlaps, and neither file contains the other's marker
- **AND** if dispatch is serial or isolation fails, the test exits non-zero and the phase re-plans on strategy B (analysis D-03a)

### Requirement: Thread artifacts have one shape
Every artifact a worker writes SHALL validate against the schema named in `thread-contracts.md`, and every claim SHALL carry a verbatim `quote` of at most 40 words attributable to a `source_id` in the same thread's `sources.json`.

#### Scenario: Contract validation
- **WHEN** the fixture thread artifacts are validated
- **THEN** `brief.json` and `index.json` validate against their schemas and every claim's `quote` is non-empty and traceable to a listed source

## Constraints

- Implementation-first, integration-only evidence (CLAUDE.md highest-precedence policy): finish the coherent edit batch, then run the smallest full-integration gate named in `verification.md`. No unit tests, mocks, or snapshots count as delivery evidence.
- One Cargo build machine-wide at a time. Check `pgrep -x cargo` before any `cargo` command; if another build is active, wait or record BLOCKED, never start a competing build. `cargo check -p <crate>` only as a narrowly targeted diagnostic.
- Local-only validation: no GitHub Actions run is evidence.
- Verification labels are `verified | unverified | blocked | inferred` on claims and `PASS | PASS WITH NOTES | BLOCKED` on provenance. A gate that could not run is recorded BLOCKED with the reason, never described as passed.
- Scripts that launchd may invoke stay bash 3.2 compatible (constraint C-05): no `mapfile`, no `declare -A`; test under `/bin/bash`.
- Every script touched keeps `set -euo pipefail` semantics and non-zero exit on failure; no silent `|| true` on a gate.
- The pack never depends on the Companion or any extension (integration contract rule 1); capability is discovered, never assumed (rule 2).
- **Stage numbers 02-04 and 09 do not change.** This phase refactors how those stages are produced, never the contract. The driver validators, `check-research-package.sh`, the daemon mirroring, and every existing fixture must keep passing unchanged (analysis §4.1, report §9 "resist renumbering").
- **Onyx is licensed NOASSERTION** (analysis D-08). Mechanisms may be reimplemented freely; Onyx prompt strings must NOT be copied verbatim without a licence check.
- Constraint C-01 (generated artifacts): any change that edits a `SKILL.md`, a plist, or a systemd unit regenerates the affected surface and passes its drift validator in the same change.

## Open Questions

- Whether the smoke test can assert true parallelism deterministically on every harness, or whether overlap is the strongest portable signal (default: overlap, with the weaker guarantee stated in the test output).

## Unresolved review findings

Adversarial review: k3 judge, producer `claude-opus-5`, diff mode, two rounds
(cap). Receipts under
`.kbd-orchestrator/phases/deep-research-onyx-parity/review/change-drt-001-dispatch-smoke-and-thread-contracts/`.
Both rounds BLOCK, both passed the findings sycophancy gate at 0.0 strict.
**Five findings, all accepted, none rejected.**

### Round 1 — 3 CRITICAL

| # | Finding | Disposition |
|---|---|---|
| 1 | The smoke test never asserted the scratch-state isolation its own spec scenario required | **Accepted.** The gap was real: I had written an isolation *comment* explaining why filesystem isolation is the wrong question, then failed to assert the right one. Each worker now writes a marker and probes for a peer id it was never given, plus a **negative control** that injects a peer and requires the probe to trip — so the assertion cannot pass by never being exercised |
| 2 | The real-harness branch suppressed child failures with `\|\| true` | **Accepted.** Exit codes are captured per child and asserted, and the test now also checks each child wrote the file it was asked for, with its own id and not its sibling's. A harness that fails now fails the gate |
| 3 | Schema validation silently passed when `jsonschema` was unimportable, falling back to a required-key check | **Accepted, and this was the worst of the three.** A fallback that reports a pass for validation that never ran is precisely what the verification rules forbid. Missing `jsonschema` is now **BLOCKED (exit 2)** |

### Round 2 — 2 CRITICAL

| # | Finding | Disposition |
|---|---|---|
| 1 | Only `brief.json` and `index.json` were schema-validated, while the requirement says *every* artifact a worker writes | **Accepted.** Rather than weaken the requirement to match the test, added `thread-sources.schema.json` and `thread-claims.schema.json` and validated both fixtures against them |
| 2 | `--with-harness` reported a missing harness as BLOCKED but still exited 0 | **Accepted — the same rule as round 1 finding 3, violated a second time in a different place.** The branch now exits 2. Verified with a control PATH that carries the core utilities but no harness: exit is 2, not 0 |

### One correction the review did not catch

While running the final gates, `thread-contracts.sh` hit its own BLOCKED path
because Homebrew's `python3` lacks `jsonschema` while `/usr/bin/python3` has it —
so the result depended on PATH ordering. The suite now probes the candidate
interpreters and uses whichever can import the module, and is BLOCKED only when
none can. Found by running the gate under a different PATH, not by inspection.
