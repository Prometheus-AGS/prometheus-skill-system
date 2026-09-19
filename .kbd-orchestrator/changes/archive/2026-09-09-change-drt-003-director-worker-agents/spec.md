# change-drt-003-director-worker-agents

**Title:** A director that cannot search and workers that cannot see each other
**Repository:** `prometheus-skill-pack`
**Phase:** deep-research-onyx-parity
**Goal:** G1, G2
**Depends on:** `change-drt-001-dispatch-smoke-and-thread-contracts`, `change-drt-002-thread-scheduler-and-budgets`, `change-drt-004-deterministic-merge`
**Backend:** native-kbd

## Why

The single idea behind Onyx's execution quality is a **constrained orchestrator**:
it has no search or fetch tools (`dr_mock_tools.py` defines exactly four —
`generate_plan`, `research_agent`, `generate_report`, `think_tool`), so it
cannot start answering and must write self-contained task briefs instead. Workers
receive the brief and nothing else — no plan, no chat history, no sibling context.

The pack has no director or worker agent. It also has something Onyx lacks: the
director has read the other dossiers, so it can populate an `avoid` field that
keeps a new thread off ground already covered — without breaking the two-level
rule, because the worker still receives only its brief.

Onyx's `think_tool` returns `"Acknowledged, please continue."`
(`dr_mock_tools.py:112`) and is discarded. Writing the same reflection to
`reflections.md` costs nothing and yields both the reasoning-forcing effect and
an audit trail (analysis D-06).

## What Changes

- `agents/research-director.md`: `tools: Read, Grep, Glob, Write`. Owns the cycle
  loop, dispatches briefs, reads returned dossiers, appends a coverage table and
  gap list to the `plan.md` ledger, decides next threads or done. **Never searches,
  never fetches.**
- `agents/research-worker.md`: `tools: WebSearch, WebFetch, Read, Write`, writing
  only under `threads/<tid>/`. One thread, fresh context, brief only. Reflection
  before every search after the first; open at least the top pages after a search;
  stop on budget, on coverage, or on two consecutive searches yielding nothing new.
- Stage 02 in `run-research.sh` invokes the scheduler **and** the merge. Wiring the scheduler without the merge would leave stage 02 unable to produce `sources/url-list.json`, so the two are wired together or not at all (adversarial round-1 finding 1).
- `SKILL.md` documents the two-level rule and why a third level is refused.

## Scope

- `skills/research/deep-research/agents/research-director.md`
- `skills/research/deep-research/agents/research-worker.md`
- `skills/research/deep-research/scripts/run-research.sh`
- `skills/research/deep-research/SKILL.md`
- `skills/research/deep-research/references/thread-contracts.md`
- `SKILLS.md` (regenerated skills index, C-01)

## Capabilities

- `research-pipeline-execution (director and worker duties added)`

## ADDED Requirements

### Requirement: The director cannot search
The director's `tools:` allowlist SHALL exclude every search and fetch tool, and no source may enter the package except through a worker's `sources.json`.

#### Scenario: Advisory harness
- **WHEN** the director runs on a harness that ignores `tools:` frontmatter
- **THEN** the merge step still refuses any dossier source absent from that thread's `sources.json`, and the violation is a CRITICAL review finding — the allowlist is the intent, the merge is the enforcement

### Requirement: Workers are isolated
A worker SHALL receive its brief and nothing else — no plan, no sibling dossier, no chat history.

#### Scenario: Brief only
- **WHEN** a worker is dispatched
- **THEN** its context contains the brief and the corpus it fetches itself, and a fixture assertion confirms no sibling thread's content is present

### Requirement: Reflection is durable
WHEN a worker searches after its first search, THEN a reflection entry SHALL exist in `threads/<tid>/reflections.md` naming what was found, what is missing, and what query addresses it.

#### Scenario: Reflection precedes the second search
- **WHEN** a fixture worker performs two searches
- **THEN** `reflections.md` holds at least one entry written between them

### Requirement: Two levels, never three
A worker SHALL NOT dispatch sub-workers; a new angle becomes another director-dispatched thread.

#### Scenario: Depth ceiling
- **WHEN** a dossier cites a source that is in no thread's `sources.json`
- **THEN** the merge flags it CRITICAL, which also catches a worker that sub-dispatched

## Constraints

- Implementation-first, integration-only evidence (CLAUDE.md highest-precedence policy): finish the coherent edit batch, then run the smallest full-integration gate named in `verification.md`. No unit tests, mocks, or snapshots count as delivery evidence.
- One Cargo build machine-wide at a time. Check `pgrep -x cargo` before any `cargo` command.
- Local-only validation: no GitHub Actions run is evidence.
- Verification labels are `verified | unverified | blocked | inferred` on claims and `PASS | PASS WITH NOTES | BLOCKED` on provenance. A gate that could not run is recorded BLOCKED with the reason.
- Scripts that launchd may invoke stay bash 3.2 compatible (constraint C-05): no `mapfile`, no `declare -A`; test under `/bin/bash`.
- Every script touched keeps `set -euo pipefail` semantics and non-zero exit on failure; no silent `|| true` on a gate.
- The pack never depends on the Companion or any extension (integration contract rule 1).
- **Stage numbers 02-04 and 09 do not change.** This phase refactors how those stages are produced, never the contract.
- **Onyx is licensed NOASSERTION** (analysis D-08). Mechanisms may be reimplemented freely; Onyx prompt strings must NOT be copied verbatim without a licence check.
- Constraint C-01 (generated artifacts): a change that edits a `SKILL.md` regenerates the skills index and passes `npm run check:skills-index` in the same change.
- **C-01, `run-research.sh` is a distributed artifact.** This change edits it, which desyncs `dist/plugins/claude/prometheus-skill-pack/skills/deep-research/scripts/run-research.sh`. Regenerate the distribution and pass `npm run check:distribution` and `npm run validate:codex` **in this change** — there is no end-of-phase reconciliation, because a desynced driver is defect D-B itself.
- **C-05 binds here.** `run-research.sh` is the script the launchd daemon drives: no `mapfile`, no `declare -A`, and the driver suite runs under `/bin/bash` 3.2 as well as bash 5.

## Open Questions

- Whether `avoid` should be advisory prose or a machine-checked exclusion list (default: prose in the brief, since the worker is a model; the merge catches overlap after the fact).
